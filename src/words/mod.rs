mod dictionary;
mod word_error;
mod word;
mod knowledge;

pub use dictionary::*;
pub use word_error::*;
pub use word::*;
pub use knowledge::*;

const MAX_AWARD: u32 = 50;

type WordID = usize;