
use std::io::{Read, Write};

pub struct U8Buffer<'a> {
    buf: &'a mut [u8],
    begin: usize,
    end: usize
}

impl<'a> U8Buffer<'a> {
    pub fn create_empty<'b>(buf: &'b mut [u8]) -> U8Buffer<'b> {
        U8Buffer { buf, begin: 0, end: 0 }
    }

    pub fn create_full<'b>(buf: &'b mut [u8]) -> U8Buffer<'b> {
        U8Buffer { begin: 0, end: buf.len(), buf }
    }

    pub fn len(&self) -> usize {
        self.end - self.begin
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn get_data<'s>(&'s self) -> &'s [u8] {
        &self.buf[self.begin..self.end]
    }

    pub fn shift_to_front(&mut self) {
        if self.begin == 0 {
            return;
        }

        let size = self.len();
        for i in 0..size {
            self.buf[i] = self.buf[i + self.begin];
        }

        self.begin = 0;
        self.end = size;
    }

    pub fn force_len(&mut self, len: usize) -> usize {
        let prev = self.len();
        self.end = self.begin + len;

        prev
    }

    // TODO: add bounds checking to these methods
    pub fn advance_read(&mut self, amount: usize) {
        self.begin += amount;
    }

    pub fn reverse_read(&mut self, amount: usize) {
        self.begin -= amount;
    }

    pub fn advance_write(&mut self, amount: usize) {
        self.end += amount;
    }

    pub fn reverse_write(&mut self, amount: usize) {
        self.end -= amount;
    }

    pub fn debug(&self) {
        println!("U8Buffer {{ begin: {}, end: {} }}", self.begin, self.end);
    }
}

impl<'a> Write for U8Buffer<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf[self.end..][..buf.len()].copy_from_slice(buf);
        self.end += buf.len();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> Read for U8Buffer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf.copy_from_slice(&self.buf[self.begin..self.end][..buf.len()]);
        self.begin += buf.len();

        Ok(self.end)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        buf.reserve(self.len());
        buf.extend_from_slice(&self.buf[self.begin..self.end]);

        Ok(self.len())
    }
}