#![cfg_attr(debug_assertions, allow(dead_code))] 

use std::{fs::File, collections::HashMap, rc::Rc, ops::Index};

use chrono::Utc;

use crate::{words::{Dictionary, Knowledge, WordKnowledge}, xml::parse_xml_dictionary, constants::LINEAR_MULTIPLIER};

mod words;
mod xml;
mod constants;

struct DictMap {
    map: HashMap<String, Rc<Dictionary>>
}

impl DictMap {
    pub fn new() -> DictMap {
        DictMap { map: HashMap::new() }
    }

    pub fn insert(&mut self, dict: Rc<Dictionary>) -> Option<Rc<Dictionary>> {
        self.map.insert(dict.get_title().to_owned(), dict)
    }
}

impl Index<String> for DictMap {
    type Output = Rc<Dictionary>;

    fn index(&self, index: String) -> &Self::Output {
        self.map.index(&index)
    }
}

fn main() {
    let mut jp_file = File::open("misc/japanese.dct").unwrap();
    let jp_dict = Rc::new(Dictionary::load_from(&mut jp_file).unwrap());
    drop(jp_file);

    let mut dict_map = DictMap::new();
    dict_map.insert(jp_dict.clone());

    let mut kw_file = File::open("misc/test.kw").unwrap();
    let mut kw = Knowledge::load_from(&mut kw_file, &dict_map).unwrap();
    drop(kw_file);

    let word = jp_dict.find_word("お金".to_owned()).unwrap();
    //kw.practice(word, 5);

    let wk = kw.get_word_knowledge(word);

    let time = wk.last_practice.unwrap().time();
    let delta = Utc::now() - time;
    let m = delta.num_seconds() / 60;
    let s = delta.num_seconds() % 60;
    println!("Time since last practice: {} minutes and {} seconds", m, s);

    let mut f = File::create("misc/test.kw").unwrap();
    kw.save_to(&mut f).unwrap();
}
