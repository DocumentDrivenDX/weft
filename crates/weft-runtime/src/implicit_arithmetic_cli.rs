//! One-shot, explicitly feature-selected 0.4 compiler transport.
//! This executable does not execute emitted SQL or discharge host obligations.
use std::io::{self, Read, Write};

const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

fn read_request(input: impl Read) -> Result<String, &'static str> {
    let mut bytes = Vec::new();
    input
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "WEFT_CLI_INPUT_IO")?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err("WEFT_CLI_INPUT_LIMIT");
    }
    String::from_utf8(bytes).map_err(|_| "WEFT_CLI_UTF8")
}

fn run(input: impl Read, mut output: impl Write) -> Result<(), &'static str> {
    let request = read_request(input)?;
    let response = weft_runtime::implicit_arithmetic::compile_json(&request);
    writeln!(output, "{response}").map_err(|_| "WEFT_CLI_OUTPUT_IO")?;
    output.flush().map_err(|_| "WEFT_CLI_OUTPUT_IO")
}

#[cfg(unix)]
fn stdin_file() -> io::Result<std::fs::File> {
    use std::os::fd::AsFd;
    io::stdin()
        .as_fd()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}
#[cfg(windows)]
fn stdin_file() -> io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle;
    io::stdin()
        .as_handle()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}
#[cfg(not(any(unix, windows)))]
fn stdin_file() -> io::Result<std::fs::File> {
    Err(io::ErrorKind::Unsupported.into())
}
fn main() {
    let result = stdin_file()
        .map_err(|_| "WEFT_CLI_INPUT_IO")
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
    fn exact_utf8_byte_boundary_and_original_whitespace_are_preserved() {
        let mut bytes = vec![b' '; MAX_REQUEST_BYTES - 2];
        bytes.extend_from_slice("é".as_bytes());
        assert_eq!(read_request(Cursor::new(&bytes)).unwrap().as_bytes(), bytes);
        bytes.push(b' ');
        assert_eq!(
            read_request(Cursor::new(&bytes)),
            Err("WEFT_CLI_INPUT_LIMIT")
        );
        assert_eq!(read_request(Cursor::new([0xc3])), Err("WEFT_CLI_UTF8"));
        let raw = " \r\n{\"untouched\":\"é\\n\"}\t\n";
        assert_eq!(read_request(Cursor::new(raw)).unwrap(), raw);
    }

    #[test]
    fn sentinel_bounds_the_read_without_draining_unbounded_input() {
        struct Endless(usize);
        impl Read for &mut Endless {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                out.fill(b' ');
                self.0 += out.len();
                Ok(out.len())
            }
        }
        let mut input = Endless(0);
        assert_eq!(read_request(&mut input), Err("WEFT_CLI_INPUT_LIMIT"));
        assert_eq!(input.0, MAX_REQUEST_BYTES + 1);
    }

    #[test]
    fn input_errors_publish_no_artifact_and_error_payloads_stay_private() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("private input error payload"))
            }
        }
        let mut output = Vec::new();
        assert_eq!(run(Broken, &mut output), Err("WEFT_CLI_INPUT_IO"));
        assert!(output.is_empty());
        assert_eq!(run(Cursor::new([0xff]), &mut output), Err("WEFT_CLI_UTF8"));
        assert!(output.is_empty());
    }

    #[test]
    fn write_and_flush_failure_are_fatal_transport_errors() {
        struct Closed(bool);
        impl Write for Closed {
            fn write(&mut self, data: &[u8]) -> io::Result<usize> {
                if self.0 {
                    Ok(data.len())
                } else {
                    Err(io::ErrorKind::BrokenPipe.into())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("private flush error payload"))
            }
        }
        for allow_write in [false, true] {
            assert_eq!(
                run(Cursor::new(b"{}"), Closed(allow_write)),
                Err("WEFT_CLI_OUTPUT_IO")
            );
        }
    }

    #[test]
    fn compiler_response_bytes_and_exactly_one_newline_are_preserved() {
        for raw in [
            "{}",
            "{",
            " \r\n{}\t\n",
            "{\"interfaceVersion\":\"weft-compile/0.3.0\",\"dialect\":\"weft-sql/0.3.0\"}",
        ] {
            let mut output = Vec::new();
            run(Cursor::new(raw), &mut output).unwrap();
            assert_eq!(
                output,
                format!("{}\n", weft_runtime::implicit_arithmetic::compile_json(raw)).as_bytes()
            );
        }
    }
}
