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
        source.read_to_end(&mut buffer).map_err(Error::from)?;
        let buffer = str::from_utf8(&buffer).map_err(Error::from)?;
        let buffer = buffer.chars().collect::<Vec<_>>();

        Ok(Self {
            buf: buffer.into_boxed_slice(),
            pos: 0,
        })
    }
}

impl Reader for MemoryReader {
    fn peek(&self) -> Option<char> {
        self.buf.get(self.pos).copied()
    }

    fn consume(&mut self) -> Result<()> {
        self.pos += 1;

        Ok(())
    }
}

impl Iterator for MemoryReader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.peek()?;
        self.consume().ok()?;

        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &[u8] = "json".as_bytes();

    #[test]
    fn test_peek_empty() -> Result<()> {
        let mem_reader = MemoryReader::new(io::empty())?;

        assert_eq!(mem_reader.peek(), None);

        Ok(())
    }

    #[test]
    fn test_peek() -> Result<()> {
        let mem_reader = MemoryReader::new(SOURCE)?;

        assert_eq!(mem_reader.peek(), Some('j'));
        assert_eq!(mem_reader.peek(), Some('j'));

        Ok(())
    }

    #[test]
    fn test_consume_empty() -> Result<()> {
        let mut mem_reader = MemoryReader::new(io::empty())?;

        assert_eq!(mem_reader.peek(), None);
        mem_reader.consume()?;
        assert_eq!(mem_reader.peek(), None);

        Ok(())
    }

    #[test]
    fn test_consume() -> Result<()> {
        let mut mem_reader = MemoryReader::new(SOURCE)?;

        assert_eq!(mem_reader.peek(), Some('j'));
        mem_reader.consume()?;
        assert_eq!(mem_reader.peek(), Some('s'));
        mem_reader.consume()?;
        assert_eq!(mem_reader.peek(), Some('o'));
        mem_reader.consume()?;
        assert_eq!(mem_reader.peek(), Some('n'));
        mem_reader.consume()?;
        assert_eq!(mem_reader.peek(), None);

        Ok(())
    }

    #[test]
    fn test_next() -> Result<()> {
        let mut mem_reader = MemoryReader::new(SOURCE)?;

        assert_eq!(mem_reader.next(), Some('j'));
        assert_eq!(mem_reader.next(), Some('s'));
        assert_eq!(mem_reader.next(), Some('o'));
        assert_eq!(mem_reader.next(), Some('n'));
        assert_eq!(mem_reader.next(), None);

        Ok(())
    }
}
