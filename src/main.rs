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

    #[test]
    fn run_propagates_scan_errors() {
        assert!(matches!(
            run("return @"),
            Err(ScanError::UnexpectedChar('@'))
        ));
        assert!(run("fn example() { return 42; }").is_ok());
    }
}
