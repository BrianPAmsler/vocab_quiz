use std::{collections::{BTreeMap, HashSet, HashMap}, cell::RefCell, mem::size_of, io::{Write, Read}};

use const_format::concatcp;
use serde::{Serialize, Deserialize};

use crate::constants::{VERSION, WORD_LENGTH_PADDING};

use super::{Word, WordError};

const DICT_HEADER: &'static str = concatcp!("DICTINARYDATA_", VERSION);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordID {
    id: usize
}

impl WordID {
    fn from(id: usize) -> Self {
        WordID {id}
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
    Manual
}

#[derive(Serialize, Deserialize)]
struct DictData<'a> {
    header: &'a str,
    name: String,
    words: Box<[Word]>,
}

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
pub struct Dictionary {
    pub(in super) title: String,
    pub(in super) words: Box<[Word]>,
    pub(in super) obscurity_index: BTreeMap<u32, RefCell<HashSet<WordID>>>,
    pub(in super) word_index: HashMap<String, WordID>
}

impl Dictionary {
    pub fn create(words: Box<[Word]>, name: String, mode: ObscurityMode) -> Dictionary {

        let mut dct = Dictionary { words, title: name, obscurity_index: BTreeMap::new(), word_index: HashMap::new() };
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

    pub fn get_words_leq_score(&self, score: u32) -> Box<[Word]> {
        let mut v = Vec::new();

        let stuff = self.obscurity_index.range(..=score);

        for (_, cell) in stuff {
            let set = cell.borrow_mut();
            for thing in set.iter() {
                v.push(self.words[thing.id].clone());
            }
        }

        v.into_boxed_slice()
    }

    pub fn find_word(&self, word: String) -> Option<WordID> {
        match self.word_index.get(&word) {
            Some(id) => Some(*id),
            None => None
        }
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