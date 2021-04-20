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
//! with two fundamental methods: [`peek(k)`] and [`consume(k)`].
//! [`peek(k)`] returns the k-th character in the input from the current
//! position.
//! [`consume(k)`] advances the input reader's position by k characters.
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
//!     assert_eq!(reader.peek(0), Some('j'));
//!     assert_eq!(reader.peek(1), Some('s'));
//!     reader.consume(1)?;
//!     assert_eq!(reader.peek(1), Some('o'));
//!     reader.consume(0)?;
//!     assert_eq!(reader.peek(2), Some('n'));
//!     reader.consume(2)?;
//!     assert_eq!(reader.peek(0), Some('n'));
//!     reader.consume(1)?;
//!     assert_eq!(reader.peek(0), None);
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
//!     assert_eq!(reader.peek(0), Some('j'));
//!     assert_eq!(reader.peek(1), Some('s'));
//!     reader.consume(1)?;
//!     assert_eq!(reader.peek(1), Some('o'));
//!     reader.consume(0)?;
//!     assert_eq!(reader.peek(2), Some('n'));
//!     reader.consume(2)?;
//!     assert_eq!(reader.peek(0), Some('n'));
//!     reader.consume(1)?;
//!     assert_eq!(reader.peek(0), None);
//!
//!     Ok(())
//! }
//! ```
//!
//! [`Read`]: [`std::io::Read`]
//! [`peek(k)`]: [`Reader::peek`]
//! [`consume(k)`]: [`Reader::consume`]

use std::{error, fmt, io, iter, result, str};

mod buffered_reader;
mod memory_reader;

pub use buffered_reader::BufferedReader;
pub use memory_reader::MemoryReader;

use buffered_reader::BUF_READER_CAPACITY;

/// The `ReadInput` trait allows for peeking and consuming input.
///
/// Implementors of the `ReadInput` trait are called 'input readers'.
///
/// Input readers are defined by two required methods, [`peek(k)`] and
/// [`consume(k)`].
/// Each call to [`peek(k)`] will attempt to return the k-th character from
/// the current position of the input reader.
/// Each call to [`consume(k)`] will attempt to advance the input reader's
/// position by k characters.
///
/// # Examples
///
/// Read input from a buffer of bytes, since [`&[u8]`][`std::slice`] implements
/// [`Read`]:
///
/// [`Read`]: [`std::io::Read`]
///
/// ```
/// use json::input_reader::{self, MemoryReader, ReadInput};
///
/// fn main() -> input_reader::Result<()> {
///     let buf: &[u8] = "json".as_bytes();
///     let mut reader = MemoryReader::new(buf)?;
///
///     assert_eq!(reader.peek(0), Some('j'));
///     assert_eq!(reader.peek(1), Some('s'));
///     reader.consume(1)?;
///     assert_eq!(reader.peek(1), Some('o'));
///     reader.consume(0)?;
///     assert_eq!(reader.peek(2), Some('n'));
///     reader.consume(2)?;
///     assert_eq!(reader.peek(0), Some('n'));
///     reader.consume(1)?;
///     assert_eq!(reader.peek(0), None);
///
///     Ok(())
/// }
/// ```
///
/// [`peek(k)`]: [`ReadInput::peek`]
/// [`consume(k)`]: [`ReadInput::consume`]
pub trait ReadInput {
    /// Returns the k-th character in the input from the current position.
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
    ///     assert_eq!(reader.peek(1), Some('s'));
    ///     assert_eq!(reader.peek(2), Some('o'));
    ///     assert_eq!(reader.peek(3), Some('n'));
    ///     assert_eq!(reader.peek(4), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    fn peek(&self, k: usize) -> Option<char>;

    /// Advances the input reader's position by k characters.
    ///
    /// # Errors
    /// This method can fail only when using a [`BufferedReader`], due to
    /// multiple reasons. One of them is trying to consume more characters than
    /// the internal buffer holds, 16 characters.
    /// This method can also fail when trying to refill the buffer. Refilling
    /// the buffer might either yield an [`io::Error`] when trying to read from
    /// the input, or an [`str::Utf8Error`] while trying to convert the
    /// buffered bytes to a string slice.
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
    ///     let buf: &[u8] = "json".as_bytes();
    ///     let mut reader = MemoryReader::new(buf)?;
    ///
    ///     assert_eq!(reader.peek(0), Some('j'));
    ///     reader.consume(2)?;
    ///     assert_eq!(reader.peek(0), Some('o'));
    ///
    ///     Ok(())
    /// }
    /// ```
    fn consume(&mut self, k: usize) -> Result<()>;

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
    ///     reader.consume(1)?;
    ///     reader.consume(2)?;
    ///     reader.consume(0)?;
    ///     reader.consume(1)?;
    ///     assert!(reader.has_reached_eof());
    ///
    ///     Ok(())
    /// }
    /// ```
    fn has_reached_eof(&self) -> bool {
        matches!(self.peek(0), None)
    }

