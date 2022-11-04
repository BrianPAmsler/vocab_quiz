use std::{path::{Path, PathBuf}, fs::{read_dir, File, metadata}, sync::Arc};

use rand::Rng;
use serde::{Serialize, Deserialize};

use crate::{tools::dict_map::DictMap, error::Error, words::{Dictionary, WordID, FileVersion}};

use super::{user::User, Progress};

macro_rules! to_dir_path {
    ($path: expr) => {
        {
            let mut buf = std::path::PathBuf::new();
            buf.push($path);
            if !buf.is_dir() {
                return Err("Path must be a directory.")?;
            }
            buf
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct DictID {
    name: String
}

impl DictID {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

pub struct UserID;

pub struct Application {
    user_dir: PathBuf,
    dict_dir: PathBuf,
    users: Vec<User>,
    dicts: DictMap,
    current_dict: Option<DictID>,
    current_user: Option<UserID>,
    current_word: Option<WordID>
}

impl Application {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(user_dir: P1, dict_dir: P2) -> Result<Application, Error> {
        let user_dir = to_dir_path!(user_dir);
        let dict_dir = to_dir_path!(dict_dir);
        let users = Vec::new();
        let dicts = DictMap::new();

        Ok(Application { user_dir , dict_dir, users, dicts, current_dict: None, current_user: None, current_word: None })
    }

    pub fn load(&mut self, tracker: Option<Progress>) -> Result<(), anyhow::Error> {
        // Count Dictionaries
        let mut dict_files = Vec::new();
        let mut dict_filesize = 0;

        for r in read_dir(&self.dict_dir)? {
            let dict_path = r?.path();

            match dict_path.extension() {
                Some(ex) => {
                    match ex.to_str() {
                        Some("dct") => {
                            dict_filesize += std::fs::metadata(&dict_path)?.len();
                            dict_files.push(dict_path);
                        } 
                        _ => ()
                    }
                }
                None => ()
            }
        }

        // Count Users
        let mut user_files = Vec::new();
        let mut user_filesize = 0;

        for r in read_dir(&self.user_dir)? {
            let user_path = r?.path();

            match user_path.extension() {
                Some(ex) => {
                    match ex.to_str() {
                        Some("usr") => {
                            user_filesize += metadata(&user_path)?.len();
                            user_files.push(user_path);
                        } 
                        _ => ()
                    }
                }
                None => ()
            }
        }

        // Create progress trackers
        let mut dict_progress = Progress::new(dict_filesize);
        let mut user_progress = Progress::new(user_filesize);

        match tracker {
            Some(mut tracker) => tracker.append(&[&dict_progress, &user_progress]),
            None => ()
        }

        // Load Dictionaries
        let dict_prog = 1.0 / (dict_files.len() as f32);
        for dict_file in dict_files {
            let mut file = File::open(&dict_file)?;

            let (dict, file_version) = Dictionary::load_from(&mut file)?;
            drop(file);

            let dict = match file_version {
                FileVersion::Current => dict,
                FileVersion::Old(_) => {
                    println!("old filetype, converting...");
                    let mut file = File::create(dict_file)?;
                    dict.save_to(&mut file)?
                }
            };

            self.dicts.insert(Arc::new(dict));

            dict_progress.add_progress(dict_prog);
        }

        // Load Users
        let user_prog = 1.0 / (user_files.len() as f32);
        for user_file in user_files {
            let mut file = File::open(user_file)?;

            let user = User::load_from(&mut file, &self.dicts)?;
            self.users.push(user);

            user_progress.add_progress(user_prog);
        }

        Ok(())
    }

    pub fn get_dict_list(&self) -> Box<[DictID]> {
        let mut list = Vec::new();

        for (k, _) in &self.dicts {
            list.push(DictID { name: k.to_owned() });
        }

        list.into_boxed_slice()
    }

    pub fn set_dict(&mut self, dict: DictID) {
        self.current_dict = Some(dict);
    }

    pub fn pick_next_word(&mut self) {
        let dict = self.dicts[self.current_dict.as_ref().expect("No dictionary selected!").name.to_owned()].as_ref();
        let ids = dict.get_words_leq_score(100);
        
        let mut rng = rand::thread_rng();
        let t = rng.gen_range(0..ids.len());

        self.current_word = Some(ids[t]);
    }

    pub fn get_current_word(&self) -> Option<crate::words::for_frontend::Word> {
        let dict = self.dicts[self.current_dict.as_ref().expect("No dictionary selected!").name.to_owned()].as_ref();

        Some(dict.get_word_from_id(self.current_word?).clone().into())
    }
}