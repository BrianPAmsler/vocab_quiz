use crate::words::{Word, Dictionary};

mod words;

fn main() {
    let test = ["a", "asdf","b", "c", "e", "f", "g"];

    let mut words = Vec::new();
    words.reserve(test.len());

    for w in test {
        let word = Word { text: w.to_string(), definition: "".to_string(), pronunciation: None, obscurity: ((w.chars().nth(0).unwrap() as u32) - ('a' as u32) + 1) };

        words.push(word);
    }

    let dict = Dictionary::create(words.into_boxed_slice(), words::ObscurityMode::Manual);
    println!("dict: {:?}", dict);

    let words = dict.get_words_leq_score(400);
    print!("words: {:?}", words);
}
