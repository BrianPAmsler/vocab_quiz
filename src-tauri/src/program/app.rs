use std::{
    collections::HashMap,
    fs::{metadata, read_dir, File},
    io::Write,
    path::{Path, PathBuf},
    ptr,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    constants::APP_DATA_FOLDER,
    error::Error,
    tools::{dict_map::DictMap, weighted_list::pick_by_weight},
    words::{Dictionary, FileVersion, Knowledge, WordID},
};

use super::{user::User, Progress};

macro_rules! to_dir_path {
    ($path: expr) => {{
        let mut buf = std::path::PathBuf::new();
        buf.push($path);
        if !buf.is_dir() {
            return Err("Path must be a directory.")?;
        }
        buf
    }};
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DictID {
    name: String,
}

impl DictID {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserID {
    name: String,
}

impl UserID {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PracticeDirection {
    Forward,
    Backward
}

struct PracticeSession {
    word_pool: Vec<(WordID, PracticeDirection)>,
    knowledge: Knowledge,
    start_time: DateTime<Utc>,
}

impl PracticeSession {
    fn new(dict: &Dictionary, knowledge: Knowledge, score_max: u32) -> PracticeSession {
        let start_time = Utc::now();
        let potential_word_pool: Vec<(f32, (WordID, PracticeDirection))> = dict
            .get_words_leq_score(score_max)
            .to_vec()
            .into_iter()
            .flat_map(|x| {
                let k = knowledge.get_word_knowledge(x);
                let pv = k.calculate_p_value_forward(start_time);
                let pv2 = k.calculate_p_value_backward(start_time);

                let mut words = Vec::new();
                if pv < 0.6 {
                    words.push((1.0 - pv, (x, PracticeDirection::Forward)));
                }
                if pv2 < 0.6 {
                    words.push((1.0 - pv2, (x, PracticeDirection::Backward)))
                }
                words
            })
            .collect();

        let mut word_pool;
        if potential_word_pool.len() <= 20 {
            word_pool = potential_word_pool.into_iter().map(|x| x.1).collect();
        } else {
            word_pool = Vec::new();

            let mut pool = potential_word_pool.clone();

            if pool.len() < 100 {
                // If pool is small, only consider p value
                for _ in 0..20 {
                    let choice = {
                        let i = pick_by_weight(&pool[..]);
                        pool.remove(i)
                    };

                    word_pool.push(choice.1);
                }
            } else {
                // If pool is big, also consider word obscurity
                for _ in 0..20 {
                    // pick 10
                    let mut picked = Vec::new();
                    while picked.len() < 10 {
                        let choice = pick_by_weight(&pool[..]);

                        if !picked.contains(&choice) {
                            picked.push(choice);
                        }
                    }

                    // re-weight based on word obscurity
                    let max_obs = picked
                        .iter()
                        .map(|x| {
                            let w = dict.get_word_from_id(pool[*x].1.0);

                            w.obscurity
                        })
                        .max()
                        .unwrap();
                    let picked: Vec<(f32, (WordID, PracticeDirection))> = picked
                        .iter()
                        .map(|x| {
                            let id = pool[*x].1;
                            let w = dict.get_word_from_id(id.0);

                            ((max_obs - w.obscurity + 1) as f32, id)
                        })
                        .collect();

                    let choice = pick_by_weight(&picked[..]);
                    word_pool.push(pool.remove(choice).1);
                }
            }
        }

        PracticeSession {
            word_pool,
            knowledge,
            start_time,
        }
    }

    fn pick_word(&mut self) -> (WordID, PracticeDirection) {
        let mut rng = rand::thread_rng();

        let choice = rng.gen_range(0..self.word_pool.len());

        self.word_pool.remove(choice)
    }

    fn get_pool_size(&self) -> usize {
        self.word_pool.len()
    }

    fn recover_knowledge(self) -> Knowledge {
        self.knowledge
    }
}

pub struct Application {
    user_dir: PathBuf,
    dict_dir: PathBuf,
    users: HashMap<String, User>,
    dicts: DictMap,
    current_dict: Option<DictID>,
    current_user: Option<UserID>,
    practice_session: Option<PracticeSession>,
    current_word: Option<(WordID, PracticeDirection)>,
    app_handle: AppHandle
}

impl Application {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        user_dir: P1,
        dict_dir: P2,
        app_handle: AppHandle
    ) -> Result<Application, Error> {
        let user_dir = to_dir_path!(user_dir);
        let dict_dir = to_dir_path!(dict_dir);
        let users = HashMap::<String, User>::new();
        let dicts = DictMap::new();

        Ok(Application {
            user_dir,
            dict_dir,
            users,
            dicts,
            current_dict: None,
            current_user: None,
            practice_session: None,
            current_word: None,
            app_handle
        })
    }

    pub fn load(&mut self, tracker: Option<Progress>) -> Result<(), Error> {
        // Count Dictionaries
        let mut dict_files = Vec::new();
        let mut dict_filesize = 0;

        for r in read_dir(&self.dict_dir)? {
            let dict_path = r?.path();

            match dict_path.extension() {
                Some(ex) => match ex.to_str() {
                    Some("dct") => {
                        dict_filesize += std::fs::metadata(&dict_path)?.len();
                        dict_files.push(dict_path);
                    }
                    _ => (),
                },
                None => (),
            }
        }

        // Count Users
        let mut user_files = Vec::new();
        let mut user_filesize = 0;

        for r in read_dir(&self.user_dir)? {
            let user_path = r?.path();

            match user_path.extension() {
                Some(ex) => match ex.to_str() {
                    Some("usr") => {
                        user_filesize += metadata(&user_path)?.len();
                        user_files.push(user_path);
                    }
                    _ => (),
                },
                None => (),
            }
        }

        // Create progress trackers
        let mut dict_progress = Progress::new(dict_filesize);
        let mut user_progress = Progress::new(user_filesize);

        match tracker {
            Some(mut tracker) => tracker.append(&[&dict_progress, &user_progress]),
            None => (),
        }

        // Load Dictionaries
        let dict_prog = 1.0 / (dict_files.len() as f32);
        for dict_file in dict_files {
            let mut file = File::open(&dict_file)?;

            let r = Dictionary::load_from(&mut file);

            if r.is_ok() {
                let (dict, file_version) = r.unwrap();
                drop(file);

                match file_version {
                    FileVersion::Current => (),
                    FileVersion::Old(_) => {
                        println!("old filetype, converting...");
                        let mut file = File::create(dict_file)?;
                        dict.save_to(&mut file)?
                    }
                };

                self.dicts.insert(Arc::new(dict));

                dict_progress.add_progress(dict_prog);
            } else {
                println!("{}", r.err().unwrap().msg());
            }
        }

        // Load Users
        let user_prog = 1.0 / (user_files.len() as f32);
        for user_file in user_files {
            let mut file = File::open(user_file)?;

            let r = User::load_from(&mut file, &self.dicts);

            if r.is_ok() {
                let user = r.unwrap();
                self.users.insert(user.get_name().to_owned(), user);

                user_progress.add_progress(user_prog);
            }
        }

        Ok(())
    }

    pub fn get_dict_list(&self) -> Box<[DictID]> {
        let list: Box<[DictID]> = (&self.dicts)
            .into_iter()
            .map(|x| DictID {
                name: x.1.get_title().to_owned(),
            })
            .collect();

        list
    }

    pub fn get_pool_size(&self, dict: DictID) -> usize {
        let dict = &self.dicts[&dict.name];
        let user = self
            .users
            .get(&self.current_user.as_ref().unwrap().name)
            .unwrap();

        let knowl = {
            let mut t = None;
            for k in user.get_knowledge() {
                if Arc::ptr_eq(&k.get_dict(), dict) {
                    t = Some(k);
                }
            }

            match t {
                Some(t) => t,
                None => return 0,
            }
        };

        let score_max = knowl.get_active_words() as u32;
        let start_time = Utc::now();

        dict.get_words_leq_score(score_max)
            .to_vec()
            .into_iter()
            .filter_map(|x| {
                let k = knowl.get_word_knowledge(x);
                let pv = k.calculate_p_value_forward(start_time);

                if pv < 0.6 {
                    Some((1.0 - pv, x))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .len()
    }

    pub fn set_dict(&mut self, dict: DictID) {
        self.current_dict = Some(dict);
    }

    pub fn start_practice_session(&mut self) -> bool {
        if self.current_user.is_none() || self.current_dict.is_none() {
            return false;
        }

        let user = self
            .users
            .get_mut(&self.current_user.as_ref().unwrap().name)
            .unwrap();
        let dict = &self.dicts[&self.current_dict.as_ref().unwrap().name.clone()];

        let knowl = {
            let mut t = std::ptr::null();
            for k in user.get_knowledge() {
                if Arc::ptr_eq(&k.get_dict(), dict) {
                    t = k as *const Knowledge;
                }
            }

            user.take_knowledge(t)
                .unwrap_or(Knowledge::create(dict.clone()))
        };

        let s = knowl.get_active_words() as u32;
        let sesh = PracticeSession::new(&dict, knowl, s);

        if sesh.get_pool_size() == 0 {
            user.add_knowledge(sesh.recover_knowledge());
            return false;
        }

        self.practice_session = Some(sesh);

        true
    }

    pub fn get_active_words(&self) -> usize {
        if self.current_user.is_none() || self.current_dict.is_none() {
            return 0;
        }

        let user = self
            .users
            .get(&self.current_user.as_ref().unwrap().name)
            .unwrap();
        let dict = &self.dicts[&self.current_dict.as_ref().unwrap().name.clone()];

        let knowl = {
            let mut t = None;
            for k in user.get_knowledge() {
                if Arc::ptr_eq(&k.get_dict(), dict) {
                    t = Some(k);
                }
            }

            t
        };

        match knowl {
            Some(k) => k.get_active_words(),
            None => 0,
        }
    }

    pub fn set_active_words(&mut self, words: usize) {
        if self.current_user.is_none() || self.current_dict.is_none() {
            return;
        }

        let user = self
            .users
            .get_mut(&self.current_user.as_ref().unwrap().name)
            .unwrap();
        let dict = &self.dicts[&self.current_dict.as_ref().unwrap().name.clone()];

        let mut knowl = {
            let mut t = ptr::null();
            for k in user.get_knowledge() {
                if Arc::ptr_eq(&k.get_dict(), dict) {
                    t = k as *const Knowledge;
                }
            }

            user.take_knowledge(t)
                .unwrap_or(Knowledge::create(dict.clone()))
        };

        knowl.set_active_words(words);
        user.add_knowledge(knowl);
    }

    pub fn pick_next_word(&mut self) {
        if self.practice_session.is_none() {
            return;
        }

        self.current_word = Some(self.practice_session.as_mut().unwrap().pick_word());
    }

    pub fn get_current_word(&self) -> Option<(crate::words::for_frontend::Word, PracticeDirection)> {
        let dict = self.dicts[&self
            .current_dict
            .as_ref()
            .expect("No dictionary selected!")
            .name
            .to_owned()]
            .as_ref();

        let word = self.current_word?;
        Some((dict.get_word_from_id(word.0).clone().into(), word.1))
    }

    pub fn practice_current_word(&mut self, result: bool) {
        let sesh = self.practice_session.as_mut().unwrap();
        let (word, direction) = self.current_word.unwrap();

        sesh.knowledge.practice(word, direction, result);
    }

    pub fn get_session_len(&self) -> usize {
        self.practice_session.as_ref().unwrap().get_pool_size()
    }

    pub fn conclude_session(&mut self) {
        let kw = self.practice_session.take().unwrap().recover_knowledge();

        let user = self
            .users
            .get_mut(&self.current_user.as_ref().unwrap().name)
            .unwrap();

        user.add_knowledge(kw);
    }

    pub fn get_users(&self) -> Box<[UserID]> {
        let mut out = Vec::new();
        out.reserve(self.users.len());

        for (name, _) in &self.users {
            out.push(UserID {
                name: name.to_owned(),
            });
        }

        out.into_boxed_slice()
    }

    pub fn set_current_user(&mut self, user: Option<UserID>) {
        if user.is_some() {
            let mut last_user_path = self.app_handle.path().data_dir().unwrap();
            last_user_path.push(APP_DATA_FOLDER);
            last_user_path.push("lastuser");

            let r = File::create(last_user_path);
            if r.is_ok() {
                let mut f = r.unwrap();

                f.write(user.as_ref().unwrap().name.as_bytes()).unwrap();
            }
        }

        self.current_user = user;
    }

    pub fn save_current_user(&mut self) -> Result<(), Error> {
        let user = self
            .users
            .get_mut(&self.current_user.as_ref().unwrap().name)
            .unwrap();

        let mut user_path = PathBuf::new();
        user_path.push(&self.user_dir);
        user_path.push(user.get_name().to_owned() + ".usr");
        let mut user_file = File::create(user_path)?;

        user.save_to(&mut user_file)?;

        Ok(())
    }

    pub fn get_current_user(&self) -> Option<UserID> {
        self.current_user.clone()
    }

    pub fn create_user(&mut self, name: String) -> Result<(), Error> {
        if name.len() == 0 {
            return Err("Must provide a name!")?;
        }
        if self.users.contains_key(&name) {
            return Err("User with this name already exists!")?;
        }

        let mut user_path = PathBuf::new();
        user_path.push(&self.user_dir);
        user_path.push(name.to_owned() + ".usr");
        let mut user_file = File::create(user_path)?;

        let user = User::create(name);
        user.save_to(&mut user_file)?;

        let id = UserID {
            name: user.get_name().to_owned(),
        };

        self.users.insert(user.get_name().to_owned(), user);

        self.set_current_user(Some(id));

        Ok(())
    }

    pub fn get_user_dir(&self) -> PathBuf {
        self.user_dir.clone()
    }

    pub fn get_dict_dir(&self) -> PathBuf {
        self.dict_dir.clone()
    }

    pub fn get_user_id(&self, name: String) -> Option<UserID> {
        if self.users.contains_key(&name) {
            Some(UserID { name })
        } else {
            None
        }
    }
}
