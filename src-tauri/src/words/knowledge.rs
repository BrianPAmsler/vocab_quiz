use std::{io::{Write, Read}, mem::size_of, ops::Index, sync::Arc};

use chrono::{DateTime, Utc, NaiveDateTime};
use rand::Rng;
use serde::{Serialize, Deserialize, de::Visitor, Deserializer};

use struct_version_manager::version_macro::version_mod;

use crate::{error::Error, tools::crypt_string::PermutedString, program::filemanager};

use super::{WordID, Dictionary};

const KNOW_HEADER: &'static str = "KNOWLEDGEDATA";
const KNOW_VERSION: &'static str = "0.1";

struct TimeVisitor;

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("utc timestamp")
    }
    
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(v, 0).unwrap(), Utc);

        Ok(Some(time))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>, {
        deserializer.deserialize_i64(self)
    }
}

#[version_mod(WordKnowledge)]
mod word_knowledge {
    pub mod v0_1 {
        use chrono::{DateTime, Utc};
        use serde::{Serialize, Deserialize, Serializer, Deserializer};
        use struct_version_manager::version_macro::version;

        use crate::words::{WordID, knowledge::TimeVisitor};

        fn serialize_time<S: Serializer>(time: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error> {
            match time {
                Some(t) => {
                    serializer.serialize_some(&t.timestamp())
                },
                None => {
                    serializer.serialize_none()
                }
            }
        }
        
        fn deserialize_time<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error> {
            deserializer.deserialize_option(TimeVisitor)
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[version("0.1")]
        pub struct WordKnowledge {
            pub word_id: WordID,
            #[serde(serialize_with = "serialize_time", deserialize_with = "deserialize_time")]
            pub last_practice: Option<DateTime<Utc>>,
            pub confidence_score: i32,
            pub last_award: i32,
            pub reinforcement_level: u32,
            pub (in super::super) _pv: ()
        }
    }
}

pub use word_knowledge::v0_1::WordKnowledge;

pub struct Knowledge {
    dict: Arc<Dictionary>,
    knowledge: Box<[WordKnowledge]>,
}

#[derive(Serialize, Deserialize)]
struct KnowledgeData {
    dict_title: PermutedString,
    knowledge_data: Box<[u8]>,
}

impl Knowledge {
    pub fn create(dict: Arc<Dictionary>) -> Knowledge {
        let mut knowledge = Vec::new();
        knowledge.reserve(dict.words.len());

        for id in dict.get_word_ids().as_ref() {
            let k = WordKnowledge { word_id: *id, last_practice: None, confidence_score: 0, last_award: 0, reinforcement_level: 0, _pv: () };

            knowledge.push(k);
        }

        Knowledge { dict, knowledge: knowledge.into_boxed_slice() }
    }

    pub fn estimate_serialized_size(&self) -> usize {
        self.knowledge.len() * size_of::<WordKnowledge>() + size_of::<KnowledgeData>()
    }

    pub fn save_to<T: Write>(&self, writable: &mut T) -> Result<usize, Error> {
        let size_estimate =  self.estimate_serialized_size();
        let mut kw_data = vec![0u8; size_estimate];
        let kw_size = postcard::to_slice(&self.knowledge, &mut kw_data)?.len();
        kw_data.truncate(kw_size);

        let data = KnowledgeData { dict_title: self.dict.title.clone().into(), knowledge_data: kw_data.into_boxed_slice() };

        let mut alloc = vec![0u8; size_estimate];

        let out_len = postcard::to_slice(&data, &mut alloc)?.len();
        alloc.truncate(out_len);

        let size = filemanager::save_file(writable, KNOW_HEADER.to_owned(), KNOW_VERSION.to_owned(), alloc.into_boxed_slice())?;
        Ok(size)
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for k in self.knowledge.as_mut() {
            k.last_practice = Some(DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(rng.gen::<u32>() as i64, 0).unwrap(), Utc).into());
            k.confidence_score = rng.gen();
            k.last_award = rng.gen();
            k.reinforcement_level = rng.gen();
        }
    }

    pub fn load_from<'a, T, I>(readable: &mut T, container: &I) -> Result<Knowledge, Error>
    where
        T: Read,
        I: Index<String, Output = Arc<Dictionary>> {
        
        let mut file = filemanager::read_file(readable)?;

        if file.header != KNOW_HEADER {
            return Err("Invalid File Header!")?;
        }

        match file.version.as_str() {
            KNOW_VERSION => {
                let data = &mut file.data[..];
        
                let know_data: KnowledgeData = postcard::from_bytes(&data)?;

                let dict = container.index(know_data.dict_title.to_string()).clone();
                let knowledge = postcard::from_bytes(&know_data.knowledge_data)?;
        
                Ok(Knowledge { dict: dict.clone(), knowledge })
            },
            v => {
                let knowl = match v {
                    _ => {
                        println!("{}", v);
                        Err("Unknown File Version!")?
                    }
                };

                Ok(knowl)
            }
        }
    }

    pub fn practice(&mut self, word: WordID, award: i32) {
        let i: usize = word.into();
        let info = &mut self.knowledge[i];

        info.last_practice = Some(Utc::now().into());
        info.last_award = award;
        info.confidence_score += award;

        if award > 0 {
            info.reinforcement_level += 1;
        } else if award < 0 && info.reinforcement_level > 0 {
            info.reinforcement_level -= 1;
        }
    }

    pub fn get_word_knowledge(&self, word: WordID) -> &WordKnowledge {
        let i: usize = word.into();

        &self.knowledge[i]
    }

    pub fn get_dict(&self) -> Arc<Dictionary> {
        self.dict.clone()
    }
}