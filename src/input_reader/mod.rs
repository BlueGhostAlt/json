//! [`char`]-based input readers based on [`Read`] implementing types.
//!
//! The `json::input_reader` module contains two common ways of reading input:
//! reading the whole input in memory and reading the input in fixed-size
//! buffers. Both input readers implement the [`ReadInput`] trait.
//!
//! # In-memory reading
//!
//! The [`MemoryReader`] input reader is recommended for inputs of trivial
//! size, where memory consumption isn't a concern, as it will deliver better
//! performance since there is no buffer to be refilled.
//!
//! # Buffered in-memory reading
//!
//! The [`BufferedReader`] input reader is recommended for inputs of a
//! significant size, where memory consumption matters more than performance,
//! as it will only ever buffer a fixed amount of bytes, at the cost of having
//! to refill its internal buffer once it has been consumed.
//!
//! # ReadInput
//!
//! The [`ReadInput`] trait describes a unifying interface for input readers,
//! with two fundamental methods: [`peek`] and [`consume`].
//! [`peek`] returns the current character in the input.
//! [`consume`] advances the reader onto the next character.
//!
//! # Examples
//!
//! Using [`MemoryReader`]:
//!
//! ```
//! use json::input_reader::{self, MemoryReader, ReadInput};
//!
//! fn main() -> input_reader::Result<()> {
//!     let mut reader = MemoryReader::new("json".as_bytes())?;
//!
//!     assert_eq!(reader.peek(), Some('j'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), Some('s'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), Some('o'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), Some('n'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), None);
//!
//!     Ok(())
//! }
//! ```
//!
//! And using [`BufferedReader`] with the exact same interface:
//!
//! ```
//! use json::input_reader::{self, BufferedReader, ReadInput};
//!
//! fn main() -> input_reader::Result<()> {
//!     let mut reader = BufferedReader::new("json".as_bytes())?;
//!
//!     assert_eq!(reader.peek(), Some('j'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), Some('s'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), Some('o'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), Some('n'));
//!     reader.consume()?;
//!     assert_eq!(reader.peek(), None);
//!
//!     Ok(())
//! }
//! ```
//!
//! [`Read`]: [`std::io::Read`]
//! [`peek`]: [`Reader::peek`]
//! [`consume`]: [`Reader::consume`]

use std::result;
use std::str;
use std::{error, fmt, io};

mod buffered_reader;
mod memory_reader;

pub use buffered_reader::BufferedReader;
pub use memory_reader::MemoryReader;

/// The `ReadInput` trait allows for peeking and consuming input.
///
/// Implementors of the `ReadInput` trait are called 'input readers'.
///
/// Input readers are defined by two required methods, [`peek()`] and
/// [`consume()`].
/// Each call to [`peek()`] will attempt to return the current
/// character from memory.
/// Each call to [`consume()`] will attempt to advance the input reader to the
/// next character in the input.
///
/// # Examples
///
/// Read input from a buffer of bytes, since [`&[u8]`] implements [`Read`]:
///
/// [`Read`]: [`std::io::Read`]
/// [`&[u8]`]: [`std::slice`]
///
/// ```
/// use json::input_reader::{self, MemoryReader, ReadInput};
///
/// fn main() -> input_reader::Result<()> {
// TODO: replace binary string with regular string, and use .as_bytes() instead
///     let buf: &[u8] = b"json";
///     let mut reader = MemoryReader::new(buf)?;
///
///     assert_eq!(reader.peek(), Some('j'));
///     reader.consume()?;
///     assert_eq!(reader.peek(), Some('s'));
///     reader.consume()?;
///     assert_eq!(reader.peek(), Some('o'));
///     reader.consume()?;
///     assert_eq!(reader.peek(), Some('n'));
///     reader.consume()?;
///     assert_eq!(reader.peek(), None);
///
///     Ok(())
/// }
/// ```
///
/// [`peek()`]: [`ReadInput::peek`]
/// [`consume()`]: [`ReadInput::consume`]
pub trait ReadInput {
    /// Returns the current character in the input.
    ///
    /// It is guaranteed that [`None`] will be returned only if no more
    /// characters are left in the input.
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
    ///
    ///     Ok(())
    /// }
    /// ```
    fn peek(&self) -> Option<char>;

