use std::fmt::Debug;

pub struct WordError (String);

impl<'a> WordError {
    pub fn msg<'b>(&'b self) -> &'b str {
        &self.0
    }

    pub fn take_msg(self) -> String {
        self.0
    }
}

impl Debug for WordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WordError: '{}'", self.0)
    }
}

impl From<std::io::Error> for WordError {
    fn from(o: std::io::Error) -> Self {
        WordError (o.to_string())
    }
}

impl From<&'static str> for WordError {
    fn from(o: &'static str) -> Self {
        WordError (o.to_string())
    }
}

impl From<postcard::Error> for WordError {
    fn from(o: postcard::Error) -> Self {
        WordError (o.to_string())
    }
}