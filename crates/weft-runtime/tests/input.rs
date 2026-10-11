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
fn transport_failure(output: &std::process::Output, code: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, format!("{code}\n").as_bytes());
}
#[test]
fn one_over_limit_refuses_without_artifact() {
    transport_failure(&cli(&vec![b' '; MAX_REQUEST_BYTES + 1]), "WEFT_CLI_INPUT_LIMIT");
}
#[test]
fn malformed_utf8_refuses_at_transport_boundary() {
    transport_failure(&cli(&[0xff]), "WEFT_CLI_UTF8");
}
#[cfg(unix)]
#[test]
fn actual_stdin_read_error_is_safe_and_nonzero() {
    let input = std::fs::File::open(".").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_weft-runtime"))
        .stdin(Stdio::from(input)).stdout(Stdio::piped()).stderr(Stdio::piped())
        .output().unwrap();
    transport_failure(&output, "WEFT_CLI_INPUT_IO");
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
        assert_eq!(result.status.code(), Some(2));
        if !broken_stderr {
            assert_eq!(
                result.stderr,
                b"WEFT_CLI_OUTPUT_IO\n"
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
    transport_failure(&output, "WEFT_CLI_INPUT_LIMIT");
    assert_eq!(
        file.stream_position().unwrap(),
        (MAX_REQUEST_BYTES + 1) as u64
    );
    let mut suffix = Vec::new();
    file.read_to_end(&mut suffix).unwrap();
    assert_eq!(suffix.len(), 65535);
}
