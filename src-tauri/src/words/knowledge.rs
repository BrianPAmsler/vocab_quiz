use std::{
    collections::HashMap,
    io::{Read, Write},
    mem::size_of,
    ops::Index,
    sync::Arc,
};

use chrono::{DateTime, NaiveDateTime, Utc};
use rand::Rng;
use serde::{de::Visitor, Deserializer, Serializer};

use struct_version_manager::version_macro::version_mod;

use crate::{error::Error, program::filemanager, tools::dict_map::DictMap};

use super::{Dictionary, WordID};

const KNOW_HEADER: &'static str = "KNOWLEDGEDATA";
const KNOW_VERSION: &'static str = "0.2";

const MIN_HALF_LIFE: f32 = 10.0;

struct TimeVisitor;

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("utc timestamp")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(v, 0).unwrap(), Utc);

        Ok(Some(time))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(self)
    }
}

fn serialize_time<S: Serializer>(
    time: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match time {
        Some(t) => serializer.serialize_some(&t.timestamp()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_time<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error> {
    deserializer.deserialize_option(TimeVisitor)
}

#[version_mod(WordKnowledge)]
mod word_knowledge {
    pub mod v0_2 {
        use chrono::{DateTime, Utc};
        use serde::{Deserialize, Serialize};
        use struct_version_manager::version_macro::version;

        use super::super::deserialize_time;
        use super::super::serialize_time;

        use crate::words::WordID;

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[version("0.2")]
        pub struct WordKnowledge {
            pub word_id: WordID,
            #[serde(
                serialize_with = "serialize_time",
                deserialize_with = "deserialize_time"
            )]
            pub last_practice: Option<DateTime<Utc>>,
            pub half_life: f32,
            pub(in super::super) _pv: (),
        }

        #[derive(Serialize, Deserialize)]
        #[version("0.2")]
        pub struct KnowledgeData {
            pub dict_title: crate::tools::crypt_string::PermutedString,
            pub active_words: usize,
            pub knowledge_data: Box<[u8]>,
        }
    }
}

use word_knowledge::v0_2::KnowledgeData;
pub use word_knowledge::v0_2::WordKnowledge;

impl WordKnowledge {
    pub fn calculate_p_value(&self, practice_time: DateTime<Utc>) -> f32 {
        if self.last_practice.is_none() {
            return 0.0;
        }

        let delta = (practice_time - self.last_practice.unwrap()).num_seconds() as f32 / 60.0;

        2.0f32.powf((-delta) / self.half_life)
    }
}

pub struct Knowledge {
    dict: Arc<Dictionary>,
    knowledge: Box<[WordKnowledge]>,
    active_words: usize,
}

impl Knowledge {
    pub fn create(dict: Arc<Dictionary>) -> Knowledge {
        let mut knowledge = Vec::new();
        knowledge.reserve(dict.words.len());

        for id in dict.get_word_ids().as_ref() {
            let k = WordKnowledge {
                word_id: *id,
                last_practice: None,
                half_life: MIN_HALF_LIFE,
                _pv: (),
            };

            knowledge.push(k);
        }

        Knowledge {
            dict,
            knowledge: knowledge.into_boxed_slice(),
            active_words: 0,
        }
    }

    pub fn estimate_serialized_size(&self) -> usize {
        self.knowledge.len() * size_of::<WordKnowledge>() + size_of::<KnowledgeData>()
    }

    pub fn save_to<T: Write>(&self, writable: &mut T) -> Result<usize, Error> {
        let size_estimate = self.estimate_serialized_size();
        let mut kw_data = vec![0u8; size_estimate];
        let kw_size = postcard::to_slice(&self.knowledge, &mut kw_data)?.len();
        kw_data.truncate(kw_size);

        let data = KnowledgeData {
            dict_title: self.dict.title.clone().into(),
            knowledge_data: kw_data.into_boxed_slice(),
            active_words: self.active_words,
        };

        let mut alloc = vec![0u8; size_estimate];

        let out_len = postcard::to_slice(&data, &mut alloc)?.len();
        alloc.truncate(out_len);

        let size = filemanager::save_file(
            writable,
            KNOW_HEADER.to_owned(),
            KNOW_VERSION.to_owned(),
            alloc.into_boxed_slice(),
        )?;
        Ok(size)
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        for k in self.knowledge.as_mut() {
            k.last_practice = Some(
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(rng.gen::<u32>() as i64, 0).unwrap(),
                    Utc,
                )
                .into(),
            );
            k.half_life = rng.gen_range(MIN_HALF_LIFE..100.0);
        }
    }

    pub fn load_from<'a, T>(readable: &mut T, container: &DictMap) -> Result<Knowledge, Error>
    where
        T: Read,
    {
        let mut file = filemanager::read_file(readable)?;

        if file.header != KNOW_HEADER {
            return Err("Invalid File Header!")?;
        }

        match file.version.as_str() {
            KNOW_VERSION => {
                let data = &mut file.data[..];

                let know_data: KnowledgeData = postcard::from_bytes(&data)?;

                let dict = container
                    .get(&know_data.dict_title.to_string())
                    .ok_or("Dict not found!")?
                    .clone();
                let knowledge = postcard::from_bytes(&know_data.knowledge_data)?;

                Ok(Knowledge {
                    dict,
                    knowledge,
                    active_words: know_data.active_words,
                })
            }
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

    pub fn practice(&mut self, word: WordID, correct: bool) {
        let i: usize = word.into();
        let info = &mut self.knowledge[i];

        let lp = info.last_practice;
        info.last_practice = Some(Utc::now().into());

        match lp {
            Some(time) => {
                let time_delta = info.last_practice.unwrap() - time;
                // > 1 if practiced after expected half-life, < 1 if practiced before
                let time_factor = time_delta.num_minutes() as f32 / info.half_life;

                // A 0 time factor will result in no change, a 1 time factor will change by a factor of 2
                let mut multiplier = 1.0 + 1.0 * time_factor;

                // If the answer was wrong, take reciprocal
                if !correct {
                    multiplier = multiplier.recip();
                }

                info.half_life = f32::max(info.half_life * multiplier, MIN_HALF_LIFE);
            }
            None => {
                if correct {
                    // Set half-life to two days if user alerady knows word the first time seeing it.
                    info.half_life = 60.0 * 24.0 * 2.0;
                } else {
                    info.half_life = MIN_HALF_LIFE;
                }
            }
        }
    }

    pub fn get_word_knowledge(&self, word: WordID) -> &WordKnowledge {
        let i: usize = word.into();

        &self.knowledge[i]
    }

    pub fn get_dict(&self) -> Arc<Dictionary> {
        self.dict.clone()
    }

    pub fn get_active_words(&self) -> usize {
        self.active_words
    }

    pub fn set_active_words(&mut self, amount: usize) {
        self.active_words = amount;
    }
}
