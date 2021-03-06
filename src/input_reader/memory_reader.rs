use std::{cmp, io, str};

use super::{ReadInput, Result};

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
///     assert_eq!(reader.peek(0), Some('j'));
///     reader.consume(1)?;
///     assert_eq!(reader.peek(0), Some('s'));
///     reader.consume(0)?;
///     reader.consume(0)?;
///     reader.consume(3)?;
///     assert_eq!(reader.peek(0), None);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct MemoryReader {
    buf: Box<[char]>,
    pos: usize,
}

impl MemoryReader {
    /// Creates a new `MemoryReader` by reading the whole input.
    ///
    /// # Errors
    ///
    /// This associated function can fail only if the provided input is not
    /// valid UTF-8.
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
        source.read_to_end(&mut buffer)?;
        let buffer = str::from_utf8(&buffer)?;
        let buffer = buffer.chars().collect::<Vec<_>>();

        Ok(Self {
            buf: buffer.into_boxed_slice(),
            pos: 0,
        })
    }
}

impl ReadInput for MemoryReader {
    fn peek(&self, k: usize) -> Option<char> {
        self.buf.get(self.pos + k).copied()
    }

    fn consume(&mut self, k: usize) -> Result<()> {
        self.pos = cmp::min(self.pos + k, self.buf.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &[u8] = "json".as_bytes();

    #[test]
    fn test_peek_empty() -> Result<()> {
        let mem_reader = MemoryReader::new(io::empty())?;

        assert_eq!(mem_reader.peek(0), None);

        Ok(())
    }

    #[test]
    fn test_peek() -> Result<()> {
        let mem_reader = MemoryReader::new(SOURCE)?;

        assert_eq!(mem_reader.peek(0), Some('j'));
        assert_eq!(mem_reader.peek(0), Some('j'));
        assert_eq!(mem_reader.peek(1), Some('s'));
        assert_eq!(mem_reader.peek(2), Some('o'));
        assert_eq!(mem_reader.peek(3), Some('n'));
        assert_eq!(mem_reader.peek(4), None);

        Ok(())
    }

    #[test]
    fn test_consume_empty() -> Result<()> {
        let mut mem_reader = MemoryReader::new(io::empty())?;

        assert_eq!(mem_reader.peek(0), None);
        mem_reader.consume(1)?;
        assert_eq!(mem_reader.peek(0), None);

        Ok(())
    }

    #[test]
    fn test_consume() -> Result<()> {
        let mut mem_reader = MemoryReader::new(SOURCE)?;

        assert_eq!(mem_reader.peek(0), Some('j'));
        assert_eq!(mem_reader.peek(1), Some('s'));
        mem_reader.consume(1)?;
        assert_eq!(mem_reader.peek(0), Some('s'));
        mem_reader.consume(0)?;
        assert_eq!(mem_reader.peek(1), Some('o'));
        mem_reader.consume(1)?;
        assert_eq!(mem_reader.peek(1), Some('n'));
        mem_reader.consume(1)?;
        assert_eq!(mem_reader.peek(0), Some('n'));
        assert_eq!(mem_reader.peek(1), None);
        mem_reader.consume(1)?;
        assert_eq!(mem_reader.peek(0), None);

        Ok(())
    }

    #[test]
    fn test_next() -> Result<()> {
        let mut mem_reader = MemoryReader::new(SOURCE)?;
        let mut input_reader = mem_reader.input_reader();

        assert_eq!(input_reader.next(), Some('j'));
        assert_eq!(input_reader.next(), Some('s'));
        assert_eq!(input_reader.next(), Some('o'));
        assert_eq!(input_reader.next(), Some('n'));
        assert_eq!(input_reader.next(), None);

        Ok(())
    }
}
