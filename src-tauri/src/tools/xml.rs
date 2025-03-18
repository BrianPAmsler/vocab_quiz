use std::num::ParseIntError;

use xmltree::Element;
use xmltree::ParseError;

use crate::words::Dictionary;
use crate::words::ObscurityMode;
use crate::words::Word;

#[derive(Debug)]
pub enum DictParseError {
    XMLParseError(ParseError),
    InvalidXMLStructure,
    InvalidXMLData,
}

impl From<ParseError> for DictParseError {
    fn from(error: ParseError) -> Self {
        DictParseError::XMLParseError(error)
    }
}

impl From<ParseIntError> for DictParseError {
    fn from(_: ParseIntError) -> Self {
        DictParseError::InvalidXMLData
    }
}

pub fn parse_xml_dictionary<T: std::io::Read>(
    readable: T,
    mode: ObscurityMode,
) -> Result<Dictionary, DictParseError> {
    let mut out = Vec::new();

    let root = Element::parse(readable)?;

    if root.name != "dictionary" {
        return Err(DictParseError::InvalidXMLStructure);
    }

    let title = match root.get_child("title") {
        Some(e) => match e.get_text() {
            Some(t) => t.to_string(),
            None => return Err(DictParseError::InvalidXMLData),
        },
        None => return Err(DictParseError::InvalidXMLStructure),
    };

    let words = match root.get_child("words") {
        Some(e) => e,
        None => return Err(DictParseError::InvalidXMLStructure),
    };

    for child in &words.children {
        let e = match child.as_element() {
            Some(e) => e,
            None => return Err(DictParseError::InvalidXMLStructure),
        };

        let text = match e.get_child("text") {
            Some(e) => match e.get_text() {
                Some(text) => text.to_string(),
                None => return Err(DictParseError::InvalidXMLData),
            },
            None => return Err(DictParseError::InvalidXMLStructure),
        };

        let definition = match e.get_child("definition") {
            Some(e) => match e.get_text() {
                Some(text) => text.to_string(),
                None => return Err(DictParseError::InvalidXMLData),
            },
            None => return Err(DictParseError::InvalidXMLStructure),
        };

        let pronunciation = match e.get_child("pronunciation") {
            Some(e) => match e.get_text() {
                Some(text) => Some(text.to_string()),
                None => None,
            },
            None => None,
        };

        let obscurity = match e.get_child("obscurity") {
            Some(e) => match e.get_text() {
                Some(text) => text.to_string().parse::<u32>()?,
                None => 0u32,
            },
            None => 0u32,
        };

        let wstruct = Word {
            text,
            pronunciation,
            definition,
            obscurity,
        };

        out.push(wstruct);
    }

    Ok(Dictionary::create(out.into_boxed_slice(), title, mode))
}
