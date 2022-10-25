use std::{time::SystemTime, io::{Write, Read}, mem::size_of, ops::Index};

use const_format::concatcp;
use serde::{Serialize, Deserialize};

use crate::constants::VERSION;

use super::{WordID, Dictionary, WordError};

const KNOW_HEADER: &'static str = concatcp!("KNOWLEDGEDATA_", VERSION);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WordKnowledge {
    word_id: WordID,
    last_practice: Option<SystemTime>,
    confidence_score: i32,
    last_award: i32,
    reinforcement_level: u32
}

pub struct Knowledge<'k> {
    dict: &'k Dictionary,
    knowledge: Box<[WordKnowledge]>,
}

#[derive(Serialize, Deserialize)]
struct KnowledgeData<'a> {
    header: &'a str,
    dict_title: String,
    knowledge: Box<[WordKnowledge]>,
}


impl<'slf> Knowledge<'slf> {
    pub fn create(dict: &Dictionary) -> Knowledge {
        let mut knowledge = Vec::new();
        knowledge.reserve(dict.words.len());

        for (i, _) in dict.words.iter().enumerate() {
            let k = WordKnowledge { word_id: i, last_practice: None, confidence_score: 0, last_award: 0, reinforcement_level: 0 };

            knowledge.push(k);
        }

        Knowledge { dict, knowledge: knowledge.into_boxed_slice() }
    }

    pub fn save_to<T: Write>(mut self, writable: &mut T) -> Result<Knowledge<'slf>, WordError> {
        let data = KnowledgeData { header: KNOW_HEADER, dict_title: self.dict.title.clone(), knowledge: self.knowledge };
        
        let size_estimate = data.knowledge.len() * size_of::<Knowledge>() + size_of::<KnowledgeData>();
    
        let mut alloc = vec![0u8; size_estimate];
        
        let out = postcard::to_slice(&data, &mut alloc)?;
        
        writable.write(out)?;
        
        self.knowledge = data.knowledge;
        Ok(self)
    }

    pub fn load_from<'a, T, I>(readable: &mut T, container: &I) -> Result<Knowledge<'a>, WordError>
    where
        T: Read,
        I: Index<String, Output = &'a Dictionary> {
        let mut data = Vec::new();
        readable.read_to_end(&mut data)?;

        let dict_data: KnowledgeData = postcard::from_bytes(&data)?;

        if dict_data.header != KNOW_HEADER {
            return Err("Invalid File!")?;
        }
    
        let dict = container[dict_data.dict_title];

        Ok(Knowledge { dict, knowledge: dict_data.knowledge })
    }
}