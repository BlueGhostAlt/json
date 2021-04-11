use std::cmp;
use std::io;
use std::mem;
use std::str;

use super::{Error, Reader, Result};

const DEFAULT_BUF_READER_CAPACITY: usize = 2;

pub struct BufferedReader<R: io::Read> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,

    last_ch: Option<char>,
}

impl<R: io::Read> BufferedReader<R> {
    pub fn new(source: R) -> Result<Self> {
        BufferedReader::with_capacity(DEFAULT_BUF_READER_CAPACITY, source)
    }

    fn with_capacity(cap: usize, inner: R) -> Result<Self> {
        let mut buffer = Vec::with_capacity(cap * mem::size_of::<char>());
        unsafe {
            buffer.set_len(cap * mem::size_of::<char>());
            inner.initializer().initialize(&mut buffer);
        }

        let mut buf_reader = Self {
            inner,
            buf: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,

            last_ch: None,
        };
        buf_reader.fill_buf()?;

        Ok(buf_reader)
    }

    fn fill_buf(&mut self) -> Result<()> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf).map_err(Error::from)?;
            self.pos = 0;
        }

        let buf = &self.buf[self.pos..self.cap];
        let str = str::from_utf8(buf).map_err(Error::from)?;

        let mut chars = str.chars();
        self.last_ch = chars.next();

        Ok(())
    }
}

impl<R: io::Read> Reader for BufferedReader<R> {
    fn peek(&self) -> Option<char> {
        self.last_ch
    }

    fn consume(&mut self) -> Result<()> {
        if let Some(last_ch) = self.last_ch {
            self.pos = cmp::min(self.pos + last_ch.len_utf8(), self.cap);
            self.fill_buf()?;
        }

        Ok(())
    }
}

impl<R: io::Read> Iterator for BufferedReader<R> {
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
        let buf_reader = BufferedReader::new(io::empty())?;

        assert_eq!(buf_reader.peek(), None);

        Ok(())
    }

    #[test]
    fn test_peek() -> Result<()> {
        let buf_reader = BufferedReader::new(SOURCE)?;

        assert_eq!(buf_reader.peek(), Some('j'));
        assert_eq!(buf_reader.peek(), Some('j'));

        Ok(())
    }

    #[test]
    fn test_consume_empty() -> Result<()> {
        let mut buf_reader = BufferedReader::new(io::empty())?;

        assert_eq!(buf_reader.peek(), None);
        buf_reader.consume()?;
        assert_eq!(buf_reader.peek(), None);

        Ok(())
    }

    #[test]
    fn test_consume() -> Result<()> {
        let mut buf_reader = BufferedReader::new(SOURCE)?;

        assert_eq!(buf_reader.peek(), Some('j'));
        buf_reader.consume()?;
        assert_eq!(buf_reader.peek(), Some('s'));
        buf_reader.consume()?;
        assert_eq!(buf_reader.peek(), Some('o'));
        buf_reader.consume()?;
        assert_eq!(buf_reader.peek(), Some('n'));
        buf_reader.consume()?;
        assert_eq!(buf_reader.peek(), None);

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
