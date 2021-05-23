use std::{borrow::Cow, error, fmt, result};

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
    Unexpected(char),
}

#[derive(Debug)]
enum ExpectedKind {
    Keyword(&'static str),
    Digit(DigitKind),
    StringTerminator,
    EscapedCharacter,
}

#[derive(Debug)]
enum DigitKind {
    Dec,
    Hex,
}

use DigitKind::{Dec, Hex};
use ExpectedKind::{Digit, EscapedCharacter, Keyword, StringTerminator};
use Repr::{Expected, InputReader, Unexpected};

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
            Expected(expected_kind) => match expected_kind {
                Keyword(kw) => write!(f, "expected keyword \"{}\"", kw),
                Digit(kind) => match kind {
                    Hex => write!(f, "expected hexadecimal digit"),
                    Dec => write!(f, "expected digit"),
                },
                StringTerminator => write!(f, "expected string terminator '\"'"),
                EscapedCharacter => write!(f, "expected escaped character"),
            },
            Unexpected(unexpected_char) => write!(f, "unexpected character '{}'", unexpected_char),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.repr {
            InputReader(input_reader_err) => Some(input_reader_err),
            Expected(_expected_kind) => None,
            Unexpected(_unexpected_char) => None,
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
    raw: Cow<'static, str>,
}

#[derive(Debug)]
pub struct IntoIter<R> {
    lexer: Lexer<R>,
    last_err: Option<Error>,
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
}

#[derive(Debug)]
enum LiteralKind {
    Null,
    Boolean,
    Number,
    String,
}

use LiteralKind::{Boolean, Null, Number};
use TokenKind::{
    CloseBrace, CloseBracket, Colon, Comma, Literal, OpenBrace, OpenBracket, Whitespace,
};

impl From<(TokenKind, Cow<'static, str>)> for Token {
    fn from((kind, raw): (TokenKind, Cow<'static, str>)) -> Self {
        Self { kind, raw }
    }
}

fn char_to_cow_str(ch: char) -> Cow<'static, str> {
    Cow::from(String::from(ch))
}

impl<R> Lexer<R> {
    pub const fn peek(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    const fn into_iter(self) -> IntoIter<R> {
        IntoIter {
            lexer: self,
            last_err: None,
        }
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
                ' ' | '\n' | '\r' | '\t' => (Whitespace, char_to_cow_str(c)),
                ',' => (Comma, char_to_cow_str(c)),
                '{' => (OpenBrace, char_to_cow_str(c)),
                '}' => (CloseBrace, char_to_cow_str(c)),
                '[' => (OpenBracket, char_to_cow_str(c)),
                ']' => (CloseBracket, char_to_cow_str(c)),
                ':' => (Colon, char_to_cow_str(c)),
                'n' => (Literal { kind: Null }, self.match_keyword("null")?.into()),
                't' => (
                    Literal { kind: Boolean },
                    self.match_keyword("true")?.into(),
                ),
                'f' => (
                    Literal { kind: Boolean },
                    self.match_keyword("false")?.into(),
                ),
                '0'..='9' | '-' => (Literal { kind: Number }, self.match_number(c)?.into()),
                '"' => (
                    Literal {
                        kind: LiteralKind::String,
                    },
                    self.match_string()?.into(),
                ),
                _ => return Err(Error::from(Unexpected(c))),
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

    fn match_keyword(&mut self, kw: &'static str) -> Result<&'static str> {
        let actual = (0..kw.len() - 1).filter_map(|k| self.input_reader.peek(k));

        if actual.ne(kw.chars().skip(1)) {
            return Err(Error::from(Expected(Keyword(kw))));
        }

        self.input_reader.consume(kw.len() - 1)?;

        Ok(kw)
    }

    fn consume_digits(&mut self) -> Result<String> {
        let mut digits = String::new();

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
        let mut literal = String::from(first_digit);

        let first_digit = if first_digit == '-' {
            let c = self
                .advance_input_reader()?
                .ok_or_else(|| Error::from(Expected(Digit(Dec))))?;
            literal.push(c);

            c
        } else {
            first_digit
        };

        match first_digit {
            '1'..='9' => literal.push_str(&self.consume_digits()?),
            '0' => {}
            _ => return Err(Error::from(Expected(Digit(Dec)))),
        }

        if matches!(self.input_reader.peek(0), Some(c) if c == '.') {
            self.advance_input_reader().unwrap();
            literal.push('.');

            let fractional = self.consume_digits()?;
            if fractional.is_empty() {
                return Err(Error::from(Expected(Digit(Dec))));
            }

            literal.push_str(&fractional);
        }

        if matches!(self.input_reader.peek(0), Some('e' | 'E')) {
            let c = self.input_reader.peek(0).unwrap();
            self.advance_input_reader().unwrap();
            literal.push(c);

            match self.input_reader.peek(0) {
                Some(c @ ('-' | '+')) => {
                    self.advance_input_reader().unwrap();
                    literal.push(c);
                }
                _ => {}
            }

            let exponent = self.consume_digits()?;
            if exponent.is_empty() {
                return Err(Error::from(Expected(Digit(Dec))));
            }

            literal.push_str(&exponent);
        }

        Ok(literal)
    }

    fn match_string(&mut self) -> Result<String> {
        let mut codepoints = String::new();

        loop {
            match self.advance_input_reader()? {
                Some(c) if c == '"' => break,
                Some(c) if c.is_ascii_control() => return Err(Error::from(Unexpected(c))),
                Some(c) if c == '\\' => {
                    codepoints.push(c);

                    match self.advance_input_reader()? {
                        Some(c @ ('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')) => {
                            codepoints.push(c)
                        }
                        Some('u') => {
                            let next_four = (0..4).filter_map(|i| self.input_reader.peek(i));
                            let valid_count = next_four.filter(char::is_ascii_hexdigit).count();

                            if valid_count != 4 {
                                self.input_reader.consume(valid_count + 1)?;
                                return Err(Error::from(Expected(Digit(Hex))));
                            }

                            codepoints.push('u');
                            for _ in 0..4 {
                                codepoints.push(self.advance_input_reader()?.unwrap());
                            }
                        }
                        _ => return Err(Error::from(Expected(EscapedCharacter))),
                    }
                }
                Some(c) => codepoints.push(c),
                None => return Err(Error::from(Expected(StringTerminator))),
            }
        }

        Ok(codepoints)
    }
}

impl<R: input_reader::ReadInput> IntoIterator for Lexer<R> {
    type Item = Result<Token>;
    type IntoIter = IntoIter<R>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<R: input_reader::ReadInput> Iterator for IntoIter<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.last_err.take() {
            self.last_err = self.lexer.consume().err();
            return Some(Err(err));
        }

        let c = self.lexer.current_token.take()?;
        self.last_err = self.lexer.consume().err();

        Some(Ok(c))
    }
}
