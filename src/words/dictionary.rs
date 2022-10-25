use std::{collections::{BTreeMap, HashSet}, cell::RefCell, mem::size_of, io::{Write, Read}};

use const_format::concatcp;
use serde::{Serialize, Deserialize};

use crate::constants::{VERSION, WORD_LENGTH_PADDING};

use super::{WordID, Word, WordError};

const DICT_HEADER: &'static str = concatcp!("DICTINARYDATA_", VERSION);

fn insert_obs(obscurity_index: &mut BTreeMap<u32, RefCell<HashSet<WordID>>>, id: WordID, obs: u32) {
    let mut set = {
        if !obscurity_index.contains_key(&obs) {
            obscurity_index.insert(obs, RefCell::new(HashSet::new()));
        }

        obscurity_index[&obs].borrow_mut()
    };

    set.insert(id);
}

#[derive(Debug)]
pub enum ObscurityMode {
    Exponential(f64),
    Linear(f64),
    Manual
}

#[derive(Serialize, Deserialize)]
struct DictData<'a> {
    header: &'a str,
    name: String,
    words: Box<[Word]>,
}

#[derive(Debug)]
pub struct Dictionary {
    pub(in super) title: String,
    pub(in super) words: Box<[Word]>,
    pub(in super) obscurity_index: BTreeMap<u32, RefCell<HashSet<WordID>>>
}

impl Dictionary {
    pub fn create(words: Box<[Word]>, name: String, mode: ObscurityMode) -> Dictionary {

        let mut dct = Dictionary { words, title: name, obscurity_index: BTreeMap::new() };
        for (i, word) in dct.words.iter_mut().enumerate() {
            let id = i as WordID;

            let obs = match mode {
                ObscurityMode::Exponential(base) => base.powi(i as i32) as u32,
                ObscurityMode::Linear(multiplier) => (multiplier * i as f64) as u32 + 1,
                ObscurityMode::Manual => word.obscurity,
            };

            word.obscurity = obs;

            insert_obs(&mut dct.obscurity_index, id, obs);
        }

        dct
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_words_leq_score(&self, score: u32) -> Box<[Word]> {
        let mut v = Vec::new();

        let stuff = self.obscurity_index.range(..=score);

        for (_, cell) in stuff {
            let set = cell.borrow_mut();
            for thing in set.iter() {
                v.push(self.words[*thing].clone());
            }
        }

        v.into_boxed_slice()
    }

    pub fn save_to<T: Write>(self, writable: &mut T) -> Result<(), WordError> {
        let data = DictData { header: DICT_HEADER, name: self.title, words: self.words };
        
        let mut i = 1;
        let mut written = false;

        // Loop just in case the word strings are unusually large and the size estimate is too small
        while !written {
            let size_estimate = data.words.len() * (size_of::<Word>() + WORD_LENGTH_PADDING * i) + size_of::<DictData>();
    
            let mut alloc = vec![0u8; size_estimate];
            
            match postcard::to_slice(&data, &mut alloc) {
                Ok(out) => {
                    writable.write(out)?;
                    written = true;
                },
                Err(_) => ()
            }
            
            i += 1;
        }

        Ok(())
    }

    pub fn load_from<'a, T: Read>(readable: &mut T) -> Result<Dictionary, WordError> {
        let mut data = Vec::new();
        readable.read_to_end(&mut data)?;

        let dict_data: DictData = postcard::from_bytes(&data[..])?;

        if dict_data.header != DICT_HEADER {
            return Err("Invalid File!")?;
        }

        let words = dict_data.words;
        let name = dict_data.name;

        Ok(Dictionary::create(words, name, ObscurityMode::Manual))
    }
}