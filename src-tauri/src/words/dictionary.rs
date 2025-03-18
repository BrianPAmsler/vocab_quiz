use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{Read, Write},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};

use crate::{error::Error, program::filemanager, tools::crypt_string::PermutedString};

use super::Word;

const DICT_HEADER: &'static str = "DICTINARYDATA";
const DICT_VERSION: &'static str = "1.0";

pub enum FileVersion {
    Current,
    Old(String),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordID {
    id: usize,
}

impl WordID {
    fn from(id: usize) -> Self {
        WordID { id }
    }
}

impl Into<usize> for WordID {
    fn into(self) -> usize {
        self.id
    }
}

#[derive(Debug)]
pub enum ObscurityMode {
    Exponential(f64),
    Linear(f64),
    Manual,
}

#[derive(Serialize, Deserialize)]
struct DictData {
    name: PermutedString,
    data: Box<[u8]>,
}

impl DictData {
    pub fn size_of(&self) -> usize {
        let size = self.name.len() + self.data.len() + std::mem::size_of::<Self>();
        size
    }
}

fn insert_obs(obscurity_index: &mut BTreeMap<u32, Mutex<HashSet<WordID>>>, id: WordID, obs: u32) {
    let mut set = {
        if !obscurity_index.contains_key(&obs) {
            obscurity_index.insert(obs, Mutex::new(HashSet::new()));
        }

        obscurity_index[&obs].lock().unwrap()
    };

    set.insert(id);
}

#[derive(Debug)]
pub struct Dictionary {
    pub(super) title: String,
    pub(super) words: Box<[Word]>,
    pub(super) obscurity_index: BTreeMap<u32, Mutex<HashSet<WordID>>>,
    pub(super) word_index: HashMap<String, WordID>,
}

impl Dictionary {
    pub fn create(words: Box<[Word]>, name: String, mode: ObscurityMode) -> Dictionary {
        let mut dct = Dictionary {
            words,
            title: name,
            obscurity_index: BTreeMap::new(),
            word_index: HashMap::new(),
        };
        for (i, word) in dct.words.iter_mut().enumerate() {
            let id = WordID::from(i);

            let obs = match mode {
                ObscurityMode::Exponential(base) => base.powi(i as i32) as u32,
                ObscurityMode::Linear(multiplier) => (multiplier * i as f64) as u32 + 1,
                ObscurityMode::Manual => word.obscurity,
            };

            word.obscurity = obs;

            insert_obs(&mut dct.obscurity_index, id, obs);
            dct.word_index.insert(word.text.to_owned(), id);
        }

        dct
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_words_leq_score(&self, score: u32) -> Box<[WordID]> {
        let mut v = Vec::new();

        let stuff = self.obscurity_index.range(..=score);

        for (_, mutex) in stuff {
            let set = mutex.lock().unwrap();
            for thing in set.iter() {
                v.push(thing.to_owned());
            }
        }

        v.into_boxed_slice()
    }

    pub fn find_word(&self, word: String) -> Option<WordID> {
        match self.word_index.get(&word) {
            Some(id) => Some(*id),
            None => None,
        }
    }

    fn get_id_from_index(&self, index: usize) -> Option<WordID> {
        if index >= self.words.len() {
            return None;
        }

        Some(WordID { id: index })
    }

    pub fn get_word_from_id(&self, id: WordID) -> &Word {
        &self.words[id.id]
    }

    pub fn get_word_ids(&self) -> Box<[WordID]> {
        let mut ids = Vec::new();
        ids.reserve(self.words.len());

        for (i, _) in self.words.iter().enumerate() {
            ids.push(WordID::from(i));
        }

        ids.into_boxed_slice()
    }

    pub fn save_to<T: Write>(&self, writable: &mut T) -> Result<(), Error> {
        let mut alloc = vec![0u8; std::mem::size_of_val(&(*self.words))];
        let num_bytes = postcard::to_slice(&self.words, &mut alloc)?.len();
        alloc.truncate(num_bytes);

        let data = alloc.into_boxed_slice();

        let data = DictData {
            name: self.title.to_owned().into(),
            data,
        };

        let mut final_alloc = vec![0u8; data.size_of()];
        let num_bytes = postcard::to_slice(&data, &mut final_alloc)?.len();
        final_alloc.truncate(num_bytes);

        filemanager::save_file(
            writable,
            DICT_HEADER.to_owned(),
            DICT_VERSION.to_owned(),
            final_alloc.into_boxed_slice(),
        )?;

        Ok(())
    }

    pub fn load_from<'a, T: Read>(readable: &mut T) -> Result<(Dictionary, FileVersion), Error> {
        let mut file = filemanager::read_file(readable)?;

        if file.header != DICT_HEADER {
            return Err("Invalid File Header!")?;
        }

        match file.version.as_str() {
            DICT_VERSION => {
                let data = &mut file.data[..];

                let dict_data: DictData = postcard::from_bytes(&data)?;

                let words = postcard::from_bytes(&dict_data.data)?;
                let name = dict_data.name;

                Ok((
                    Dictionary::create(words, name.to_string(), ObscurityMode::Manual),
                    FileVersion::Current,
                ))
            }
            v => {
                let dict = match v {
                    _ => {
                        println!("{}", v);
                        Err("Unknown File Version!")?
                    }
                };

                Ok((dict, FileVersion::Old(v.to_owned())))
            }
        }
    }
}
