use std::io::{self, Read, Write};

const MAX_REQUEST_BYTES: usize = weft_core::compile::MAX_REQUEST_BYTES;

fn read_request(input: impl Read) -> Result<String, &'static str> {
    let mut input = input;
    let mut bytes = Vec::with_capacity(MAX_REQUEST_BYTES + 1);
    let mut chunk = [0u8; 8192];
    loop {
        let available = (MAX_REQUEST_BYTES + 1 - bytes.len()).min(chunk.len());
        match input.read(&mut chunk[..available]) {
            Ok(0) => break,
            Ok(count) => {
                bytes.extend_from_slice(&chunk[..count]);
                if bytes.len() > MAX_REQUEST_BYTES { return Err("WEFT_CLI_INPUT_LIMIT"); }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("WEFT_CLI_INPUT_IO"),
        }
    }
    String::from_utf8(bytes).map_err(|_| "WEFT_CLI_UTF8")
}

fn run(input: impl Read, mut output: impl Write) -> Result<(), &'static str> {
    let request = read_request(input)?;
    let response = weft_runtime::compile_json(&request);
    writeln!(output, "{response}").map_err(|_| "WEFT_CLI_OUTPUT_IO")?;
    output.flush().map_err(|_| "WEFT_CLI_OUTPUT_IO")
}

// OS-handle reads avoid StdinLock read-ahead consuming the untrusted suffix.
#[cfg(unix)]
fn stdin_file() -> io::Result<std::fs::File> {
    use std::os::fd::AsFd;
    io::stdin().as_fd().try_clone_to_owned().map(std::fs::File::from)
}
#[cfg(windows)]
fn stdin_file() -> io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle;
    io::stdin().as_handle().try_clone_to_owned().map(std::fs::File::from)
}
#[cfg(not(any(unix, windows)))]
fn stdin_file() -> io::Result<std::fs::File> { Err(io::ErrorKind::Unsupported.into()) }
fn main() {
    let result = stdin_file().map_err(|_| "WEFT_CLI_INPUT_IO")
        .and_then(|input| run(input, io::stdout().lock()));
    if let Err(code) = result {
        let _ = writeln!(io::stderr().lock(), "{code}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn exact_byte_boundary_and_multibyte_text() {
        let mut bytes = vec![b' '; MAX_REQUEST_BYTES - 2];
        bytes.extend_from_slice("é".as_bytes());
        assert_eq!(
            read_request(Cursor::new(&bytes)).unwrap().len(),
            MAX_REQUEST_BYTES
        );
        bytes.push(b' ');
        assert_eq!(
            read_request(Cursor::new(&bytes)),
            Err("WEFT_CLI_INPUT_LIMIT")
        );
        assert_eq!(read_request(Cursor::new(&[0xc3])), Err("WEFT_CLI_UTF8"));
    }

    #[test]
    fn sentinel_bounds_read_even_when_more_input_is_available() {
        struct Endless {
            observed: usize,
        }
        impl Read for &mut Endless {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                out.fill(b' ');
                self.observed += out.len();
                Ok(out.len())
            }
        }
        let mut input = Endless { observed: 0 };
        assert_eq!(read_request(&mut input), Err("WEFT_CLI_INPUT_LIMIT"));
        assert_eq!(input.observed, MAX_REQUEST_BYTES + 1);
    }

    #[test]
    fn input_errors_emit_no_response_and_output_errors_are_fatal() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "private payload"))
            }
        }
        let mut output = Vec::new();
        assert_eq!(run(Broken, &mut output), Err("WEFT_CLI_INPUT_IO"));
        assert!(output.is_empty());
        assert_eq!(run(Cursor::new([0xff]), &mut output), Err("WEFT_CLI_UTF8"));
        assert!(output.is_empty());
        struct Closed;
        impl Write for Closed {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert_eq!(run(Cursor::new(b"{}"), Closed), Err("WEFT_CLI_OUTPUT_IO"));
    }

    #[test]
    fn ordinary_response_retains_exact_compiler_bytes_and_one_newline() {
        let request = "{}";
        let mut output = Vec::new();
        run(Cursor::new(request), &mut output).unwrap();
        assert_eq!(
            output,
            format!("{}\n", weft_runtime::compile_json(request)).as_bytes()
        );
    }
}