    /// Advances the input reader by one character.
    ///
    /// # Errors
    /// This method, as of right now, can only fail when trying to refill the buffer,
    /// as is the case for the [`BufferedReader`]. Refilling the buffer might
    /// either yield an [`io::Error`] when trying to read from the input, or
    /// an [`str::Utf8Error`] while trying to convert the buffered bytes to a
    /// string slice.
    ///
    /// It is guaranteed that this operation will never fail for the
    /// [`MemoryReader`] input reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use json::input_reader::{self, MemoryReader, ReadInput};
    ///
    /// fn main() -> input_reader::Result<()> {
    // TODO: same as on line 109
    ///     let buf: &[u8] = b"json";
    ///     let mut reader = MemoryReader::new(buf)?;
    ///
    ///     assert_eq!(reader.peek(), Some('j'));
    ///     reader.consume()?;
    ///     assert_eq!(reader.peek(), Some('s'));
    ///
    ///     Ok(())
    /// }
    /// ```
    fn consume(&mut self) -> Result<()>;

    /// Checks whether or not the input has ran out of characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use json::input_reader::{self, MemoryReader, ReadInput};
    ///
    /// fn main() -> input_reader::Result<()> {
    ///     let mut reader = MemoryReader::new("json".as_bytes())?;
    ///
    ///     reader.consume()?;
    ///     reader.consume()?;
    ///     reader.consume()?;
    ///     reader.consume()?;
    ///     assert!(reader.has_reached_eof());
    ///
    ///     Ok(())
    /// }
    /// ```
    fn has_reached_eof(&self) -> bool {
        matches!(self.peek(), None)
    }
}

/// A specialized [`Result`] type for input reading operations.
///
/// This type is currently used for the [`consume()`] method as it might fail.
///
/// This typedef is generally used to avoid writing out [`input_reader::Error`]
/// directly and is otherwise a direct mapping to [`Result`].
///
/// While usual Rust style is to import types directly, aliases of [`Result`]
/// often are not, to make it easier to distinguish between them. [`Result`] is
/// generally assumed to be [`std::result::Result`], and so users of this alias
/// will generally use `input_reader::Result` instead of shadowing the
/// [prelude]'s import of [`std::result::Result`].
///
/// [`consume()`]: [`ReadInput::consume`]
/// [`input_reader::Error`]: Error
/// [`Result`]: std::result::Result
/// [prelude]: std::prelude
///
/// # Examples
///
/// A function that bubbles an `input_reader::Result` to its caller:
///
/// ```
/// use std::io;
///
/// use json::input_reader::{self, BufferedReader, ReadInput};
///
/// fn get_first_char<R>(input: R) -> input_reader::Result<Option<char>>
/// where
///     R: io::Read,
/// {
///     let reader = BufferedReader::new(input)?;
///
///     Ok(reader.peek())
/// }
/// ```
pub type Result<T> = result::Result<T, Error>;

/// The error type for input reading operations of the [`ReadInput`] trait.
///
/// Errors originate entirely from the lower-level modules, Errors being either
/// [I/O errors] or [UTF-8 errors].
///
/// [I/O errors]: std::io::Error
/// [UTF-8 errors]: std::str::Utf8Error
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

    #[test]
    fn test_readers_have_reached_eof() -> Result<()> {
        let mut buf_reader = BufferedReader::new(SOURCE)?;
        let mut mem_reader = MemoryReader::new(SOURCE)?;

        (buf_reader.consume()?, mem_reader.consume()?);
        (buf_reader.consume()?, mem_reader.consume()?);
        (buf_reader.consume()?, mem_reader.consume()?);
        (buf_reader.consume()?, mem_reader.consume()?);

        assert!(buf_reader.has_reached_eof());
        assert!(mem_reader.has_reached_eof());

        Ok(())
    }
}
