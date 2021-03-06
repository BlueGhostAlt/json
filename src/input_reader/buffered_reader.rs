use std::{cmp, io, mem, str};

use super::{Error, ReadInput, Result};

pub const BUF_READER_CAPACITY: usize = 16;

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
#[derive(Debug)]
pub struct BufferedReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,

    chars: [Option<char>; BUF_READER_CAPACITY],
}

impl<R: io::Read> BufferedReader<R> {
    /// Creates a new `BuffferedReader<R>` with a default buffer capacity. The
    /// default is currently 68 bytes, allowing for peeking 16 characters, but
    /// may change in the future.
    ///
    /// # Errors
    ///
    /// This function can fail only if it doesn't manage to fill the internal
    /// buffer. For more details see the documentation for
    /// [`ReadInput::consume`].
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
        Self::with_capacity(BUF_READER_CAPACITY, source)
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

            chars: [None; BUF_READER_CAPACITY],
        };
        buf_reader.fill_buf()?;

        Ok(buf_reader)
    }

    fn fill_buf(&mut self) -> Result<()> {
        // Branch using `>=` instead of the more correct `==` to tell the
        // compiler that the pos..cap slice is always valid.
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }

        let buf = &self.buf[self.pos..self.cap];
        let str = str::from_utf8(buf)?;

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
        // TODO: Write tests for erroring on buffer overconsumption
        if k > self.chars.len() {
            return Err(Error::overconsume_buffer(k));
        }

        let len = self
            .chars
            .iter()
            .take(k)
            .filter_map(|c| c.map(char::len_utf8))
            .sum::<usize>();
        self.pos = cmp::min(self.pos + len, self.cap);
        self.fill_buf()?;

        Ok(())
    }
}

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
        let mut input_reader = buf_reader.input_reader();

        assert_eq!(input_reader.next(), Some('j'));
        assert_eq!(input_reader.next(), Some('s'));
        assert_eq!(input_reader.next(), Some('o'));
        assert_eq!(input_reader.next(), Some('n'));
        assert_eq!(input_reader.next(), None);

        Ok(())
    }
}
