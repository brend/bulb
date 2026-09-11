use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn assert_exits_after_stdin_closes(input: &[u8]) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bulb"))
        .stdin(Stdio::piped())
        // Discard output so the EOF bug cannot fill a pipe or consume unbounded memory.
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not start bulb");

    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let write_result = stdin.write_all(input);
    drop(stdin);

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait().expect("could not poll bulb") {
            write_result.expect("could not write input to bulb");
            assert!(status.success(), "bulb exited unsuccessfully: {status}");
            return;
        }
        if Instant::now() >= deadline {
            // Always kill and reap the looping process before failing the test.
            child
                .kill()
                .expect("could not terminate bulb after timeout");
            child.wait().expect("could not reap bulb after timeout");
            panic!("bulb did not exit within two seconds after stdin closed");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn exits_when_stdin_is_empty() {
    assert_exits_after_stdin_closes(b"");
}

#[test]
fn exits_after_a_final_line_without_a_newline() {
    assert_exits_after_stdin_closes(b"if (answer2 + 42) { else; }");
}
