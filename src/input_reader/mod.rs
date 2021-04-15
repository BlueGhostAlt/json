use std::result;
use std::str;
use std::{error, fmt, io};

mod buffered_reader;
mod memory_reader;

pub use buffered_reader::BufferedReader;
pub use memory_reader::MemoryReader;

pub trait ReadInput {
    fn peek(&self) -> Option<char>;

    fn consume(&mut self) -> Result<()>;
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    #[allow(dead_code)]
    repr: Repr,
}

#[derive(Debug)]
enum Repr {
    Io(io::Error),
    Utf8(str::Utf8Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error {
            repr: Repr::Io(error),
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(error: str::Utf8Error) -> Error {
        Error {
            repr: Repr::Utf8(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Io(io_err) => write!(f, "{}", io_err),
            Repr::Utf8(utf8_err) => write!(f, "{}", utf8_err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match &self.repr {
            Repr::Io(io_err) => io_err,
            Repr::Utf8(utf8_err) => utf8_err,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use io::Empty;

    const EMPTY_SOURCE: Empty = io::empty();
    const SOURCE: &[u8] = "json".as_bytes();

    #[test]
    fn test_empty_readers_are_eq() -> Result<()> {
        let buf_reader = BufferedReader::new(EMPTY_SOURCE)?;
        let mem_reader = MemoryReader::new(EMPTY_SOURCE)?;

        assert!(buf_reader.eq(mem_reader));

        Ok(())
    }

    #[test]
    fn test_readers_are_eq() -> Result<()> {
        let buf_reader = BufferedReader::new(SOURCE)?;
        let mem_reader = MemoryReader::new(SOURCE)?;

        assert!(buf_reader.eq(mem_reader));

        Ok(())
    }
}
