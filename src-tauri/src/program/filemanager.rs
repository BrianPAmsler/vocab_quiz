use std::{marker::PhantomData, io::{Read, Write}, mem::size_of, vec};

use serde::{Serialize, Deserialize};

use crate::{error::Error, tools::crypt_string::PermutedString};


#[derive(Clone, Serialize, Deserialize)]
pub struct AppFile {
    pub header: String,
    pub version: PermutedString,
    pub data: Box<[u8]>,
    _pd: PhantomData<()>
}

pub fn read_file<R: Read>(readable: &mut R) -> Result<AppFile, Error> {
    let mut data = Vec::new();
    readable.read_to_end(&mut data)?;

    let dict_data: AppFile = postcard::from_bytes(&data)?;

    Ok(dict_data)
}

pub fn save_file<W: Write>(writable: &mut W, header: String, version: String, data: Box<[u8]>) -> Result<usize, Error> {
    let size = size_of::<AppFile>() + data.len();

    let file = AppFile { header, version: version.into(), data, _pd: PhantomData };

    let mut out_data = vec![0u8; size];
    let out_data = postcard::to_slice(&file, &mut out_data)?;

    writable.write(out_data)?;

    Ok(out_data.len())
}