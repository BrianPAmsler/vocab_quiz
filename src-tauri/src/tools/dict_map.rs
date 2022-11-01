use std::{collections::HashMap, rc::Rc, ops::Index};

use crate::words::Dictionary;

pub struct DictMap {
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

impl Default for DictMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<String> for DictMap {
    type Output = Rc<Dictionary>;

    fn index(&self, index: String) -> &Self::Output {
        self.map.index(&index)
    }
}

impl<'a> Index<&'a str> for DictMap {
    type Output = Rc<Dictionary>;

    fn index(&self, index: &'a str) -> &Self::Output {
        self.map.index(&index.to_owned())
    }
}

impl IntoIterator for DictMap {
    type IntoIter = <HashMap::<String, Rc<Dictionary>> as IntoIterator>::IntoIter;
    type Item = <HashMap::<String, Rc<Dictionary>> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a> IntoIterator for &'a DictMap {
    type IntoIter = <&'a HashMap::<String, Rc<Dictionary>> as IntoIterator>::IntoIter;
    type Item = <&'a HashMap::<String, Rc<Dictionary>> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}