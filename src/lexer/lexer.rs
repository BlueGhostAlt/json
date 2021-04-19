use std::mem;

use super::{Error, Result};
use crate::input_reader;

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
        if let Some(c) = self.input_reader.peek(0) {
            self.input_reader.consume(1).map_err(Error::from)?;

            let token = match c {
                ' ' | '\n' | '\r' | '\t' => Token::from(Whitespace),
                ',' => Token::from(Comma),
                '{' => Token::from(OpenBrace),
                '}' => Token::from(CloseBrace),
                '[' => Token::from(OpenBracket),
                ']' => Token::from(CloseBracket),
                ':' => Token::from(Colon),
                'n' => {
                    if self.match_keyword("null")? {
                        Token::from(Literal { kind: Null })
                    } else {
                        Token::from(Unknown)
                    }
                }
                't' => {
                    if self.match_keyword("true")? {
                        Token::from(Literal {
                            kind: Boolean(true),
                        })
                    } else {
                        Token::from(Unknown)
                    }
                }
                'f' => {
                    if self.match_keyword("false")? {
                        Token::from(Literal {
                            kind: Boolean(false),
                        })
                    } else {
                        Token::from(Unknown)
                    }
                }
                _ => Token::from(Unknown),
            };

            self.current_token = Some(token)
        } else {
            self.current_token = None
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
    // TODO: Replace concrete types with Self in From implementations
    fn from(kind: TokenKind) -> Token {
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
        let c = mem::take(&mut self.lexer.current_token)?;
        self.lexer.consume().ok();

        Some(c)
    }
}
