use serde::{Serialize, de::Visitor, Deserialize};


fn encrypt_string(string: String) -> Box<[u8]> {
    let mut bytes = string.into_bytes().into_boxed_slice();

    let mut overflow = 0u8;
    for c in bytes.iter_mut().rev() {
        let _c = !(*c);
        let shift = (_c << 4) | overflow;
        overflow = _c >> 4;

        *c = shift;
    }

    let last = &mut bytes[bytes.len() - 1];
    *last = *last | overflow;

    bytes
}

fn decrypt_string(mut string: Box<[u8]>) -> String {
    let mut overflow = 0u8;
    for c in string.iter_mut() {
        let _c = !(*c);
        let shift = (_c >> 4) | overflow;
        overflow = _c << 4;

        *c = shift;
    }

    let first = &mut string[0];
    *first = *first | overflow;
    
    unsafe {String::from_utf8_unchecked(string.to_vec())}
}

#[derive(Debug, Clone)]
pub struct PermutedString {
    string: String
}

impl PermutedString {
    pub fn to_string(self) -> String {
        self.string
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }
}

impl From<String> for PermutedString {
    fn from(string: String) -> Self {
        Self {string}
    }
}

impl Serialize for PermutedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            let bytes = encrypt_string(self.string.to_owned());
            serializer.serialize_bytes(&bytes)
    }
}
struct EStrVisitor;

impl<'de> Visitor<'de> for EStrVisitor {
    type Value = PermutedString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("bytes representing an encrypted string.")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let string = decrypt_string(v.to_owned().into_boxed_slice());
        Ok(PermutedString {string})
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let string = decrypt_string(v.into_boxed_slice());
        Ok(PermutedString { string })
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
            let string = decrypt_string(v.to_owned().into_boxed_slice());
            Ok(PermutedString {string})
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut bytes = Vec::new();
        match seq.size_hint() {
            Some(size) => {
                bytes.reserve(size);
                for _ in 0..size {
                    bytes.push(seq.next_element::<u8>()?.unwrap());
                }
            },
            None => {
                let mut prev = seq.next_element::<u8>()?;
                while prev.is_some() {
                    bytes.push(prev.unwrap());

                    prev = seq.next_element()?;
                }
            }
        }

        self.visit_byte_buf(bytes)
    }
}

impl<'de> Deserialize<'de> for PermutedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_bytes(EStrVisitor)
    }
}