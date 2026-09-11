use crate::scan::Token;
use crate::scan::TokenType::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    Const(f64),
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
    line: usize,
}

impl Instruction {
    fn new(opcode: Opcode, line: usize) -> Instruction {
        Instruction { 
            opcode,
            line 
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken,
    InvalidNumericLiteral,
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
                _ => todo!(),
            },
        }
    }

    fn number(&mut self, token: &Token<'a>) -> Result<(), ParseError> {
        match token.lexeme().parse::<f64>() {
            Ok(value) => Ok(self.emit(Opcode::Const(value), &token)),
            _ => Err(ParseError::InvalidNumericLiteral),
        }
    }

    fn emit(&mut self, opcode: Opcode, token: &Token<'a>) {
        let inst = Instruction::new(opcode, token.line());
        self.instructions.push(inst);
    }
}
