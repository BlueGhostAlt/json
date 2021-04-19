use std::cmp;
use std::io;
use std::iter;
use std::mem;
use std::str;

use super::{Error, ReadInput, Result};

const DEFAULT_BUF_READER_CAPACITY: usize = 16;

/// The `BufferedReader<R>` struct provides in-memory buffered input reading.
///
/// This input reader is meant to be used in situations such as with big data,
/// where memory can be exhausted quickly if the whole input where to be read
/// at once.
///
/// A `BufferedReader<R>` buffers a part of the input in memory in a fixed-size
/// heap-allocated buffer. Though, that means multiple read calls, which might
/// be unaffordable in performance critical operations.
///
/// # Examples
///
/// ```
/// use json::input_reader::{self, BufferedReader, ReadInput};
///
/// fn main() -> input_reader::Result<()> {
///     let mut reader = BufferedReader::new("json".as_bytes())?;
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
pub struct BufferedReader<R: io::Read> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,

    chars: [Option<char>; DEFAULT_BUF_READER_CAPACITY],
}

impl<R: io::Read> BufferedReader<R> {
    /// Creates a new `BuffferedReader<R>` with a default buffer capacity. The
    /// default is currently 68 bytes, allowing for peeking 16 characters, but
    /// may change in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// use json::input_reader::{self, BufferedReader, ReadInput};
    ///
    /// fn main() -> input_reader::Result<()> {
    ///     let mut reader = BufferedReader::new("json".as_bytes())?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(source: R) -> Result<Self> {
        BufferedReader::with_capacity(DEFAULT_BUF_READER_CAPACITY, source)
    }

    fn with_capacity(cap: usize, inner: R) -> Result<Self> {
        let mut buffer = Vec::with_capacity((cap + 1) * mem::size_of::<char>());
        unsafe {
            buffer.set_len(cap * mem::size_of::<char>());
            inner.initializer().initialize(&mut buffer);
        }

        let mut buf_reader = Self {
            inner,
            buf: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,

            chars: [None; DEFAULT_BUF_READER_CAPACITY],
        };
        buf_reader.fill_buf()?;

        Ok(buf_reader)
    }

    fn fill_buf(&mut self) -> Result<()> {
        // Branch using `>=` instead of the more correct `==` to tell the
        // compiler that the pos..cap slice is always valid.
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf).map_err(Error::from)?;
            self.pos = 0;
        }

        let buf = &self.buf[self.pos..self.cap];
        let str = str::from_utf8(buf).map_err(Error::from)?;

        let mut chars = str.chars();
        self.chars.iter_mut().for_each(|c| *c = chars.next());

        Ok(())
    }
}

impl<R: io::Read> ReadInput for BufferedReader<R> {
    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(k).copied().flatten()
    }

    fn consume(&mut self, k: usize) -> Result<()> {
        let len = self
            .chars
            .iter()
            .take(k)
            .filter_map(|c| c.map(|c| c.len_utf8()))
            .sum::<usize>();
        self.pos = cmp::min(self.pos + len, self.cap);
        self.fill_buf()?;

        Ok(())
    }
}

impl<R: io::Read> Iterator for BufferedReader<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.peek(0)?;
        self.consume(1).ok()?;

        Some(c)
    }
}

impl<R: io::Read> iter::FusedIterator for BufferedReader<R> {}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &[u8] = "json".as_bytes();

    #[test]
    fn test_peek_empty() -> Result<()> {
        let buf_reader = BufferedReader::new(io::empty())?;

        assert_eq!(buf_reader.peek(0), None);

        Ok(())
    }

    #[test]
    fn test_peek() -> Result<()> {
        let buf_reader = BufferedReader::new(SOURCE)?;

        assert_eq!(buf_reader.peek(0), Some('j'));
        assert_eq!(buf_reader.peek(0), Some('j'));
        assert_eq!(buf_reader.peek(1), Some('s'));
        assert_eq!(buf_reader.peek(2), Some('o'));
        assert_eq!(buf_reader.peek(3), Some('n'));
        assert_eq!(buf_reader.peek(4), None);

        Ok(())
    }

    #[test]
    fn test_consume_empty() -> Result<()> {
        let mut buf_reader = BufferedReader::new(io::empty())?;

        assert_eq!(buf_reader.peek(0), None);
        buf_reader.consume(1)?;
        assert_eq!(buf_reader.peek(0), None);

        Ok(())
    }

    #[test]
    fn test_consume() -> Result<()> {
        let mut buf_reader = BufferedReader::new(SOURCE)?;

        assert_eq!(buf_reader.peek(0), Some('j'));
        assert_eq!(buf_reader.peek(1), Some('s'));
        buf_reader.consume(1)?;
        assert_eq!(buf_reader.peek(0), Some('s'));
        buf_reader.consume(0)?;
        assert_eq!(buf_reader.peek(1), Some('o'));
        buf_reader.consume(1)?;
        assert_eq!(buf_reader.peek(1), Some('n'));
        buf_reader.consume(1)?;
        assert_eq!(buf_reader.peek(0), Some('n'));
        assert_eq!(buf_reader.peek(1), None);
        buf_reader.consume(1)?;
        assert_eq!(buf_reader.peek(0), None);

        Ok(())
    }

    #[test]
    fn test_next() -> Result<()> {
        let mut buf_reader = BufferedReader::new(SOURCE)?;

        assert_eq!(buf_reader.next(), Some('j'));
        assert_eq!(buf_reader.next(), Some('s'));
        assert_eq!(buf_reader.next(), Some('o'));
        assert_eq!(buf_reader.next(), Some('n'));
        assert_eq!(buf_reader.next(), None);

        Ok(())
    }
}
