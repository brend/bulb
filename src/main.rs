fn main() {
    repl()
}

fn repl() {
    let mut input = String::new();

    loop {
        print!("> ");
        stdout().flush().unwrap();
        input.clear();

        match stdin().read_line(&mut input) {
            Ok(bytes_read) if bytes_read == 0 => return,
            Ok(_) => match run(&input) {
                Err(error) => eprintln!("Scan error: {:?}", error),
                _ => (),
            },
            Err(error) => eprintln!("Read error: {:?}", error),
        }
    }
}

fn run(source: &str) -> Result<(), ScanError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan()?;

    for t in tokens {
        println!("{:>4} {}", t.line, t);
    }

    Ok(())
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenType<'a> {
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

    Number(f64),
    Identifier(&'a str),

    If,
    Else,
    Fn,
    Return,

    Semicolon,
    Comma,
    Dot,

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

use std::{
    collections::HashMap,
    io::{Write, stdin, stdout},
    iter::Scan,
};

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
            Number(value) => write!(f, "NUM {}", value),
            Identifier(value) => write!(f, "IDENT {}", value),
            If => write!(f, "IF"),
            Else => write!(f, "ELSE"),
            Fn => write!(f, "FN"),
            Return => write!(f, "RETURN"),
            Semicolon => write!(f, "SEMICOLON"),
            Comma => write!(f, "COMMA"),
            Dot => write!(f, "DOT"),
            Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug)]
enum ScanError {
    UnexpectedChar(char),
    Io(std::io::Error),
}

impl From<std::io::Error> for ScanError {
    fn from(value: std::io::Error) -> Self {
        ScanError::Io(value)
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
        HashMap::from([("if", If), ("else", Else), ("fn", Fn), ("return", Return)])
    }

    fn scan(mut self) -> Result<Vec<Token<'a>>, ScanError> {
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
            .expect("Unable to parse number");

