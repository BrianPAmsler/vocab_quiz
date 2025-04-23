use std::{
    io::{Read, Write},
    mem::size_of,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use serde::{de::Visitor, Deserializer, Serializer};

use struct_version_manager::version_macro::version_mod;

use crate::{error::Error, program::{filemanager, PracticeDirection}, tools::dict_map::DictMap};

use super::{Dictionary, WordID};

const KNOW_HEADER: &'static str = "KNOWLEDGEDATA";
const KNOW_VERSION: &'static str = "0.3";

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
        let time = DateTime::<Utc>::from_timestamp(v, 0);

        Ok(time)
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
    pub mod v0_3 {
        use chrono::{DateTime, Utc};
        use serde::{Deserialize, Serialize};
        use struct_version_manager::version_macro::version;

        use super::super::deserialize_time;
        use super::super::serialize_time;

        use crate::words::WordID;

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[version("0.3")]
        pub struct WordKnowledge {
            pub word_id: WordID,
            #[serde(
                serialize_with = "serialize_time",
                deserialize_with = "deserialize_time"
            )]
            pub last_practice: Option<DateTime<Utc>>,
            #[serde(
                serialize_with = "serialize_time",
                deserialize_with = "deserialize_time"
            )]
            pub last_practice_reverse: Option<DateTime<Utc>>,
            pub half_life: f32,
            pub half_life_reverse: f32,
            pub(in super::super) _pv: (),
        }
    }
    
    pub mod v0_2 {
        use chrono::{DateTime, Utc};
        use serde::{Deserialize, Serialize};
        use struct_version_manager::convert::ConvertTo;
        use struct_version_manager::version_macro::converts_to;
        use struct_version_manager::version_macro::version;

        use super::super::deserialize_time;
        use super::super::serialize_time;

        use crate::words::knowledge::MIN_HALF_LIFE;
        use crate::words::WordID;

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[version("0.2")]
        #[converts_to("0.3")]
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

        impl ConvertTo<super::v0_3::WordKnowledge> for WordKnowledge {
            fn convert_to(self) -> super::v0_3::WordKnowledge {
                super::v0_3::WordKnowledge {
                    word_id: self.word_id,
                    last_practice: self.last_practice,
                    last_practice_reverse: None,
                    half_life: self.half_life,
                    half_life_reverse: (self.half_life + MIN_HALF_LIFE) / 2.0,
                    _pv: self._pv
                }
            }
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
pub use word_knowledge::v0_3::WordKnowledge;

impl WordKnowledge {
    pub fn calculate_p_value_forward(&self, practice_time: DateTime<Utc>) -> f32 {
        if self.last_practice.is_none() {
            return 0.0;
        }

        let delta = (practice_time - self.last_practice.unwrap()).num_seconds() as f32 / 60.0;

        2.0f32.powf((-delta) / self.half_life)
    }

    pub fn calculate_p_value_backward(&self, practice_time: DateTime<Utc>) -> f32 {
        if self.last_practice.is_none() {
            return 0.0;
        }

        let delta = (practice_time - self.last_practice.unwrap()).num_seconds() as f32 / 60.0;

        2.0f32.powf((-delta) / self.half_life_reverse)
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
                last_practice_reverse: None,
                half_life: MIN_HALF_LIFE,
                half_life_reverse: MIN_HALF_LIFE,
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

    // pub fn randomize(&mut self) {
    //     let mut rng = rand::thread_rng();
    //     for k in self.knowledge.as_mut() {
    //         k.last_practice = Some(
    //             DateTime::<Utc>::from_utc(
    //                 NaiveDateTime::from_timestamp_opt(rng.gen::<u32>() as i64, 0).unwrap(),
    //                 Utc,
    //             )
    //             .into(),
    //         );
    //         k.half_life = rng.gen_range(MIN_HALF_LIFE..100.0);
    //     }
    // }

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
            },
            "0.2" => {
                let data = &mut file.data[..];

                let know_data: KnowledgeData = postcard::from_bytes(&data)?;

                let dict = container
                    .get(&know_data.dict_title.to_string())
                    .ok_or("Dict not found!")?
                    .clone();
                let knowledge: Box<[word_knowledge::v0_2::WordKnowledge]> = postcard::from_bytes(&know_data.knowledge_data)?;
                let knowledge = knowledge.into_vec().into_iter().map(|know| {
                    know.upgrade()
                }).collect();

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

    pub fn practice(&mut self, word: WordID, direction: PracticeDirection, correct: bool) {
        let i: usize = word.into();
        let info = &mut self.knowledge[i];

        let mut lp = Some(Utc::now().into());
        let (now, half_life) = match direction {
            PracticeDirection::Forward => {std::mem::swap(&mut info.last_practice, &mut lp); (&info.last_practice, &mut info.half_life)},
            PracticeDirection::Backward => {std::mem::swap(&mut info.last_practice_reverse, &mut lp); (&info.last_practice_reverse, &mut info.half_life_reverse)}
        };

        match lp {
            Some(time) => {
                let time_delta = now.unwrap() - time;
                // > 1 if practiced after expected half-life, < 1 if practiced before
                let time_factor = time_delta.num_minutes() as f32 / *half_life;

                // A 0 time factor will result in no change, a 1 time factor will change by a factor of 2
                let mut multiplier = 1.0 + 1.0 * time_factor;

                // If the answer was wrong, take reciprocal
                if !correct {
                    multiplier = multiplier.recip();
                }

                *half_life = f32::max(*half_life * multiplier, MIN_HALF_LIFE);
            }
            None => {
                if correct {
                    // Set half-life to two days if user alerady knows word the first time seeing it.
                    *half_life = 60.0 * 24.0 * 2.0;
                } else {
                    *half_life = MIN_HALF_LIFE;
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
