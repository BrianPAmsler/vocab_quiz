#![cfg_attr(debug_assertions, allow(dead_code))] 

use std::{fs::File, collections::HashMap, rc::Rc, ops::Index};

use crate::{words::{Dictionary, Knowledge}, xml::parse_xml_dictionary, constants::LINEAR_MULTIPLIER};

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

    let mut dict_map = DictMap::new();
    dict_map.insert(jp_dict);

    let mut kw_file = File::open("misc/test.kw").unwrap();
    let kw = Knowledge::load_from(&mut kw_file, &dict_map).unwrap();

    println!("{}", kw.get_dict().get_title()); 
}
