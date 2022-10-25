use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub definition: String,
    pub pronunciation: Option<String>,
    pub obscurity: u32
}