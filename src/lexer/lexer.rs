use std::mem;

use super::{Error, Result};
use crate::input_reader;

pub struct Lexer<R> {
    reader: R,
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
            reader: input_reader,
            current_token: None,
        };
        lexer.consume()?;

        Ok(lexer)
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
                'n' => {
                    let c1 = self.next_char()?;
                    let c2 = self.next_char()?;
                    let c3 = self.next_char()?;

                    match (c1, c2, c3) {
                        (Some(c1), Some(c2), Some(c3)) => {
                            if (c1, c2, c3) == ('u', 'l', 'l') {
                                Token::from(Literal { kind: Null })
                            } else {
                                Token::from(Unknown)
                            }
                        }
                        _ => Token::from(Unknown),
                    }
                }
                't' => {
                    let c1 = self.next_char()?;
                    let c2 = self.next_char()?;
                    let c3 = self.next_char()?;

                    match (c1, c2, c3) {
                        (Some(c1), Some(c2), Some(c3)) => {
                            if (c1, c2, c3) == ('r', 'u', 'e') {
                                Token::from(Literal {
                                    kind: Boolean(true),
                                })
                            } else {
                                Token::from(Unknown)
                            }
                        }
                        _ => Token::from(Unknown),
                    }
                }
                'f' => {
                    let c1 = self.next_char()?;
                    let c2 = self.next_char()?;
                    let c3 = self.next_char()?;
                    let c4 = self.next_char()?;

                    match (c1, c2, c3, c4) {
                        (Some(c1), Some(c2), Some(c3), Some(c4)) => {
                            if (c1, c2, c3, c4) == ('a', 'l', 's', 'e') {
                                Token::from(Literal {
                                    kind: Boolean(false),
                                })
                            } else {
                                Token::from(Unknown)
                            }
                        }
                        _ => Token::from(Unknown),
                    }
                }
                _ => Token::from(Unknown),
            })
        } else {
            self.current_token = None
        }

        Ok(())
    }

    fn next_char(&mut self) -> Result<Option<char>> {
        let c = self.reader.peek();
        self.reader.consume().map_err(Error::from)?;

        Ok(c)
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
