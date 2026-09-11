use super::{ScanError, Scanner, Token, TokenType::*};

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
            Token::new(Identifier, 1, "é"),
            Token::new(GreaterEqual, 1, ">="),
            Token::new(Number, 1, "2"),
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
            Token::new(Identifier, 1, "fn2"),
            Token::new(Identifier, 1, "fn_"),
            Token::new(Identifier, 1, "returnValue"),
            Token::new(Identifier, 1, "return_"),
            Token::new(Identifier, 1, "Fn"),
            Token::new(Identifier, 1, "Return"),
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
            Token::new(Identifier, 1, "iffy"),
            Token::new(Identifier, 1, "elsewhere"),
            Token::new(Identifier, 1, "if2"),
            Token::new(Identifier, 1, "else_"),
            Token::new(Identifier, 1, "If"),
            Token::new(Identifier, 1, "ELSE"),
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
            Token::new(Identifier, 1, "_x2"),
            Token::new(Plus, 1, "+"),
            Token::new(Identifier, 1, "éclair"),
            Token::new(Semicolon, 1, ";"),
            Token::new(Identifier, 2, "变量9"),
            Token::new(Eof, 2, ""),
        ]
    );
}

#[test]
fn scans_sample_with_exact_lexemes() {
    let source = std::string::String::from(" 23; 47; 69 ; 67;88;99;");
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
    for token in tokens[..12].iter().step_by(2) {
        assert_eq!(token.typ(), Number);
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
            Token::new(Number, 1, "9"),
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
        vec![Token::new(Number, 1, "99"), Token::new(Eof, 1, "")]
    );
}

#[test]
fn scans_strings_with_exact_quoted_lexemes_at_end_of_input() {
    for source in [
        r#""""#,
        r#""hello world""#,
        "\" \t\u{2003} \"",
        r#""éclair 变量 💡""#,
        r#""if return 42; @ +-*/(){}<=>,.""#,
        r#""literal\n\t""#,
    ] {
        assert_eq!(
            Scanner::new(source).scan().expect("string should scan"),
            vec![Token::new(String, 1, source), Token::new(Eof, 1, "")],
            "source: {source:?}"
        );
    }
}

#[test]
fn strings_preserve_adjacent_strings_and_other_tokens() {
    assert_eq!(
        Scanner::new("return\"é\"\"💡\"+42;name")
            .scan()
            .expect("adjacent tokens should scan"),
        vec![
            Token::new(Return, 1, "return"),
            Token::new(String, 1, "\"é\""),
            Token::new(String, 1, "\"💡\""),
            Token::new(Plus, 1, "+"),
            Token::new(Number, 1, "42"),
            Token::new(Semicolon, 1, ";"),
            Token::new(Identifier, 1, "name"),
            Token::new(Eof, 1, ""),
        ]
    );
}

#[test]
fn rejects_unterminated_strings_at_end_of_input() {
    for source in [
        "\"",
        "\"hello",
        "\"é💡",
        "return \"oops",
        "\"a\nb",
        "\"ok\"\"",
    ] {
        assert!(
            matches!(
                Scanner::new(source).scan(),
                Err(ScanError::UnterminatedString)
            ),
            "source: {source:?}"
        );
    }
}

#[test]
fn tracks_lines_after_multiline_strings() {
    let tokens = Scanner::new("\n\"first\n第二\n💡\";\n42")
        .scan()
        .expect("multiline string should scan");
    assert_eq!(tokens[0].typ(), String);
    assert_eq!(tokens[0].lexeme(), "\"first\n第二\n💡\"");
    assert_eq!(
        &tokens[1..],
        &[
            Token::new(Semicolon, 4, ";"),
            Token::new(Number, 5, "42"),
            Token::new(Eof, 5, ""),
        ]
    );
}

#[test]
fn reports_unexpected_characters_after_a_closed_string() {
    assert!(matches!(
        Scanner::new("\"@ is allowed inside\"@").scan(),
        Err(ScanError::UnexpectedChar('@'))
    ));
}

#[test]
fn displays_string_tokens_with_their_quoted_lexemes() {
    let tokens = Scanner::new("\"\" \"é 💡\"").scan().unwrap();
    assert_eq!(tokens[0].to_string(), "STRING \"\"");
    assert_eq!(tokens[1].to_string(), "STRING \"é 💡\"");
    assert_eq!(
        ScanError::UnterminatedString.to_string(),
        "Unterminated string"
    );
}
