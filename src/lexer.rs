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
}

impl From<input_reader::Error> for Error {
    fn from(error: input_reader::Error) -> Self {
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

pub type Result<T> = result::Result<T, Error>;

pub struct Lexer<R> {
    input_reader: R,
    current_token: Option<Token>,
}

#[derive(Debug)]
pub struct Token {
    #[allow(dead_code)]
    kind: TokenKind,
}

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

        if let Some(c) = self.input_reader.peek(0) {
            self.input_reader.consume(1).map_err(Error::from)?;

            let kind = match c {
                ' ' | '\n' | '\r' | '\t' => Whitespace,
                ',' => Comma,
                '{' => OpenBrace,
                '}' => CloseBrace,
                '[' => OpenBracket,
                ']' => CloseBracket,
                ':' => Colon,
                'n' if self.match_keyword("null")? => Literal { kind: Null },
                't' if self.match_keyword("true")? => Literal {
                    kind: Boolean(true),
                },
                'f' if self.match_keyword("false")? => Literal {
                    kind: Boolean(false),
                },
                _ => Unknown,
            };

            self.current_token = Some(Token::from(kind));
        }

        Ok(())
    }

    fn match_keyword(&mut self, kw: &str) -> Result<bool> {
        let actual_chars = (0..kw.len() - 1).filter_map(|k| self.input_reader.peek(k));
        let expect_chars = kw[1..].chars();

        if actual_chars.ne(expect_chars) {
            return Ok(false);
        }

        self.input_reader
            .consume(kw.len() - 1)
            .map_err(Error::from)?;

        Ok(true)
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
