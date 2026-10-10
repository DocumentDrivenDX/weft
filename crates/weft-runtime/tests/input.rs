//! @covers US-008-AC2 Exercise the actual CLI transport, not only its reader.
use std::io::Write;
use std::process::{Command, Stdio};
use weft_core::compile::MAX_REQUEST_BYTES;
fn cli(bytes: &[u8]) -> std::process::Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_weft-runtime"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = process.stdin.take().unwrap();
    input.write_all(bytes).unwrap();
    drop(input);
    process.wait_with_output().unwrap()
}
fn response(output: &std::process::Output, code: &str) -> serde_json::Value {
    assert!(output.stderr.is_empty());
    assert!(output.stdout.ends_with(b"\n"));
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["status"], "blocked");
    assert_eq!(response["diagnostics"][0]["code"], code);
    assert!(response.get("sql").is_none());
    response
}
#[test]
fn exact_limit_reaches_normal_compilation() {
    let mut raw = vec![b' '; MAX_REQUEST_BYTES];
    raw[..2].copy_from_slice(b"{}");
    let output = cli(&raw);
    assert!(output.status.success());
    response(&output, "WFT-INPUT");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        weft_runtime::compile_json("{}") + "\n"
    );
}
#[test]
fn one_over_limit_refuses_without_artifact() {
    let output = cli(&vec![b' '; MAX_REQUEST_BYTES + 1]);
    assert!(output.status.success());
    let observed = response(&output, "WFT-LIMIT");
    let direct: serde_json::Value = serde_json::from_str(&weft_runtime::compile_json(
        &" ".repeat(MAX_REQUEST_BYTES + 1),
    ))
    .unwrap();
    assert_eq!(observed, direct);
}
#[test]
fn malformed_utf8_refuses_at_transport_boundary() {
    let output = cli(&[0xff]);
    assert!(output.status.success());
    response(&output, "WFT-UTF8");
}
#[cfg(unix)]
#[test]
fn actual_stdin_read_error_is_safe_and_nonzero() {
    // A directory FD can be passed as stdin but cannot be read as a byte stream.
    let input = std::fs::File::open(".").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_weft-runtime"))
        .stdin(Stdio::from(input))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let observed = response(&output, "WFT-IO");
    assert_eq!(
        observed["diagnostics"][0]["message"],
        "Request input could not be read"
    );
    assert_eq!(observed["diagnostics"][0]["recoverability"], "host-action");
}
#[cfg(unix)]
#[test]
fn output_failure_stays_nonzero_even_when_stderr_is_unwritable() {
    for broken_stderr in [false, true] {
        let mut process = Command::new(env!("CARGO_BIN_EXE_weft-runtime"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Keep stdin open until readers are closed, so the child cannot emit early.
        drop(process.stdout.take());
        if broken_stderr {
            drop(process.stderr.take());
        }
        drop(process.stdin.take());
        let result = process.wait_with_output().unwrap();
        assert_eq!(result.status.code(), Some(1));
        if !broken_stderr {
            assert_eq!(
                result.stderr,
                b"WFT-IO: response output could not be written\n"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn actual_stdin_preserves_suffix_after_one_sentinel_byte() {
    use std::io::{Read, Seek, SeekFrom};
    let path = std::env::temp_dir().join(format!("weft-r1-suffix-{}", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    file.write_all(&vec![b'x'; MAX_REQUEST_BYTES + 65536])
        .unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_weft-runtime"))
        .stdin(Stdio::from(file.try_clone().unwrap()))
        .output()
        .unwrap();
    assert!(output.status.success());
    response(&output, "WFT-LIMIT");
    assert_eq!(
        file.stream_position().unwrap(),
        (MAX_REQUEST_BYTES + 1) as u64
    );
    let mut suffix = Vec::new();
    file.read_to_end(&mut suffix).unwrap();
    assert_eq!(suffix.len(), 65535);
}
