#![cfg_attr(debug_assertions, allow(dead_code))] 

use std::fs::File;

use crate::{words::{Dictionary, Knowledge}, xml::parse_xml_dictionary, constants::LINEAR_MULTIPLIER};

mod words;
mod xml;
mod constants;

fn main() {
    println!("Reading xml file...");
    let wordfile = File::open("misc/japanese.xml").unwrap();
    println!("Parsing xml...");
    let jp_dict = parse_xml_dictionary(wordfile, words::ObscurityMode::Linear(LINEAR_MULTIPLIER)).unwrap();

    let f = File::create("japanese.dct").unwrap();

    println!("Saving jp_dict...");
    jp_dict.save_to(f).unwrap();

    println!("Loading jp_dict...");
    let f = File::open("japanese.dct").unwrap();
    let jp_dict = Dictionary::load_from(f).unwrap();

    println!("Seraching words...");
    let words = jp_dict.get_words_leq_score(5);

    println!("First 5 words: {:?}", words);
    println!("Dictionary title: \"{}\"", jp_dict.get_title());

    let k = Knowledge::create(&jp_dict);
    let f = File::create("test.kw").unwrap();
    let k = k.save_to(f).unwrap();

    k.test();
}
