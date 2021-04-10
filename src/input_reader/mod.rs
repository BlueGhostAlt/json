use std::io;
use std::result;
use std::str;

mod buffered_reader;
mod memory_reader;

pub use buffered_reader::BufferedReader;
pub use memory_reader::MemoryReader;

pub(crate) trait Reader {
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
