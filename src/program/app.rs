use std::{path::{Path, PathBuf}, fs::{read_dir, File}};

use crate::{tools::DictMap, error::Error};

use super::{user::{User, self}};

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

        // Load users
        for r in read_dir(&user_dir)? {
            let user_path = r?.path();

            match user_path.extension() {
                Some(ex) => {
                    match ex.to_str() {
                        Some("usr") => {
                            let mut user_file = File::open(user_path)?;
                            let user = User::load_from(&mut user_file, &dicts)?;
                        } 
                        _ => ()
                    }
                }
                None => ()
            }
        }

        Ok(Application { user_dir , dict_dir, users, dicts })
    }
}