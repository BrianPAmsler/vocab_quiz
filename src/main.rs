#![cfg_attr(debug_assertions, allow(dead_code))] 

use crate::program::User;
use std::{fs::File, collections::HashMap, rc::Rc, ops::Index};

use crate::{words::{Dictionary, Knowledge}};

mod words;
mod xml;
mod constants;
mod program;
mod tools;

fn gen_usr(dicts: &DictMap, filename: &str) -> User {
    let jp_dict = &dicts["10,000 Most Common Japanese Words (kanjibox.net)"];

    let mut kw = Knowledge::create(jp_dict.clone());

    kw.randomize();

    let mut user = User::create("guy".to_owned());
    user.add_knowledge(kw);

    let mut f = File::create(filename).unwrap();
    user.save_to(&mut f).unwrap()
}

fn load_usr(dicts: &DictMap, filename: &str) -> User {
    let mut f = File::open(filename).unwrap();

    User::load_from(&mut f, dicts).unwrap()
}

fn main() {
    let mut jp_file = File::open("misc/japanese.dct").unwrap();
    let jp_dict = Rc::new(Dictionary::load_from(&mut jp_file).unwrap());

    let mut dict_map = DictMap::new();
    dict_map.insert(jp_dict.clone());
}
