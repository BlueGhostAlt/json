use std::{error, fmt, result};

use crate::input_reader;

#[derive(Debug)]
pub struct Error {
    #[allow(dead_code)]
    repr: Repr,
}

#[derive(Debug)]
enum Repr {
    InputReader(input_reader::Error),
    Expected(ExpectedKind),
}

#[derive(Debug)]
enum ExpectedKind {
    Keyword(&'static str),
    Digit,
}

use ExpectedKind::{Digit, Keyword};
use Repr::{Expected, InputReader};

impl From<input_reader::Error> for Error {
    fn from(error: input_reader::Error) -> Self {
        Self {
            repr: Repr::InputReader(error),
        }
    }
}

impl From<Repr> for Error {
    fn from(repr: Repr) -> Self {
        Self { repr }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            InputReader(input_reader_err) => write!(f, "{}", input_reader_err),
            Expected(expected_err) => match expected_err {
                Keyword(kw) => write!(f, "expected keyword {}", kw),
                Digit => write!(f, "expected digit"),
            },
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.repr {
            InputReader(input_reader_err) => Some(input_reader_err),
            Expected(_expected_err) => None,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Lexer<R> {
    input_reader: R,
    current_token: Option<Token>,
}

#[derive(Debug)]
pub struct Token {
    #[allow(dead_code)]
    kind: TokenKind,
    raw: String,
}

#[derive(Debug)]
pub struct IntoIter<R> {
    lexer: Lexer<R>,
}

#[derive(Debug)]
enum TokenKind {
    Whitespace,

    Comma,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,

    Literal { kind: LiteralKind },

    Unknown,
}

#[derive(Debug)]
enum LiteralKind {
    Null,
    Boolean,
    Number,
}

use LiteralKind::{Boolean, Null, Number};
use TokenKind::{
    CloseBrace, CloseBracket, Colon, Comma, Literal, OpenBrace, OpenBracket, Unknown, Whitespace,
};

impl From<(TokenKind, String)> for Token {
    fn from((kind, raw): (TokenKind, String)) -> Self {
        Self { kind, raw }
    }
}

impl<R> Lexer<R> {
    pub const fn peek(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    const fn into_iter(self) -> IntoIter<R> {
        IntoIter { lexer: self }
    }
}

impl<R: input_reader::ReadInput> Lexer<R> {
    pub fn new(input_reader: R) -> Result<Self> {
        let mut lexer = Self {
            input_reader,
            current_token: None,
        };
        lexer.consume()?;

        Ok(lexer)
    }

    pub fn consume(&mut self) -> Result<()> {
        self.current_token = None;

        if let Some(c) = self.advance_input_reader()? {
            let (kind, raw) = match c {
                ' ' | '\n' | '\r' | '\t' => (Whitespace, c.into()),
                ',' => (Comma, c.into()),
                '{' => (OpenBrace, c.into()),
                '}' => (CloseBrace, c.into()),
                '[' => (OpenBracket, c.into()),
                ']' => (CloseBracket, c.into()),
                ':' => (Colon, c.into()),
                'n' => (Literal { kind: Null }, self.match_keyword("null")?),
                't' => (Literal { kind: Boolean }, self.match_keyword("true")?),
                'f' => (Literal { kind: Boolean }, self.match_keyword("false")?),
                '0'..='9' | '-' => (Literal { kind: Number }, self.match_number(c)?),
                _ => (Unknown, c.into()),
            };

            self.current_token = Some(Token::from((kind, raw)));
        }

        Ok(())
    }

    fn advance_input_reader(&mut self) -> Result<Option<char>> {
        if let Some(c) = self.input_reader.peek(0) {
            self.input_reader.consume(1)?;

            return Ok(Some(c));
        }

        Ok(None)
    }

    fn match_keyword(&mut self, kw: &'static str) -> Result<String> {
        let actual = (0..kw.len() - 1)
            .filter_map(|k| self.input_reader.peek(k))
            .collect::<String>();

        if &actual != kw {
            return Err(Error::from(Expected(Keyword(kw))));
        }

        self.input_reader.consume(kw.len() - 1)?;

        Ok(actual)
    }

    fn consume_digits(&mut self) -> Result<Vec<char>> {
        let mut digits = Vec::new();

        loop {
            match self.input_reader.peek(0) {
                Some('_') => {
                    self.input_reader.consume(1)?;
                }
                Some(c @ '0'..='9') => {
                    digits.push(c);
                    self.input_reader.consume(1)?;
                }
                _ => break,
            }
        }

        Ok(digits)
    }

    fn match_number(&mut self, first_digit: char) -> Result<String> {
        let mut digits = vec![first_digit];

        let first_digit = if first_digit == '-' {
            let c = self
                .advance_input_reader()?
                .ok_or_else(|| Error::from(Expected(Digit)))?;
            digits.push(c);

            c
        } else {
            first_digit
        };

        match first_digit {
            '1'..='9' => digits.extend(self.consume_digits()?),
            '0' => {}
            _ => return Err(Error::from(Expected(Digit))),
        }

        if matches!(self.input_reader.peek(0), Some(c) if c == '.') {
            self.advance_input_reader().unwrap();
            digits.push('.');

            let fractional = self.consume_digits()?;
            if matches!(fractional.first(), None) {
                return Err(Error::from(Expected(Digit)));
            }

            digits.extend(fractional);
        }

        if matches!(self.input_reader.peek(0), Some('e' | 'E')) {
            let c = self.input_reader.peek(0).unwrap();
            self.advance_input_reader().unwrap();
            digits.push(c);

            match self.input_reader.peek(0) {
                Some(c @ ('-' | '+')) => {
                    self.advance_input_reader().unwrap();
                    digits.push(c);
                }
                _ => {}
            }

            let exponent = self.consume_digits()?;
            if matches!(exponent.first(), None) {
                return Err(Error::from(Expected(Digit)));
            }

            digits.extend(exponent);
        }

        Ok(digits.into_iter().collect::<String>())
    }
}

impl<R: input_reader::ReadInput> IntoIterator for Lexer<R> {
    type Item = Token;
    type IntoIter = IntoIter<R>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<R: input_reader::ReadInput> Iterator for IntoIter<R> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.lexer.current_token.take()?;
        self.lexer.consume().ok();

        Some(c)
    }
}
