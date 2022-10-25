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

    pub fn insert(&mut self, k: String, dict: Rc<Dictionary>) -> Option<Rc<Dictionary>> {
        self.map.insert(k, dict)
    }
}

impl Index<String> for DictMap {
    type Output = Rc<Dictionary>;

    fn index(&self, index: String) -> &Self::Output {
        self.map.index(&index)
    }
}

fn main() {
    println!("Reading xml file...");
    let wordfile = File::open("misc/japanese.xml").unwrap();
    println!("Parsing xml...");
    let jp_dict = parse_xml_dictionary(wordfile, words::ObscurityMode::Linear(LINEAR_MULTIPLIER)).unwrap();

    let mut f = File::create("misc/japanese.dct").unwrap();

    println!("Saving jp_dict...");
    jp_dict.save_to(&mut f).unwrap();

    println!("Loading jp_dict...");
    let mut f = File::open("misc/japanese.dct").unwrap();
    let jp_dict = Dictionary::load_from(&mut f).unwrap();

    println!("Seraching words...");
    let words = jp_dict.get_words_leq_score(5);

    println!("First 5 words: {:?}", words);
    println!("Dictionary title: \"{}\"", jp_dict.get_title());

    let mut map = DictMap::new();
    let jp_title = jp_dict.get_title().to_owned();
    map.insert(jp_title.to_owned(), Rc::new(jp_dict));

    let k = Knowledge::create(map[jp_title].clone());
    let mut f = File::create("misc/test.kw").unwrap();
    k.save_to(&mut f).unwrap();

    let mut f = File::open("misc/test.kw").unwrap();
    let k = Knowledge::load_from(&mut f, &map).unwrap();

    let d = k.get_dict();
    println!("dict name: {}", d.get_title());
}
