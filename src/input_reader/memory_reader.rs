use std::cmp;
use std::io;
use std::iter;
use std::str;

use super::{Error, ReadInput, Result};

/// The `MemoryReader` struct provides in-memory whole input reading.
///
/// This input reader is meant to be used in situations such as performance
/// critical software, where memory consumption is not necessarily a concern
/// but where performance matters, and where any extra syscall or extra read
/// of the input is undesirable.
///
/// A `MemoryReader` reads the whole input in memory in a fixed-size
/// heap-allocated buffer. That means only one read call, but a potential
/// exhaustion of available memory.
///
/// # Examples
///
/// ```
/// use json::input_reader::{self, MemoryReader, ReadInput};
///
/// fn main() -> input_reader::Result<()> {
///     let mut reader = MemoryReader::new("json".as_bytes())?;
///
///     assert_eq!(reader.peek(), Some('j'));
///     reader.consume()?;
///     assert_eq!(reader.peek(), Some('s'));
///     reader.consume()?;
///     reader.consume()?;
///     reader.consume()?;
///     assert_eq!(reader.peek(), None);
///
///     Ok(())
/// }
/// ```
pub struct MemoryReader {
    buf: Box<[char]>,
    pos: usize,
}

impl MemoryReader {
    /// Creates a new `MemoryReader` by reading the whole input.
    ///
    /// # Examples
    ///
    /// ```
    /// use json::input_reader::{self, MemoryReader, ReadInput};
    ///
    /// fn main() -> input_reader::Result<()> {
    ///     let mut reader = MemoryReader::new("json".as_bytes())?;
    ///
    ///     Ok(())
    /// }
    /// ```
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

impl ReadInput for MemoryReader {
    fn peek(&self) -> Option<char> {
        self.buf.get(self.pos).copied()
    }

    fn consume(&mut self) -> Result<()> {
        self.pos = cmp::min(self.pos + 1, self.buf.len());

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.buf.len() - self.pos, Some(self.buf.len() - self.pos))
    }
}

unsafe impl iter::TrustedLen for MemoryReader {}

impl ExactSizeIterator for MemoryReader {}

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
