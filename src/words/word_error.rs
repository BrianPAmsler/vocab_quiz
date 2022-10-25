use std::fmt::Debug;

pub struct WordError {
    msg: String
}

impl Debug for WordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WordError: '{}'", self.msg)
    }
}

impl From<std::io::Error> for WordError {
    fn from(o: std::io::Error) -> Self {
        WordError { msg: o.to_string() }
    }
}

impl From<&'static str> for WordError {
    fn from(o: &'static str) -> Self {
        WordError { msg: o.to_string() }
    }
}

impl From<postcard::Error> for WordError {
    fn from(o: postcard::Error) -> Self {
        WordError { msg: o.to_string() }
    }
}