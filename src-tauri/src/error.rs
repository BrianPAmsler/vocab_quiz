use std::fmt::{Debug, Display};

use crate::tools::xml::DictParseError;

#[derive(thiserror::Error)]
pub struct Error(String);

impl<'a> Error {
    pub fn msg<'b>(&'b self) -> &'b str {
        &self.0
    }

    pub fn take_msg(self) -> String {
        self.0
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WordError: '{}'", self.0)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "test")
    }
}

impl From<&'static str> for Error {
    fn from(o: &'static str) -> Self {
        Error(o.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(e.to_string())
    }
}

impl From<postcard::Error> for Error {
    fn from(o: postcard::Error) -> Self {
        Error(o.to_string())
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.0
    }
}

impl From<DictParseError> for Error {
    fn from(value: DictParseError) -> Self {
        let s = format!("{:?}", value);
        Error(s)
    }
}
