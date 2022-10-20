use std::{time::SystemTime, collections::{BTreeMap, BTreeSet,  HashSet}, cell::RefCell, f64::consts::E, borrow::BorrowMut, clone};

const MAX_AWARD: u32 = 50;
const LOGARITHMIC_BASE: f64 = E;
const LINEAR_MULTIPLIER: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub definition: String,
    pub pronunciation: Option<String>,
    pub obscurity: u32
}

type WordID = usize;

#[derive(Clone, Debug)]
struct WordKnowledge {
    word_id: WordID,
    last_practice: Option<SystemTime>,
    confidence_score: i32,
    last_award: i32,
    reinforcement_level: u32
}

#[derive(Debug)]
pub enum ObscurityMode {
    Exponential,
    Linear,
    Manual
}

#[derive(Debug)]
pub struct Dictionary {
    words: Box<[Word]>,
    knowledge: Box<[WordKnowledge]>,
    obscurity_index: BTreeMap<u32, RefCell<HashSet<WordID>>>
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

fn delete_obs(obscurity_index: &mut BTreeMap<u32, RefCell<HashSet<WordID>>>, id: WordID, obs: u32) {
    let should_remove = {
        if obscurity_index.contains_key(&obs) {
            let mut set = obscurity_index[&obs].borrow_mut();

            set.remove(&id);

            set.len() == 0
        } else {
            false
        }
    };

    if should_remove {
        obscurity_index.remove(&obs);
    }
}

impl Dictionary {
    pub fn create(words: Box<[Word]>, mode: ObscurityMode) -> Dictionary {
        let knowledge = vec![WordKnowledge { word_id: 0, last_practice: None, confidence_score: 0, last_award: 0, reinforcement_level: 0}; words.len()];

        let mut dct = Dictionary { words, knowledge: knowledge.into_boxed_slice(), obscurity_index: BTreeMap::new() };
        for (i, word) in dct.words.iter_mut().enumerate() {
            let id = i as WordID;

            let kw = &mut dct.knowledge[i];
            kw.word_id = id;

            let obs = match mode {
                ObscurityMode::Exponential => LOGARITHMIC_BASE.powi(i as i32) as u32,
                ObscurityMode::Linear => (LINEAR_MULTIPLIER * i as f64) as u32 + 1,
                ObscurityMode::Manual => word.obscurity,
            };

            word.obscurity = obs;

            insert_obs(&mut dct.obscurity_index, id, obs);
        }

        dct
    }

    pub fn get_words_leq_score(&self, score: u32) -> Box<[Word]> {
        let mut v = Vec::new();

        let stuff = self.obscurity_index.range(..=score);

        for (i, cell) in stuff {
            let set = cell.borrow_mut();
            for thing in set.iter() {
                v.push(self.words[*thing].clone());
            }
        }

        v.into_boxed_slice()
    }
}