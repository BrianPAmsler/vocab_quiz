use std::{io::{Write, Read}, mem::size_of, rc::Rc, ops::Index};

use serde::{Serialize, Deserialize};

use crate::{words::{Knowledge, Dictionary}, tools::U8Buffer, error::Error};

pub struct User {
    name: String,
    knowledge: Vec<Knowledge>
}

impl User {
    pub fn create(name: String) -> User {
        User { name, knowledge: Vec::new() }
    }

    pub fn add_knowledge(&mut self, knowledge: Knowledge) {
        self.knowledge.push(knowledge);
    }

    pub fn take_knowledge(&mut self, knowledge: &Knowledge) -> Option<Knowledge> {
        for i in 0..self.knowledge.len() {
            let kw = &self.knowledge[i] as *const Knowledge;

            if kw == knowledge as *const Knowledge {
                return Some(self.knowledge.remove(i));
            }
        }

        None
    }

    pub fn get_knowledge(&self) -> &[Knowledge] {
        &self.knowledge
    }

    pub fn get_name<'u>(&'u self) -> &'u str {
        &self.name
    }

    pub fn save_to<T: Write>(self, writable: &mut T) -> Result<Self, Error> {
        let data = UserData::create(&self)?;

        let size_estimate = size_of::<UserData>() + size_of::<u8>() * data.knowledge_data.len();
    
        let mut alloc = vec![0u8; size_estimate];
        
        let out = postcard::to_slice(&data, &mut alloc)?;
        
        writable.write(out)?;
        
        Ok(self)
    }

    pub fn load_from<T: Read, I: Index<String, Output = Rc<Dictionary>>>(readable: &mut T, dict_container: &I) -> Result<Self, Error> {
        let mut bytes = Vec::new();
        readable.read_to_end(&mut bytes)?;

        let mut data = postcard::from_bytes::<UserData>(&mut bytes)?;

        let kw_data = decode_knowledge_data(&mut data.knowledge_data, dict_container)?;

        Ok(User { name: data.name, knowledge: kw_data.into_vec() })
    }
}

fn encode_knowledge_data(knowledge: &[Knowledge]) -> Result<Box<[u8]>, Error> {
    let mut size_estimate = size_of::<usize>();

    for kw in knowledge {
        size_estimate += kw.estimate_serialized_size();
    }

    let mut data = vec![0u8; size_estimate];
    println!("Estimated size: {}", size_estimate);

    let size_len = postcard::to_slice(&knowledge.len(), &mut data[..])?.len();

    let mut buf = U8Buffer::create_empty(&mut data[size_len..]);
    for k in knowledge {
        buf.advance_write(size_of::<u64>());
        let kw_size = k.save_to(&mut buf)?;
        buf.reverse_write(size_of::<u64>() + kw_size);

        let size_bytes = (kw_size as u64).to_ne_bytes();
        buf.write(&size_bytes)?;
        buf.advance_write(kw_size);
    }
    
    let total_size = buf.len() + size_len;
    drop(buf);

    data.truncate(total_size);
    println!("Actual size: {}", data.len());

    Ok(data.into_boxed_slice()) 
}

fn decode_knowledge_data<I: Index<String, Output = Rc<Dictionary>>>(data: &mut [u8], container: &I) -> Result<Box<[Knowledge]>, Error> {
    let (count, used) = {let t = postcard::take_from_bytes::<usize>(data)?; (t.0, data.len() - t.1.len())};
    let data = &mut data[used..];

    let mut out = Vec::new();
    out.reserve(count);
    let mut buf = U8Buffer::create_full(data);
    for _ in 0..count {
        let mut size_bytes = [0u8; size_of::<u64>()];
        buf.read(&mut size_bytes)?;
        let kw_size = u64::from_ne_bytes(size_bytes) as usize;

        let og_len = buf.force_len(kw_size);
        let k = Knowledge::load_from(&mut buf, container)?;
        buf.force_len(og_len);

        buf.advance_read(kw_size);
        out.push(k);
    }

    Ok(out.into_boxed_slice())
}

#[derive(Serialize, Deserialize)]
struct UserData {
    name: String,
    knowledge_data: Box<[u8]>
}

impl UserData {
    pub fn create(user: &User) -> Result<UserData, Error> {
        let name = user.name.to_owned();
        let knowledge_data = encode_knowledge_data(&user.knowledge)?;

        Ok(UserData { name, knowledge_data})
    }
}