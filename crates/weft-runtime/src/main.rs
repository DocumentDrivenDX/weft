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
    let response = weft_runtime::compile_json(&request);
    writeln!(output, "{response}").map_err(|_| "WEFT_CLI_OUTPUT_IO")?;
    output.flush().map_err(|_| "WEFT_CLI_OUTPUT_IO")
}

fn main() {
    if let Err(code) = run(io::stdin().lock(), io::stdout().lock()) {
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
