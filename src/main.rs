fn main() {
    let mut input = String::new();

    loop {
        print!("> ");
        stdout().flush().unwrap();

        input.clear();
        stdin().read_line(&mut input).expect("could not read from stdin for some reason");
        
        let scanner = Scanner::new(&input);
        let tokens = scanner.scan();

        for t in tokens {
            println!("{:>4} {}", t.line, t);
        }
    }    
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenType<'a> {
    Plus,
    Minus,
    Star,
    Slash,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Number(f64),
    Identifier(&'a str),

    If,
    Else,
    
    Semicolon,
    
    Eof,
}

#[derive(Debug, PartialEq)]
struct Token<'a> {
    typ: TokenType<'a>,
    line: usize,
    lexeme: &'a str,
}

impl<'a> Token<'a> {
    fn new(typ: TokenType<'a>, line: usize, lexeme: &'a str) -> Token<'a> {
        Token { typ, line, lexeme }
    }
}

use std::{collections::HashMap, io::{Write, stdin, stdout}};

use TokenType::*;

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            Plus => write!(f, "PLUS"),
            Minus => write!(f, "MINUS"),
            Star => write!(f, "STAR"),
            Slash => write!(f, "SLASH"),
            LeftParen => write!(f, "LEFT_PAREN"),
            RightParen => write!(f, "RIGHT_PAREN"),
            LeftBrace => write!(f, "LEFT_BRACE"),
            RightBrace => write!(f, "RIGHT_BRACE"),
            Number(value) => write!(f, "NUM {}", value),
            Identifier(value) => write!(f, "IDENT {}", value),
            If => write!(f, "IF"),
            Else => write!(f, "ELSE"),
            Semicolon => write!(f, "SEMICOLON"),
            Eof => write!(f, "EOF"),
        }
    }
}

struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenType<'a>>,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
            keywords: Self::keywords(),
        }
    }

    fn keywords() -> HashMap<&'static str, TokenType<'a>> {
        HashMap::from([
            ("if", If),
            ("else", Else),
        ])
    }

    fn scan(mut self) -> Vec<Token<'a>> {
        let mut tokens = vec![];

        loop {
            self.skip_whitespace();
            self.start = self.current;
            if self.is_at_end() {
                break;
            }
            tokens.push(self.token());
        }

        tokens.push(self.make(Eof));

        tokens
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_whitespace() {
            if self.consume() == '\n' {
                self.line += 1;
            }
        }
    }

    fn token(&mut self) -> Token<'a> {
        match self.consume() {
            '+' => self.make(Plus),
            '-' => self.make(Minus),
            '*' => self.make(Star),
            '/' => self.make(Slash),
            '(' => self.make(LeftParen),
            ')' => self.make(RightParen),
            '{' => self.make(LeftBrace),
            '}' => self.make(RightBrace),
            ';' => self.make(Semicolon),
            c if c.is_digit(10) => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            _ => unreachable!(),
        }
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    fn consume(&mut self) -> char {
        let c = self.peek();
        self.current += c.len_utf8();
        c
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn make(&self, typ: TokenType<'a>) -> Token<'a> {
        let lexeme = &self.source[self.start..self.current];
        Token::new(typ, self.line, lexeme)
    }

    fn number(&mut self) -> Token<'a> {
        while self.peek().is_ascii_digit() {
            self.consume();
        }

        let value = self.source[self.start..self.current]
            .parse::<f64>()
            .unwrap();

        self.make(Number(value))
    }

    fn identifier(&mut self) -> Token<'a> {
        loop {
            let c = self.peek();

            if c.is_alphabetic() || c.is_digit(10) || c == '_' {
                self.consume();
            } else {
                break;
            }
        }

        let value = &self.source[self.start..self.current];

        match self.keywords.get(value) {
            Some(typ) => self.make(*typ),
            None => self.make(Identifier(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_sample_with_exact_lexemes() {
        let source = String::from(" 23; 47; 69 ; 67;88;99;");
        let tokens = Scanner::new(&source).scan();
        let expected = [
            "23", ";", "47", ";", "69", ";", "67", ";", "88", ";", "99", ";", "",
        ];
        assert_eq!(
            tokens.iter().map(|t| t.lexeme).collect::<Vec<_>>(),
            expected
        );
        for (token, value) in tokens.iter().step_by(2).zip([23., 47., 69., 67., 88., 99.]) {
            assert_eq!(token.typ, Number(value));
            assert_eq!(token.line, 1);
        }
        for token in tokens[..12].iter().skip(1).step_by(2) {
            assert_eq!(token.typ, Semicolon);
        }
        assert_eq!(tokens.last().unwrap().typ, Eof);
    }

    #[test]
    fn tracks_lines_and_skips_unicode_whitespace() {
        assert_eq!(
            Scanner::new("\u{2003}9\n ;\n\t").scan(),
            vec![
                Token::new(Number(9.), 1, "9"),
                Token::new(Semicolon, 2, ";"),
                Token::new(Eof, 3, "")
            ]
        );
    }

    #[test]
    fn handles_empty_input_and_number_at_end() {
        assert_eq!(Scanner::new("").scan(), vec![Token::new(Eof, 1, "")]);
        assert_eq!(Scanner::new(" \n\t").scan(), vec![Token::new(Eof, 2, "")]);
        assert_eq!(
            Scanner::new("99").scan(),
            vec![Token::new(Number(99.), 1, "99"), Token::new(Eof, 1, "")]
        );
    }
}
