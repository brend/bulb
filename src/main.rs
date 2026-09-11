mod scan;

use scan::{ScanError, Scanner};
use std::io::{self, Write};

fn main() {
    repl()
}

fn repl() {
    let mut input = String::new();
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();

        match stdin.read_line(&mut input) {
            Ok(0) => return,
            Ok(_) => {
                if let Err(error) = run(&input) {
                    eprintln!("Scan error: {:?}", error)
                }
            }
            Err(error) => eprintln!("Read error: {:?}", error),
        }
    }
}

fn run(source: &str) -> Result<(), ScanError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan()?;

    for t in tokens {
        println!("{:>4} {}", t.line(), t);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use scan::Token;
    use scan::TokenType::*;

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
            tokens.iter().map(|t| t.lexeme()).collect::<Vec<_>>(),
            expected
        );
        for (token, value) in tokens.iter().step_by(2).zip([23., 47., 69., 67., 88., 99.]) {
            assert_eq!(token.typ(), Number(value));
            assert_eq!(token.line(), 1);
        }
        for token in tokens[..12].iter().skip(1).step_by(2) {
            assert_eq!(token.typ(), Semicolon);
        }
        assert_eq!(tokens.last().unwrap().typ(), Eof);
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
