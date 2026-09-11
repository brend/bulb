use crate::scan::Token;
use crate::scan::TokenType::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    Const(f64),
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Const(value) => write!(f, "CON {}", value),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
    line: usize,
}

impl Instruction {
    fn new(opcode: Opcode, line: usize) -> Instruction {
        Instruction { opcode, line }
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<4} {}", self.line(), self.opcode)
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken,
    InvalidNumericLiteral,
    UnexpectedEof,
}

pub struct Compiler<'a> {
    tokens: Vec<Token<'a>>,
    instructions: Vec<Instruction>,
    current: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            instructions: vec![],
            current: 0,
        }
    }

    pub fn compile(mut self) -> Result<Vec<Instruction>, ParseError> {
        self.expression()?;
        if let Some(token) = self.peek() && token.typ() != Eof {
            return Err(ParseError::UnexpectedToken);
        }
        Ok(self.instructions)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.current)
    }

    fn consume(&mut self) -> Option<Token<'a>> {
        if self.is_at_end() {
            None
        } else {
            let token = self.tokens[self.current];
            self.current += 1;
            Some(token)
        }
    }

    fn expression(&mut self) -> Result<(), ParseError> {
        match self.consume() {
            None => Err(ParseError::UnexpectedToken),
            Some(token) => match token.typ() {
                Number => self.number(&token),
                Eof => Err(ParseError::UnexpectedEof),
                _ => Err(ParseError::UnexpectedToken),
            },
        }
    }

    fn number(&mut self, token: &Token<'a>) -> Result<(), ParseError> {
        match token.lexeme().parse::<f64>() {
            Ok(value) => {
                self.emit(Opcode::Const(value), token);
                Ok(())
            }
            _ => Err(ParseError::InvalidNumericLiteral),
        }
    }

    fn emit(&mut self, opcode: Opcode, token: &Token<'a>) {
        let inst = Instruction::new(opcode, token.line());
        self.instructions.push(inst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::Scanner;

    fn compile(source: &str) -> Result<Vec<Instruction>, ParseError> {
        Compiler::new(Scanner::new(source).scan().unwrap()).compile()
    }

    #[test]
    fn compiles_a_number_with_its_source_line() {
        let code = compile("\n 42\n").unwrap();
        assert_eq!(code.len(), 1);
        assert_eq!(code[0].opcode, Opcode::Const(42.0));
        assert_eq!(code[0].line(), 2);
    }

    #[test]
    fn compiles_the_entire_decimal_value() {
        let code = compile("1.5").unwrap();
        assert_eq!(code.len(), 1);
        assert_eq!(code[0].opcode, Opcode::Const(1.5));
    }

    #[test]
    fn rejects_adjacent_numbers_instead_of_discarding_the_second() {
        assert!(matches!(compile("42 99"), Err(ParseError::UnexpectedToken)));
    }

    #[test]
    fn rejects_an_incomplete_expression_instead_of_compiling_its_prefix() {
        assert!(matches!(compile("42 +"), Err(ParseError::UnexpectedToken)));
    }

    #[test]
    fn empty_source_returns_a_parse_error() {
        assert!(matches!(compile(""), Err(ParseError::UnexpectedEof)));
    }

    #[test]
    fn unexpected_token_returns_a_parse_error() {
        assert!(matches!(compile(")"), Err(ParseError::UnexpectedToken)));
    }

    #[test]
    fn rejects_numeric_overflow_instead_of_emitting_infinity() {
        let source = "9".repeat(400);
        assert!(matches!(
            compile(&source),
            Err(ParseError::InvalidNumericLiteral)
        ));
    }

    #[test]
    fn malformed_numeric_token_returns_a_parse_error() {
        let tokens = vec![Token::new(Number, 1, "invalid"), Token::new(Eof, 1, "")];
        assert!(matches!(
            Compiler::new(tokens).compile(),
            Err(ParseError::InvalidNumericLiteral)
        ));
    }
}
