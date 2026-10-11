//! Actual explicit042 CLI transport; compiler/engine qualification is separate.
#![cfg(feature = "ashlar-databricks-implicit-arithmetic")]
use std::process::{Command, Stdio};
const LIMIT: usize = 16 * 1024 * 1024;
#[cfg(unix)]
#[test]
fn actual042_stdin_preserves_suffix_after_sentinel() {
    use std::io::{Seek, SeekFrom, Write};
    let path = std::env::temp_dir().join(format!("weft042-suffix-{}", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    file.write_all(&vec![b'x'; LIMIT + 65536]).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_weft-implicit-arithmetic"))
        .stdin(Stdio::from(file.try_clone().unwrap()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"WEFT_CLI_INPUT_LIMIT\n");
    assert_eq!(file.stream_position().unwrap(), (LIMIT + 1) as u64);
}
#[cfg(unix)]
#[test]
fn actual042_stdin_io_error_is_payload_free() {
    let output = Command::new(env!("CARGO_BIN_EXE_weft-implicit-arithmetic"))
        .stdin(Stdio::from(std::fs::File::open(".").unwrap()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"WEFT_CLI_INPUT_IO\n");
}