        self.make(Number(value))
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
            None => self.make(Identifier(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_comparison_operators_at_end_of_input() {
        for (source, typ) in [
            ("=", Equal),
            ("<", Less),
            ("<=", LessEqual),
            (">", Greater),
            (">=", GreaterEqual),
        ] {
            assert_eq!(
                Scanner::new(source)
                    .scan()
                    .expect("valid source should scan"),
                vec![Token::new(typ, 1, source), Token::new(Eof, 1, "")],
                "source: {source:?}"
            );
        }
    }

    #[test]
    fn comparison_lookahead_preserves_adjacent_tokens_and_whitespace() {
        assert_eq!(
            Scanner::new("<é>=2<==>\n= > =")
                .scan()
                .expect("valid source should scan"),
            vec![
                Token::new(Less, 1, "<"),
                Token::new(Identifier("é"), 1, "é"),
                Token::new(GreaterEqual, 1, ">="),
                Token::new(Number(2.), 1, "2"),
                Token::new(LessEqual, 1, "<="),
                Token::new(Equal, 1, "="),
                Token::new(Greater, 1, ">"),
                Token::new(Equal, 2, "="),
                Token::new(Greater, 2, ">"),
                Token::new(Equal, 2, "="),
                Token::new(Eof, 2, ""),
            ]
        );
    }

    #[test]
    fn recognizes_function_keywords_only_as_complete_lowercase_words() {
        assert_eq!(
            Scanner::new("fn return fn2 fn_ returnValue return_ Fn Return")
                .scan()
                .expect("valid source should scan"),
            vec![
                Token::new(Fn, 1, "fn"),
                Token::new(Return, 1, "return"),
                Token::new(Identifier("fn2"), 1, "fn2"),
                Token::new(Identifier("fn_"), 1, "fn_"),
                Token::new(Identifier("returnValue"), 1, "returnValue"),
                Token::new(Identifier("return_"), 1, "return_"),
                Token::new(Identifier("Fn"), 1, "Fn"),
                Token::new(Identifier("Return"), 1, "Return"),
                Token::new(Eof, 1, ""),
            ]
        );
    }

    #[test]
    fn displays_new_tokens_in_repl_output() {
        for (typ, expected) in [
            (Equal, "EQUAL"),
            (Less, "LESS"),
            (LessEqual, "LESS_EQUAL"),
            (Greater, "GREATER"),
            (GreaterEqual, "GREATER_EQUAL"),
            (Fn, "FN"),
            (Return, "RETURN"),
            (Comma, "COMMA"),
            (Dot, "DOT"),
        ] {
            assert_eq!(Token::new(typ, 1, "").to_string(), expected);
        }
    }

    #[test]
    fn unsupported_input_returns_the_first_unexpected_character() {
        for (source, expected) in [
            ("@", '@'),
            ("42;\n  @ !", '@'),
            ("éclair 💡", '💡'),
            ("<@", '@'),
            (">💡", '💡'),
            ("\0", '\0'),
        ] {
            assert!(
                matches!(Scanner::new(source).scan(),
                    Err(ScanError::UnexpectedChar(actual)) if actual == expected),
                "source: {source:?}"
            );
        }
    }

    #[test]
    fn run_propagates_scan_errors() {
        assert!(matches!(
            run("return @"),
            Err(ScanError::UnexpectedChar('@'))
        ));
        assert!(run("fn example() { return 42; }").is_ok());
    }

    #[test]
    fn scans_all_punctuation_without_whitespace() {
        assert_eq!(
            Scanner::new("+-*/(){};,.")
                .scan()
                .expect("valid source should scan"),
            vec![
                Token::new(Plus, 1, "+"),
                Token::new(Minus, 1, "-"),
                Token::new(Star, 1, "*"),
                Token::new(Slash, 1, "/"),
                Token::new(LeftParen, 1, "("),
                Token::new(RightParen, 1, ")"),
                Token::new(LeftBrace, 1, "{"),
                Token::new(RightBrace, 1, "}"),
                Token::new(Semicolon, 1, ";"),
                Token::new(Comma, 1, ","),
                Token::new(Dot, 1, "."),
                Token::new(Eof, 1, ""),
            ]
        );
    }

    #[test]
    fn recognizes_only_exact_lowercase_keywords() {
        assert_eq!(
            Scanner::new("if else iffy elsewhere if2 else_ If ELSE")
                .scan()
                .expect("valid source should scan"),
            vec![
                Token::new(If, 1, "if"),
                Token::new(Else, 1, "else"),
                Token::new(Identifier("iffy"), 1, "iffy"),
                Token::new(Identifier("elsewhere"), 1, "elsewhere"),
                Token::new(Identifier("if2"), 1, "if2"),
                Token::new(Identifier("else_"), 1, "else_"),
                Token::new(Identifier("If"), 1, "If"),
                Token::new(Identifier("ELSE"), 1, "ELSE"),
                Token::new(Eof, 1, ""),
            ]
        );
    }

    #[test]
    fn preserves_utf8_identifiers_and_byte_boundaries() {
        assert_eq!(
            Scanner::new("_x2+éclair;\n变量9")
                .scan()
                .expect("valid source should scan"),
            vec![
                Token::new(Identifier("_x2"), 1, "_x2"),
                Token::new(Plus, 1, "+"),
                Token::new(Identifier("éclair"), 1, "éclair"),
                Token::new(Semicolon, 1, ";"),
                Token::new(Identifier("变量9"), 2, "变量9"),
                Token::new(Eof, 2, ""),
            ]
        );
    }

    #[test]
    fn scans_sample_with_exact_lexemes() {
        let source = String::from(" 23; 47; 69 ; 67;88;99;");
        let tokens = Scanner::new(&source)
            .scan()
            .expect("valid source should scan");
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
            Scanner::new("\u{2003}9\n ;\n\t")
                .scan()
                .expect("valid source should scan"),
            vec![
                Token::new(Number(9.), 1, "9"),
                Token::new(Semicolon, 2, ";"),
                Token::new(Eof, 3, "")
            ]
        );
    }

    #[test]
    fn handles_empty_input_and_number_at_end() {
        assert_eq!(
            Scanner::new("").scan().expect("valid source should scan"),
            vec![Token::new(Eof, 1, "")]
        );
        assert_eq!(
            Scanner::new(" \n\t")
                .scan()
                .expect("valid source should scan"),
            vec![Token::new(Eof, 2, "")]
        );
        assert_eq!(
            Scanner::new("99").scan().expect("valid source should scan"),
            vec![Token::new(Number(99.), 1, "99"), Token::new(Eof, 1, "")]
        );
    }
}
