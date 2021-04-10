use std::io;
use std::str;

use super::{Error, Reader, Result};

pub struct MemoryReader {
    buf: Box<[char]>,
    pos: usize,
}

impl MemoryReader {
    pub fn new<R>(mut source: R) -> Result<Self>
    where
        R: io::Read,
    {
        let mut buffer = Vec::new();
        source
            .read_to_end(&mut buffer)
            .map_err(|e| Error::from(e))?;
        let buffer = str::from_utf8(&buffer).map_err(|e| Error::from(e))?;
        let buffer = buffer.chars().collect::<Vec<_>>();

        Ok(Self {
            buf: buffer.into_boxed_slice(),
            pos: 0,
        })
    }
}

impl Reader for MemoryReader {
    fn peek(&mut self, at: usize) -> Option<char> {
        self.buf.get(self.pos + at).copied()
    }

    fn consume(&mut self, amt: usize) {
        self.pos += amt;
    }
}

impl Iterator for MemoryReader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.peek(0)?;
        self.consume(1);

        Some(c)
    }
}