    /// Returns an ergonomic iterator over this input reader's input
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use json::input_reader::{self, MemoryReader, ReadInput};
    ///
    /// fn main() -> input_reader::Result<()> {
    ///     let mut reader = MemoryReader::new("json".as_bytes())?;
    ///     let mut input_reader = reader.input_reader();
    ///
    ///     assert_eq!(
    ///         input_reader
    ///             .map(|c| c.to_ascii_uppercase())
    ///             .collect::<String>(),
    ///         String::from("JSON")
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    fn input_reader(&mut self) -> InputReader<'_, Self>
    where
        Self: Sized,
    {
        InputReader(self)
    }
}

/// Iterator over an input reader's input
///
/// This struct is created by the [`input_reader`] method on input readers.
///
/// # Examples
///
/// ```
/// use json::input_reader::{self, MemoryReader, ReadInput};
///
/// fn main() -> input_reader::Result<()> {
///     let mut reader = MemoryReader::new("json".as_bytes())?;
///     let mut input_reader = reader.input_reader();
///
///     assert!(input_reader.all(|c| c.is_alphabetic()));
///
///     Ok(())
/// }
/// ```
///
/// [`input_reader`]: [`ReadInput::input_reader`]
pub struct InputReader<'a, R>(&'a mut R);

impl<R: ReadInput> Iterator for InputReader<'_, R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.0.peek(0)?;
        self.0.consume(1).ok()?;

        Some(c)
    }
}

impl<R: ReadInput> iter::FusedIterator for InputReader<'_, R> {}

/// A specialized [`Result`] type for input reading operations.
///
/// This type is currently used for the [`consume(k)`] method as it might fail.
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
/// [`consume(k)`]: [`ReadInput::consume`]
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
///     Ok(reader.peek(0))
/// }
/// ```
pub type Result<T> = result::Result<T, Error>;

/// The error type for input reading operations of the [`ReadInput`] trait.
///
/// Errors originate mostly from the lower-level modules, foreign Errors being
/// either [I/O errors] or [UTF-8 errors]. There might also be buffer errors
/// caused by using a [`BufferedReader`] wrong.
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
    Buffer(BufferErrorKind),
}

#[derive(Debug)]
enum BufferErrorKind {
    Overconsumed(usize),
}

impl Error {
    fn overconsume_buffer(count: usize) -> Self {
        Error {
            repr: Repr::Buffer(BufferErrorKind::Overconsumed(count)),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            repr: Repr::Io(error),
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(error: str::Utf8Error) -> Self {
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
            Repr::Buffer(buffer_err) => match buffer_err {
                BufferErrorKind::Overconsumed(count) => write!(
                    f,
                    "input reader consumed {} characters when the buffer holds only {} characters",
                    count, BUF_READER_CAPACITY
                ),
            },
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.repr {
            Repr::Io(io_err) => Some(io_err),
            Repr::Utf8(utf8_err) => Some(utf8_err),
            Repr::Buffer(_) => None,
        }
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
        let mut buf_reader = BufferedReader::new(EMPTY_SOURCE)?;
        let mut mem_reader = MemoryReader::new(EMPTY_SOURCE)?;

        assert!(buf_reader.input_reader().eq(mem_reader.input_reader()));

        Ok(())
    }

    #[test]
    fn test_readers_are_eq() -> Result<()> {
        let mut buf_reader = BufferedReader::new(SOURCE)?;
        let mut mem_reader = MemoryReader::new(SOURCE)?;

        assert!(buf_reader.input_reader().eq(mem_reader.input_reader()));

        Ok(())
    }

    #[test]
    fn test_readers_have_reached_eof() -> Result<()> {
        let mut buf_reader = BufferedReader::new(SOURCE)?;
        let mut mem_reader = MemoryReader::new(SOURCE)?;

        buf_reader.consume(1)?;
        buf_reader.consume(2)?;
        buf_reader.consume(0)?;
        buf_reader.consume(1)?;

        mem_reader.consume(1)?;
        mem_reader.consume(2)?;
        mem_reader.consume(0)?;
        mem_reader.consume(1)?;

        assert!(buf_reader.has_reached_eof());
        assert!(mem_reader.has_reached_eof());

        Ok(())
    }
}
