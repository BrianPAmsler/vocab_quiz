use std::{
    collections::{
        hash_map::{Keys, Values},
        HashMap,
    },
    ops::Index,
    sync::Arc,
};

use crate::words::Dictionary;

pub struct DictMap {
    map: HashMap<String, Arc<Dictionary>>,
}

impl DictMap {
    pub fn new() -> DictMap {
        DictMap {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, dict: Arc<Dictionary>) -> Option<Arc<Dictionary>> {
        self.map.insert(dict.get_title().to_owned(), dict)
    }

    pub fn values<'a>(&'a self) -> Values<'a, String, Arc<Dictionary>> {
        self.map.values()
    }

    pub fn keys<'a>(&'a self) -> Keys<'a, String, Arc<Dictionary>> {
        self.map.keys()
    }

    pub fn get(&self, index: &String) -> Option<&Arc<Dictionary>> {
        self.map.get(index)
    }
}

impl Default for DictMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<&String> for DictMap {
    type Output = Arc<Dictionary>;

    fn index(&self, index: &String) -> &Self::Output {
        self.map.index(index)
    }
}

impl<'a> Index<&'a str> for DictMap {
    type Output = Arc<Dictionary>;

    fn index(&self, index: &'a str) -> &Self::Output {
        self.map.index(&index.to_owned())
    }
}

impl IntoIterator for DictMap {
    type IntoIter = <HashMap<String, Arc<Dictionary>> as IntoIterator>::IntoIter;
    type Item = <HashMap<String, Arc<Dictionary>> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a> IntoIterator for &'a DictMap {
    type IntoIter = <&'a HashMap<String, Arc<Dictionary>> as IntoIterator>::IntoIter;
    type Item = <&'a HashMap<String, Arc<Dictionary>> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
