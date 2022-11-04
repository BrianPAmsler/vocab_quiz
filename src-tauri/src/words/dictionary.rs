use std::{collections::{BTreeMap, HashSet, HashMap}, io::{Write, Read}, sync::Mutex};


use serde::{Serialize, Deserialize};

use crate::{constants::{VERSION}, error::Error, tools::crypt_string::EncryptedString};

use super::{Word};

const DICT_HEADER_: &'static str = "DICTINARYDATA_";

pub enum FileVersion {
    Current,
    Old(String)
}

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
    name: EncryptedString,
    data: Box<[u8]>,
}

impl<'a> DictData<'a> {
    pub fn size_of(&self) -> usize {
        let size = self.header.len() + self.name.as_str().len() + self.data.len() + std::mem::size_of::<Self>();
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
    pub(in super) title: String,
    pub(in super) words: Box<[Word]>,
    pub(in super) obscurity_index: BTreeMap<u32, Mutex<HashSet<WordID>>>,
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
            None => None
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

    pub fn save_to<T: Write>(mut self, writable: &mut T) -> Result<Self, Error> {
        let mut alloc = vec![0u8; std::mem::size_of_val(&(*self.words))];
        let num_bytes = postcard::to_slice(&self.words, &mut alloc)?.len();
        alloc.truncate(num_bytes);

        let data = alloc.into_boxed_slice();

        let header = DICT_HEADER_.to_owned() + VERSION;
        let data = DictData { header: header.as_str(), name: self.title.into(), data };
        
        let mut final_alloc = vec![0u8; data.size_of()];
        let num_bytes = postcard::to_slice(&data, &mut final_alloc)?.len();
        final_alloc.truncate(num_bytes);

        writable.write(&mut final_alloc)?;

        self.title = data.name.to_string();

        Ok(self)
    }

    pub fn load_from<'a, T: Read + Write>(read_writable: &mut T) -> Result<(Dictionary, FileVersion), Error> {
        let mut data = Vec::new();
        read_writable.read_to_end(&mut data)?;

        let dict_data: DictData = postcard::from_bytes(&data)?;

        let header = {if dict_data.header.len() < DICT_HEADER_.len() { "" } else {&dict_data.header[..DICT_HEADER_.len()]}};
        let version = &dict_data.header[DICT_HEADER_.len()..];
        if header != DICT_HEADER_ {
            return Err("Invalid file header!")?;
        }

        match version {
            VERSION => {
                let words = postcard::from_bytes(&dict_data.data)?;
                let name = dict_data.name;

                Ok((Dictionary::create(words, name.to_string(), ObscurityMode::Manual), FileVersion::Current))
            },
            v => {
                let dict = match v {
                    "v0.02" => {
                        #[derive(Serialize, Deserialize)]
                        struct DictDataV0_02<'a> {
                            header: &'a str,
                            name: String,
                            data: Box<[u8]>,
                        }

                        let dict_data: DictDataV0_02 = postcard::from_bytes(&data)?;

                        let words = postcard::from_bytes(&dict_data.data)?;
                        let name = dict_data.name;
        
                        Dictionary::create(words, name.to_string(), ObscurityMode::Manual)
                    },
                    "v0.01" => {
                        #[derive(Debug, Clone, Serialize, Deserialize)]
                        pub struct WordV0_01 {
                            pub text: String,
                            pub definition: String,
                            pub pronunciation: Option<String>,
                            pub obscurity: u32
                        }
                        
                        #[derive(Serialize, Deserialize)]
                        struct DictDataV0_01<'a> {
                            header: &'a str,
                            name: String,
                            words: Box<[WordV0_01]>,
                        }

                        impl Into<Word> for WordV0_01 {
                            fn into(self) -> Word {
                                Word { text: self.text, definition: self.definition, pronunciation: self.pronunciation, obscurity: self.obscurity }
                            }
                        }

                        let dict_data = postcard::from_bytes::<DictDataV0_01>(&data)?;

                        let words = {
                            let mut words = Vec::new();
                            words.reserve(dict_data.words.len());

                            for word in dict_data.words.into_vec() {
                                words.push(word.into());
                            }
                            
                            words.into_boxed_slice()
                        };

                        let name = dict_data.name;

                        Dictionary::create(words, name, ObscurityMode::Manual)
                    },
                    _ => Err("Unknown file version!")?
                };

                Ok((dict, FileVersion::Old(v.to_owned())))
            }
        }
    }
}