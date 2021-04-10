use std::cmp;
use std::io;
use std::mem;
use std::str;

use super::Reader;

const DEFAULT_BUF_READER_CAPACITY: usize = 2;

pub struct BufferedReader<R: io::Read> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl<R: io::Read> BufferedReader<R> {
    pub fn new(source: R) -> Self {
        BufferedReader::with_capacity(DEFAULT_BUF_READER_CAPACITY, source)
    }

    fn with_capacity(cap: usize, inner: R) -> Self {
        let mut buffer = Vec::with_capacity(cap * mem::size_of::<char>());
        unsafe {
            buffer.set_len(cap * mem::size_of::<char>());
            inner.initializer().initialize(&mut buffer);
        }

        Self {
            inner,
            buf: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,
        }
    }

    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }

        Ok(&self.buf[self.pos..self.cap])
    }
}

impl<R: io::Read> Reader for BufferedReader<R> {
    fn peek(&mut self, at: usize) -> Option<char> {
        let buf = self.fill_buf().ok()?;
        let str = str::from_utf8(buf).ok()?;
        let mut chars = str.chars();

        chars.nth(at)
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}

impl<R: io::Read> Iterator for BufferedReader<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.peek(0)?;
        self.consume(c.len_utf8());

        Some(c)
    }
}
