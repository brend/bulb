use std::io::{Read, Write};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn run_repl(input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bulb"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("could not start bulb");

    // Drain both pipes concurrently, with a cap in case the EOF loop regresses.
    // Closing a pipe at the cap also prevents unbounded output from a buggy child.
    fn capture(stream: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            stream.take(64 * 1024).read_to_end(&mut bytes).unwrap();
            bytes
        })
    }
    let stdout = capture(child.stdout.take().unwrap());
    let stderr = capture(child.stderr.take().unwrap());

    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let write_result = stdin.write_all(input);
    drop(stdin);

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait().expect("could not poll bulb") {
            write_result.expect("could not write input to bulb");
            return Output {
                status,
                stdout: stdout.join().expect("stdout reader panicked"),
                stderr: stderr.join().expect("stderr reader panicked"),
            };
        }
        if Instant::now() >= deadline {
            // Always kill and reap the looping process before failing the test.
            child
                .kill()
                .expect("could not terminate bulb after timeout");
            child.wait().expect("could not reap bulb after timeout");
            stdout.join().expect("stdout reader panicked");
            stderr.join().expect("stderr reader panicked");
            panic!("bulb did not exit within two seconds after stdin closed");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn assert_exits_after_stdin_closes(input: &[u8]) {
    let output = run_repl(input);
    assert!(
        output.status.success(),
        "bulb exited unsuccessfully: {:?}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn code_listing_prints_each_instruction_line_number_once() {
    let output = run_repl(b"42\n");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let code = stdout
        .split_once("=== CODE ===\n")
        .expect("missing code listing")
        .1;
    let instruction = code.lines().next().expect("missing instruction");
    assert_eq!(
        instruction.split_whitespace().collect::<Vec<_>>(),
        ["1", "CON", "42"]
    );
}

#[cfg(unix)]
#[test]
fn permanent_stdin_error_is_reported_once_and_terminates_the_repl() {
    // Reading a directory reliably produces a permanent I/O error on Unix.
    let directory = std::fs::File::open(env!("CARGO_MANIFEST_DIR")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_bulb"))
        .stdin(Stdio::from(directory))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("could not start bulb");
    let mut stderr = child.stderr.take().unwrap();
    let reader = thread::spawn(move || {
        let mut captured = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = stderr.read(&mut buffer).unwrap();
            if count == 0 {
                return captured;
            }
            // Keep draining after the cap: closing the pipe could make the
            // buggy process panic and falsely appear to handle the read error.
            let keep = count.min((64 * 1024usize).saturating_sub(captured.len()));
            captured.extend_from_slice(&buffer[..keep]);
        }
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    let timed_out = loop {
        if child.try_wait().expect("could not poll bulb").is_some() {
            break false;
        }
        if Instant::now() >= deadline {
            child.kill().expect("could not terminate bulb after timeout");
            child.wait().expect("could not reap bulb after timeout");
            break true;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stderr = String::from_utf8(reader.join().expect("stderr reader panicked")).unwrap();
    assert!(!timed_out, "REPL kept retrying a permanent stdin error");
    assert!(!stderr.contains("panicked at"), "I/O errors must not panic");
    assert_eq!(stderr.matches("Read error:").count(), 1);
}

#[test]
fn exits_when_stdin_is_empty() {
    assert_exits_after_stdin_closes(b"");
}

#[test]
fn reports_parse_errors_and_compiles_the_next_line() {
    let output = run_repl(b")\n7\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "parse errors should not terminate the REPL: {stderr}"
    );
    assert!(
        stderr.contains("Parse error: UnexpectedToken"),
        "missing parse error diagnostic: {stderr}"
    );
    assert!(!stderr.contains("panicked at"), "parse errors must not panic");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("CON 7"), "next line was not compiled: {stdout}");
}

#[test]
fn exits_after_a_final_line_without_a_newline() {
    assert_exits_after_stdin_closes(b"42");
}

#[test]
fn reports_scan_errors_and_processes_the_next_line() {
    let output = run_repl(b"@\n42");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Scan error: UnexpectedChar('@')"),
        "missing scan error on stderr: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Scan error:"));
    let number = stdout.find("NUM 42").expect("next line was not processed");
    assert!(stdout[number..].contains("CON 42"));
    assert!(stdout[number..].contains("EOF"));
}

#[test]
fn blank_lines_do_not_end_the_repl() {
    let output = run_repl(b"\n \t\n7\n");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("NUM 7"),
        "blank lines must not be treated as EOF"
    );
}

#[test]
fn invalid_utf8_input_reports_an_io_error_and_processes_the_next_line() {
    let output = run_repl(b"\xff\n42");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the REPL should recover from invalid UTF-8 and exit successfully: {stderr}"
    );
    assert!(
        stderr.contains("Read error:"),
        "missing input error on stderr: {stderr}"
    );
    assert!(
        !stderr.contains("panicked at"),
        "input errors must not panic: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Read error:"));
    let number = stdout.find("NUM 42").expect("next line was not processed");
    assert!(stdout[number..].contains("CON 42"));
    assert!(stdout[number..].contains("EOF"));
}

#[test]
fn reports_unterminated_strings_and_compiles_the_next_line() {
    let output = run_repl(b"\"unfinished\n42\n");
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("Scan error: UnterminatedString"),
        "missing string error on stderr: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let number = stdout.find("NUM 42").expect("next line was not processed");
    assert!(stdout[number..].contains("EOF"));
    assert!(stdout[number..].contains("=== CODE ==="));
    assert!(stdout[number..].contains("CON 42"));
}
