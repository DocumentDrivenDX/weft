use std::io::{self, Read, Write};
use weft_core::compile::MAX_REQUEST_BYTES;

#[derive(Debug, PartialEq)]
enum InputFailure {
    Limit,
    Utf8,
    Io,
}
impl InputFailure {
    fn response(&self) -> &'static str {
        match self {
            Self::Limit => {
                r#"{"diagnostics":[{"code":"WFT-LIMIT","message":"Request exceeds sixteen MiB","phase":"input","recoverability":"correct-input","severity":"error"}],"interfaceVersion":"weft-compile/0.1.0","status":"blocked"}"#
            }
            Self::Utf8 => {
                r#"{"diagnostics":[{"code":"WFT-UTF8","message":"Invalid UTF-8 request","phase":"input","recoverability":"correct-input","severity":"error"}],"interfaceVersion":"weft-compile/0.1.0","status":"blocked"}"#
            }
            Self::Io => {
                r#"{"diagnostics":[{"code":"WFT-IO","message":"Request input could not be read","phase":"host","recoverability":"host-action","severity":"error"}],"interfaceVersion":"weft-compile/0.1.0","status":"blocked"}"#
            }
        }
    }
}
fn read_request(mut reader: impl Read) -> Result<String, InputFailure> {
    // The sentinel lets overflow refuse without draining an attacker-owned stream.
    // Capacity is fixed before reading and cannot grow beyond the admitted bytes.
    let mut bytes = Vec::with_capacity(MAX_REQUEST_BYTES + 1);
    let mut chunk = [0u8; 8192];
    loop {
        let available = (MAX_REQUEST_BYTES + 1 - bytes.len()).min(chunk.len());
        match reader.read(&mut chunk[..available]) {
            Ok(0) => break,
            Ok(count) => {
                bytes.extend_from_slice(&chunk[..count]);
                if bytes.len() > MAX_REQUEST_BYTES {
                    return Err(InputFailure::Limit);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(InputFailure::Io),
        }
    }
    String::from_utf8(bytes).map_err(|_| InputFailure::Utf8)
}
// Clone the OS handle rather than using StdinLock's read-ahead buffer.
#[cfg(unix)]
fn read_stdin() -> Result<String, InputFailure> {
    use std::os::fd::AsFd;
    let handle = io::stdin()
        .as_fd()
        .try_clone_to_owned()
        .map_err(|_| InputFailure::Io)?;
    read_request(std::fs::File::from(handle))
}
#[cfg(windows)]
fn read_stdin() -> Result<String, InputFailure> {
    use std::os::windows::io::AsHandle;
    let handle = io::stdin()
        .as_handle()
        .try_clone_to_owned()
        .map_err(|_| InputFailure::Io)?;
    read_request(std::fs::File::from(handle))
}
#[cfg(not(any(unix, windows)))]
fn read_stdin() -> Result<String, InputFailure> {
    Err(InputFailure::Io)
}
fn main() {
    let request = read_stdin();
    let (response, code) = match request {
        Ok(request) => (weft_runtime::compile_json(&request), 0),
        Err(failure) => (
            failure.response().to_owned(),
            if failure == InputFailure::Io { 1 } else { 0 },
        ),
    };
    let mut output = io::stdout().lock();
    if output
        .write_all(response.as_bytes())
        .and_then(|_| output.write_all(b"\n"))
        .and_then(|_| output.flush())
        .is_err()
    {
        let _ = io::stderr()
            .lock()
            .write_all(b"WFT-IO: response output could not be written\n");
        std::process::exit(1);
    }
    std::process::exit(code);
}
#[cfg(test)]
mod tests {
    use super::*;
    // @covers US-008-AC2
    #[test]
    fn exact_limit_and_overflow_are_bounded_before_utf8() {
        for length in [
            MAX_REQUEST_BYTES - 1,
            MAX_REQUEST_BYTES,
            MAX_REQUEST_BYTES + 1,
        ] {
            let bytes = vec![b' '; length];
            let result = read_request(bytes.as_slice());
            if length <= MAX_REQUEST_BYTES {
                let request = result.unwrap();
                assert_eq!(request.len(), length);
                assert!(request.capacity() <= MAX_REQUEST_BYTES + 1);
            } else {
                assert_eq!(result.unwrap_err(), InputFailure::Limit);
            }
        }
    }
    struct Endless {
        bytes: usize,
    }
    impl Read for Endless {
        fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
            target.fill(b'x');
            self.bytes += target.len();
            Ok(target.len())
        }
    }
    #[test]
    fn overflow_does_not_read_the_untrusted_suffix() {
        let mut source = Endless { bytes: 0 };
        assert_eq!(read_request(&mut source).unwrap_err(), InputFailure::Limit);
        assert_eq!(source.bytes, MAX_REQUEST_BYTES + 1);
    }
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("secret-shaped input must not escape"))
        }
    }
    #[test]
    fn utf8_and_read_failures_return_safe_atomic_refusals() {
        assert_eq!(
            read_request([0xff].as_slice()).unwrap_err(),
            InputFailure::Utf8
        );
        let error = read_request(Broken).unwrap_err();
        assert_eq!(error, InputFailure::Io);
        assert!(!error.response().contains("secret-shaped"));
        for error in [InputFailure::Utf8, InputFailure::Io, InputFailure::Limit] {
            let response: serde_json::Value = serde_json::from_str(error.response()).unwrap();
            assert_eq!(response["status"], "blocked");
            assert!(response.get("sql").is_none());
        }
    }
}
