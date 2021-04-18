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

#[derive(Debug)]
enum TokenKind {
    OpenBrace,
    CloseBrace,

    Unknown,
}

impl<R: ReadInput> Lexer<R> {
    pub fn new(input_reader: R) -> Result<Self> {
        let mut lexer = Lexer {
            reader: input_reader,
            current_token: None,
        };
        lexer.consume()?;

        Ok(lexer)
    }

    pub fn peek(&self) -> Option<Token> {
        match &self.current_token {
            Some(t) => unsafe { Some(std::ptr::read(t as *const _)) },
            None => None,
        }
    }

    pub fn consume(&mut self) -> Result<()> {
        if let Some(c) = self.reader.peek() {
            self.reader.consume().map_err(Error::from)?;

            self.current_token = Some(match c {
                '{' => Token::from(TokenKind::OpenBrace),
                '}' => Token::from(TokenKind::CloseBrace),
                _ => Token::from(TokenKind::Unknown),
            })
        } else {
            self.current_token = None
        }

        Ok(())
    }
}

impl From<TokenKind> for Token {
    // TODO: Replace concrete types with Self in From implementations
    fn from(kind: TokenKind) -> Token {
        Token { kind }
    }
}

impl<R: ReadInput> Iterator for Lexer<R> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.peek()?;
        self.consume().ok()?;

        Some(c)
    }
}
