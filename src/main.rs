fn main() {
    let source = " 23; 47; 69 ; 67;88;99;";
    let scanner = Scanner::new(&source);
    let tokens = scanner.scan();

    for t in tokens {
        println!("{:>4} {}", t.line, t);
    }
}

#[derive(Debug, PartialEq)]
enum TokenType {
    Number(f64),
    Semicolon,
    Eof,
}

#[derive(Debug, PartialEq)]
struct Token<'a> {
    typ: TokenType,
    line: usize,
    lexeme: &'a str,
}

impl<'a> Token<'a> {
    fn new(typ: TokenType, line: usize, lexeme: &'a str) -> Token<'a> {
        Token { typ, line, lexeme }
    }
}

use TokenType::*;

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            Number(value) => write!(f, "NUM {}", value),
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
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Scanner<'a> {
        Scanner { 
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan(mut self) -> Vec<Token<'a>> {
        let mut tokens = vec![];

        loop {
            self.skip_whitespace();
            if self.is_at_end() { break; }
            tokens.push(self.token());
            if self.is_at_end() { break; }
        }

        if !self.is_at_end() {
            panic!();
        }

        tokens.push(self.make(Eof));

        return tokens;
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_whitespace() {
            if self.consume() == '\n' {
                self.line += 1;
                self.start = self.current;
            }
        }
    }

    fn token(&mut self) -> Token {
        match self.consume() {
            ';' => self.make(Semicolon),
            '0'..'9' => self.number(),
            _ => unreachable!(),
        }
    }

    fn peek(&self) -> char {
        match self.source[self.current..].chars().next() {
            None => '\0',
            Some(c) => c,
        }
    }

    fn consume(&mut self) -> char {
        let c = self.peek();
        self.current += 1;
        c
    }

    fn is_at_end(&self) -> bool {
        self.peek() == '\0'
    }

    fn make(&mut self, typ: TokenType) -> Token {
        let lexeme = &self.source[self.start..self.current];
        let token = Token::new(typ, self.line, lexeme);
        self.start = self.current;
        token
    }

    fn number(&mut self) -> Token {
        while self.peek().is_digit(10) {
            self.consume();
        }

        let value = self.source[self.start..self.current].parse::<f64>().unwrap();

        self.make(Number(value))
    }
}