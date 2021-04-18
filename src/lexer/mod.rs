use std::error;
use std::fmt;
use std::result;

mod lexer;

pub use crate::input_reader;
pub use lexer::{Lexer, Token};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    #[allow(dead_code)]
    repr: Repr,
}

#[derive(Debug)]
enum Repr {
    InputReader(input_reader::Error),
}

impl From<input_reader::Error> for Error {
    fn from(error: input_reader::Error) -> Error {
        Error {
            repr: Repr::InputReader(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::InputReader(input_reader_err) => write!(f, "{}", input_reader_err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match &self.repr {
            Repr::InputReader(input_reader_err) => input_reader_err,
        })
    }
}
