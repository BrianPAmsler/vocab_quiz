use std::{path::{Path, PathBuf}, fs::{read_dir, File, metadata}, rc::Rc};

use crate::{tools::DictMap, error::Error, words::{Dictionary, Word}};

use super::{user::{User}, Progress};

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

pub struct Application {
    user_dir: PathBuf,
    dict_dir: PathBuf,
    users: Vec<User>,
    dicts: DictMap
}

impl Application {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(user_dir: P1, dict_dir: P2) -> Result<Application, Error> {
        let user_dir = to_dir_path!(user_dir);
        let dict_dir = to_dir_path!(dict_dir);
        let users = Vec::new();
        let dicts = DictMap::new();

        Ok(Application { user_dir , dict_dir, users, dicts })
    }

    pub fn load(&mut self, tracker: Option<Progress>) -> Result<(), Error> {
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
            let mut file = File::open(dict_file)?;

            let dict = Dictionary::load_from(&mut file)?;
            self.dicts.insert(Rc::new(dict));

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

    pub fn test(&self) -> String {
        for (k, _) in &self.dicts {
            return k.to_owned();
        }

        "none".to_owned()
    }

    pub fn get_word(&self, dict: &str, word: &str) -> Option<Word> {
        let dict = &self.dicts[dict.to_owned()];
        let id = dict.find_word(word.to_owned())?;

        Some(dict.get_word_from_id(id).clone())
    }
}