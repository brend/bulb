mod compilation;
mod scan;

use scan::{ScanError, Scanner};
use std::io::{self, Write};

use crate::compilation::{Compiler, ParseError};

#[derive(Debug)]
enum BulbError {
    Scan(ScanError),
    Parse(ParseError),
}

impl From<ScanError> for BulbError {
    fn from(value: ScanError) -> Self {
        BulbError::Scan(value)
    }
}

impl From<ParseError> for BulbError {
    fn from(value: ParseError) -> Self {
        BulbError::Parse(value)
    }
}

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
                    match error {
                        BulbError::Scan(error) => eprintln!("Scan error: {:?}", error),
                        BulbError::Parse(error) => eprintln!("Parse error: {:?}", error),
                    }
                }
            }
            Err(error) => eprintln!("Read error: {:?}", error),
        }
    }
}

fn run(source: &str) -> Result<(), BulbError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan()?;

    println!("=== TOKENS ===");
    for t in &tokens {
        println!("{:>4} {}", t.line(), t);
    }
    println!();

    let compiler = Compiler::new(tokens);
    let code = compiler.compile()?;

    println!("=== CODE ===");
    for inst in code {
        println!("{:>4} {}", inst.line(), inst);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_propagates_scan_errors() {
        assert!(matches!(
            run("return @"),
            Err(BulbError::Scan(ScanError::UnexpectedChar('@')))
        ));
        assert!(run("42").is_ok());
    }

    #[test]
    fn run_propagates_parse_errors() {
        assert!(matches!(
            run("42 +"),
            Err(BulbError::Parse(ParseError::UnexpectedToken))
        ));
        assert!(matches!(
            run(""),
            Err(BulbError::Parse(ParseError::UnexpectedEof))
        ));
    }
}
