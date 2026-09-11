#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,

    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Number,
    Identifier,
    String,

    If,
    Else,
    Fn,
    Return,
    Let,

    Semicolon,
    Comma,
    Dot,

    Eof,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Token<'a> {
    typ: TokenType,
    line: usize,
    lexeme: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(typ: TokenType, line: usize, lexeme: &'a str) -> Token<'a> {
        Token { typ, line, lexeme }
    }

    pub fn typ(&self) -> TokenType {
        self.typ
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn lexeme(&self) -> &'a str {
        self.lexeme
    }
}

use std::collections::HashMap;

use TokenType::*;

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            Plus => write!(f, "PLUS"),
            Minus => write!(f, "MINUS"),
            Star => write!(f, "STAR"),
            Slash => write!(f, "SLASH"),
            Equal => write!(f, "EQUAL"),
            Less => write!(f, "LESS"),
            LessEqual => write!(f, "LESS_EQUAL"),
            Greater => write!(f, "GREATER"),
            GreaterEqual => write!(f, "GREATER_EQUAL"),
            LeftParen => write!(f, "LEFT_PAREN"),
            RightParen => write!(f, "RIGHT_PAREN"),
            LeftBrace => write!(f, "LEFT_BRACE"),
            RightBrace => write!(f, "RIGHT_BRACE"),
            Number => write!(f, "NUM {}", self.lexeme()),
            Identifier => write!(f, "IDENT {}", self.lexeme()),
            String => write!(f, "STRING {}", self.lexeme()),
            If => write!(f, "IF"),
            Else => write!(f, "ELSE"),
            Fn => write!(f, "FN"),
            Return => write!(f, "RETURN"),
            Let => write!(f, "LET"),
            Semicolon => write!(f, "SEMICOLON"),
            Comma => write!(f, "COMMA"),
            Dot => write!(f, "DOT"),
            Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug)]
pub enum ScanError {
    UnexpectedChar(char),
    Io(std::io::Error),
    UnterminatedString,
}

impl From<std::io::Error> for ScanError {
    fn from(value: std::io::Error) -> Self {
        ScanError::Io(value)
    }
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::Io(error) => write!(f, "I/O error: {}", error),
            ScanError::UnexpectedChar(c) => write!(f, "Unexpected character: {}", c),
            ScanError::UnterminatedString => write!(f, "Unterminated string"),
        }
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
            keywords: Self::keywords(),
        }
    }

    fn keywords() -> HashMap<&'static str, TokenType> {
        HashMap::from([
            ("if", If),
            ("else", Else),
            ("fn", Fn),
            ("return", Return),
            ("let", Let),
        ])
    }

    pub fn scan(mut self) -> Result<Vec<Token<'a>>, ScanError> {
        let mut tokens = vec![];

        loop {
            self.skip_whitespace();
            self.start = self.current;
            if self.is_at_end() {
                break;
            }
            tokens.push(self.token()?);
        }

        tokens.push(self.make(Eof));

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_whitespace() {
            if self.consume() == '\n' {
                self.line += 1;
            }
        }
    }

    fn token(&mut self) -> Result<Token<'a>, ScanError> {
        Ok(match self.consume() {
            '+' => self.make(Plus),
            '-' => self.make(Minus),
            '*' => self.make(Star),
            '/' => self.make(Slash),
            '=' => self.make(Equal),
            '<' => {
                if self.match_char('=') {
                    self.make(LessEqual)
                } else {
                    self.make(Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make(GreaterEqual)
                } else {
                    self.make(Greater)
                }
            }
            '(' => self.make(LeftParen),
            ')' => self.make(RightParen),
            '{' => self.make(LeftBrace),
            '}' => self.make(RightBrace),
            ';' => self.make(Semicolon),
            ',' => self.make(Comma),
            '.' => self.make(Dot),
            '"' => self.string()?,
            c if c.is_ascii_digit() => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            c => return Err(ScanError::UnexpectedChar(c)),
        })
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    fn consume(&mut self) -> char {
        let c = self.peek();
        self.current += c.len_utf8();
        c
    }

    fn match_char(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.consume();
            return true;
        }
        false
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn make(&self, typ: TokenType) -> Token<'a> {
        let lexeme = &self.source[self.start..self.current];
        Token::new(typ, self.line, lexeme)
    }

    fn number(&mut self) -> Token<'a> {
        while self.peek().is_ascii_digit() {
            self.consume();

            if self.peek() == '.' {
                self.consume();
                while self.peek().is_ascii_digit() {
                    self.consume();
                }
                break;
            }
        }

        self.make(Number)
    }

    fn identifier(&mut self) -> Token<'a> {
        loop {
            let c = self.peek();

            if c.is_alphabetic() || c.is_ascii_digit() || c == '_' {
                self.consume();
            } else {
                break;
            }
        }

        let value = &self.source[self.start..self.current];

        match self.keywords.get(value) {
            Some(typ) => self.make(*typ),
            None => self.make(Identifier),
        }
    }

    fn string(&mut self) -> Result<Token<'a>, ScanError> {
        loop {
            match self.consume() {
                '\0' => return Err(ScanError::UnterminatedString),
                '\n' => self.line += 1,
                '"' => return Ok(self.make(String)),
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests;
