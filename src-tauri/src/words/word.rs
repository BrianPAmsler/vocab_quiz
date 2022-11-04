use serde::{Serialize, Deserialize, Serializer, Deserializer};

use crate::tools::crypt_string::EncryptedString;

fn serialize_as_ecrypted<S: Serializer>(string: &String, s: S) -> Result<S::Ok, S::Error> {
    let enc: EncryptedString = string.to_owned().into();
    enc.serialize(s)
}

fn deserialize_as_ecrypted<'de, D: Deserializer<'de>>(s: D) -> Result<String, D::Error> {
    let enc = EncryptedString::deserialize(s)?;
    Ok(enc.to_string())
}

fn serialize_as_ecrypted_op<S: Serializer>(string: &Option<String>, s: S) -> Result<S::Ok, S::Error> {
    let enc: Option<EncryptedString> = match string {
        Some(s) => Some(s.to_owned().into()),
        None => None
    };

    enc.serialize(s)
}

fn deserialize_as_ecrypted_op<'de, D: Deserializer<'de>>(s: D) -> Result<Option<String>, D::Error> {
    let enc = Option::<EncryptedString>::deserialize(s)?;

    match enc {
        Some(es) => Ok(Some(es.to_string())),
        None => Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    #[serde(serialize_with = "serialize_as_ecrypted", deserialize_with = "deserialize_as_ecrypted")]
    pub text: String,
    #[serde(serialize_with = "serialize_as_ecrypted", deserialize_with = "deserialize_as_ecrypted")]
    pub definition: String,
    #[serde(serialize_with = "serialize_as_ecrypted_op", deserialize_with = "deserialize_as_ecrypted_op")]
    pub pronunciation: Option<String>,
    pub obscurity: u32
}

pub mod for_frontend {
    
    #[derive(Debug, Clone, super::Serialize, super::Deserialize)]
    pub struct Word {
        pub text: String,
        pub definition: String,
        pub pronunciation: Option<String>,
        pub obscurity: u32
    }

    impl From<super::Word> for Word {
        fn from(value: super::Word) -> Self {
            Word { text: value.text, definition: value.definition, pronunciation: value.pronunciation, obscurity: value.obscurity }
        }
    }
}