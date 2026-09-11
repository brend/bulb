mod compilation;
mod scan;

use scan::{ScanError, Scanner};
use std::io::{self, Write};

use crate::compilation::Compiler;

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

    println!("=== TOKENS ===");
    for t in &tokens {
        println!("{:>4} {}", t.line(), t);
    }
    println!();

    let compiler = Compiler::new(tokens);
    let code = compiler.compile().unwrap();

    println!("=== CODE ===");
    for inst in code {
        println!("{:>4} {:?}", inst.line(), inst);
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
            Err(ScanError::UnexpectedChar('@'))
        ));
        assert!(run("fn example() { return 42; }").is_ok());
    }
}
