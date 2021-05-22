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

use ExpectedKind::*;
use Repr::*;

impl From<input_reader::Error> for Error {
    fn from(error: input_reader::Error) -> Self {
        Error {
            repr: Repr::InputReader(error),
        }
    }
}

impl From<Repr> for Error {
    fn from(repr: Repr) -> Self {
        Error { repr }
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
    Boolean(bool),
    Number(f64),
}

use LiteralKind::*;
use TokenKind::*;

impl<R> Lexer<R> {
    pub fn peek(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    fn into_iter(self) -> IntoIter<R> {
        IntoIter { lexer: self }
    }
}

impl<R: input_reader::ReadInput> Lexer<R> {
    pub fn new(input_reader: R) -> Result<Self> {
        let mut lexer = Lexer {
            input_reader,
            current_token: None,
        };
        lexer.consume()?;

        Ok(lexer)
    }

    pub fn consume(&mut self) -> Result<()> {
        self.current_token = None;

        if let Some(c) = self.advance_input_reader()? {
            let kind = match c {
                ' ' | '\n' | '\r' | '\t' => Whitespace,
                ',' => Comma,
                '{' => OpenBrace,
                '}' => CloseBrace,
                '[' => OpenBracket,
                ']' => CloseBracket,
                ':' => Colon,
                'n' => {
                    self.match_keyword("null")?;
                    Literal { kind: Null }
                }
                't' => {
                    self.match_keyword("true")?;
                    Literal {
                        kind: Boolean(true),
                    }
                }
                'f' => {
                    self.match_keyword("false")?;
                    Literal {
                        kind: Boolean(false),
                    }
                }
                '0'..='9' | '-' => Literal {
                    kind: Number(self.match_number(c)?),
                },
                _ => Unknown,
            };

            self.current_token = Some(Token::from(kind));
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

    fn match_keyword(&mut self, kw: &'static str) -> Result<()> {
        let actual_chars = (0..kw.len() - 1).filter_map(|k| self.input_reader.peek(k));
        let expect_chars = kw[1..].chars();

        if actual_chars.ne(expect_chars) {
            return Err(Error::from(Expected(Keyword(kw))));
        }

        self.input_reader.consume(kw.len() - 1)?;

        Ok(())
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

    fn match_number(&mut self, first_digit: char) -> Result<f64> {
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
            _ => Err(Error::from(Expected(Digit)))?,
        }

        if matches!(self.input_reader.peek(0), Some(c) if c == '.') {
            self.advance_input_reader().unwrap();
            digits.push('.');

            let fractional = self.consume_digits()?;
            if matches!(fractional.first(), None) {
                Err(Error::from(Expected(Digit)))?
            }

            digits.extend(fractional);
        }

        if matches!(self.input_reader.peek(0), Some(c) if c.to_ascii_lowercase() == 'e') {
            self.advance_input_reader().unwrap();
            digits.push('e');

            match self.input_reader.peek(0) {
                Some('-') => {
                    self.advance_input_reader().unwrap();
                    digits.push('-');
                }
                Some('+') => {
                    self.advance_input_reader().unwrap();
                }
                _ => {}
            }

            let exponent = self.consume_digits()?;
            if matches!(exponent.first(), None) {
                Err(Error::from(Expected(Digit)))?
            }

            digits.extend(exponent);
        }

        Ok(digits.into_iter().collect::<String>().parse().unwrap())
    }
}

impl From<TokenKind> for Token {
    fn from(kind: TokenKind) -> Self {
        Token { kind }
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
