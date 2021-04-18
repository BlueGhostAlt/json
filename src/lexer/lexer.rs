use std::mem;

use super::{Error, Result};
use crate::input_reader::ReadInput;

pub struct Lexer<R: ReadInput> {
    reader: R,
    current_token: Option<Token>,
}

#[derive(Debug)]
pub struct Token {
    #[allow(dead_code)]
    kind: TokenKind,
}

pub struct IntoIter<R: ReadInput> {
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

    Unknown,
}

use TokenKind::*;

impl<R: ReadInput> Lexer<R> {
    pub fn new(input_reader: R) -> Result<Self> {
        let mut lexer = Lexer {
            reader: input_reader,
            current_token: None,
        };
        lexer.consume()?;

        Ok(lexer)
    }

    pub fn peek(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn consume(&mut self) -> Result<()> {
        if let Some(c) = self.reader.peek() {
            self.reader.consume().map_err(Error::from)?;

            self.current_token = Some(match c {
                ' ' | '\n' | '\r' | '\t' => Token::from(Whitespace),
                ',' => Token::from(Comma),
                '{' => Token::from(OpenBrace),
                '}' => Token::from(CloseBrace),
                '[' => Token::from(OpenBracket),
                ']' => Token::from(CloseBracket),
                ':' => Token::from(Colon),
                _ => Token::from(Unknown),
            })
        } else {
            self.current_token = None
        }

        Ok(())
    }

    fn into_iter(self) -> IntoIter<R> {
        IntoIter { lexer: self }
    }
}

impl From<TokenKind> for Token {
    // TODO: Replace concrete types with Self in From implementations
    fn from(kind: TokenKind) -> Token {
        Token { kind }
    }
}

impl<R: ReadInput> IntoIterator for Lexer<R> {
    type Item = Token;
    type IntoIter = IntoIter<R>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<R: ReadInput> Iterator for IntoIter<R> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let c = mem::take(&mut self.lexer.current_token)?;
        self.lexer.consume().ok();

        Some(c)
    }
}
