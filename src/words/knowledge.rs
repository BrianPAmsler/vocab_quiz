use std::{time::SystemTime, io::{Write, Read}, mem::size_of, ops::Index, rc::Rc, str::FromStr};

use chrono::{DateTime, TimeZone, Utc, Date, NaiveDateTime};
use const_format::concatcp;
use serde::{Serialize, Deserialize, de::Visitor};

use crate::constants::VERSION;

use super::{WordID, Dictionary, WordError};

const KNOW_HEADER: &'static str = concatcp!("KNOWLEDGEDATA_", VERSION);

#[derive(Debug, Clone, Copy)]
pub struct TimeWrapper {
    time: DateTime<Utc>
}

impl TimeWrapper {
    pub fn time(self) -> DateTime<Utc> {
        self.time
    }
}

impl From<DateTime<Utc>> for TimeWrapper {
    fn from(time: DateTime<Utc>) -> Self {
        TimeWrapper { time }
    }
}

impl Serialize for TimeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_i64(self.time.timestamp())
    }
}

struct TimeVisitor;

impl<'de> Visitor<'de> for TimeVisitor {
    type Value = TimeWrapper;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("utc string")
    }
    
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(v, 0), Utc);

        Ok(TimeWrapper {time})
    }
}

impl<'de> Deserialize<'de> for TimeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_i64(TimeVisitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordKnowledge {
    pub word_id: WordID,
    pub last_practice: Option<TimeWrapper>,
    pub confidence_score: i32,
    pub last_award: i32,
    pub reinforcement_level: u32,
    _pv: ()
}

pub struct Knowledge {
    dict: Rc<Dictionary>,
    knowledge: Box<[WordKnowledge]>,
}

#[derive(Serialize, Deserialize)]
struct KnowledgeData<'a> {
    header: &'a str,
    dict_title: String,
    knowledge: Box<[WordKnowledge]>,
}

impl Knowledge {
    pub fn create(dict: Rc<Dictionary>) -> Knowledge {
        let mut knowledge = Vec::new();
        knowledge.reserve(dict.words.len());

        for id in dict.get_word_ids().as_ref() {
            let k = WordKnowledge { word_id: *id, last_practice: None, confidence_score: 0, last_award: 0, reinforcement_level: 0, _pv: () };

            knowledge.push(k);
        }

        Knowledge { dict, knowledge: knowledge.into_boxed_slice() }
    }

    pub fn save_to<T: Write>(mut self, writable: &mut T) -> Result<Knowledge, WordError> {
        let data = KnowledgeData { header: KNOW_HEADER, dict_title: self.dict.title.clone(), knowledge: self.knowledge };
        
        let size_estimate = data.knowledge.len() * size_of::<Knowledge>() + size_of::<KnowledgeData>();
    
        let mut alloc = vec![0u8; size_estimate];
        
        let out = postcard::to_slice(&data, &mut alloc)?;
        
        writable.write(out)?;
        
        self.knowledge = data.knowledge;
        Ok(self)
    }

    pub fn load_from<'a, T, I>(readable: &mut T, container: &I) -> Result<Knowledge, WordError>
    where
        T: Read,
        I: Index<String, Output = Rc<Dictionary>> {
        let mut data = Vec::new();
        readable.read_to_end(&mut data)?;

        let dict_data: KnowledgeData = postcard::from_bytes(&data)?;

        if dict_data.header != KNOW_HEADER {
            return Err("Invalid File!")?;
        }

        let dict = container.index(dict_data.dict_title).clone();

        Ok(Knowledge { dict: dict.clone(), knowledge: dict_data.knowledge })
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

    pub fn get_dict(&self) -> Rc<Dictionary> {
        self.dict.clone()
    }
}