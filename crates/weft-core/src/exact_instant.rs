// SPDX-License-Identifier: MIT OR Apache-2.0
//! Consumer binding of UMF CONTRACT-065; generic timestamp and Key stay unchanged.
use crate::error::{Diagnostic, Result};
pub(crate) const PROFILE: &str = "umf-rfc3339-exact-instant/1.0.0";
pub(crate) fn key(token: &str) -> Result<(i64, &str)> {
    let fail = || Diagnostic::new("WFT-TIMESTAMP-DOMAIN", "type", "Selected RFC3339 exact instant token required");
    let bytes = token.as_bytes();
    if bytes.len() > 1048576 { return Err(Diagnostic::new("WFT-LIMIT", "type", "Timestamp token exceeds one MiB")); }
    if bytes.len() < 20 || !bytes.is_ascii() || bytes[4] != b'-' || bytes[7] != b'-'
        || !matches!(bytes[10], b'T' | b't') || bytes[13] != b':' || bytes[16] != b':' { return Err(fail()); }
    let number = |start: usize, end: usize| -> Result<i64> {
        let value = &bytes[start..end];
        if !value.iter().all(u8::is_ascii_digit) { return Err(fail()); }
        Ok(value.iter().fold(0, |sum, digit| sum * 10 + i64::from(digit - b'0')))
    };
    let year = number(0,4)?; let month = number(5,7)?; let day = number(8,10)?;
    let hour = number(11,13)?; let minute = number(14,16)?; let second = number(17,19)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let lengths = [31, if leap {29} else {28}, 31,30,31,30,31,31,30,31,30,31];
    if year == 0 || !(1..=12).contains(&month) || day < 1 || day > lengths[(month-1) as usize]
        || hour > 23 || minute > 59 || second > 59 { return Err(fail()); }
    let mut position = 19;
    let mut fraction = "";
    if bytes[position] == b'.' {
        position += 1; let start = position;
        while position < bytes.len() && bytes[position].is_ascii_digit() { position += 1; }
        if position == start { return Err(fail()); }
        fraction = token[start..position].trim_end_matches('0');
    }
    if position >= bytes.len() { return Err(fail()); }
    let offset = if matches!(bytes[position], b'Z' | b'z') {
        if position + 1 != bytes.len() { return Err(fail()); } 0
    } else {
        if position + 6 != bytes.len() || !matches!(bytes[position], b'+' | b'-') || bytes[position+3] != b':' { return Err(fail()); }
        let hours = number(position+1,position+3)?; let minutes = number(position+4,position+6)?;
        if hours > 23 || minutes > 59 { return Err(fail()); }
        (hours * 60 + minutes) * if bytes[position] == b'-' {-1} else {1}
    };
    let before = |y: i64| { let n = y - 1; n * 365 + n / 4 - n / 100 + n / 400 };
    let days = before(year) - before(1970) + lengths[..(month-1) as usize].iter().sum::<i64>() + day - 1;
    Ok((days * 86400 + hour * 3600 + minute * 60 + second - offset * 60, fraction))
}

#[cfg(test)]
mod tests {
    use super::key;
    #[test] fn exact_instants() {
        assert_eq!(key("2024-01-01T00:00:00.123456789Z").unwrap(), key("2023-12-31T19:00:00.1234567890-05:00").unwrap());
        assert_ne!(key("2024-01-01T00:00:00.1234567Z").unwrap(), key("2024-01-01T00:00:00.1234568Z").unwrap());
        assert_eq!(key("1969-12-31t23:59:59.5000z").unwrap(), (-1,"5"));
        assert_eq!(key("1970-01-01T00:00:00-00:00").unwrap(), (0,""));
        assert_eq!(key("0001-01-01T00:00:00+23:59").unwrap().0, -62135683140);
        assert_eq!(key("9999-12-31T23:59:59-23:59").unwrap().0, 253402387139);
        for token in ["1900-02-29T00:00:00Z", "0000-01-01T00:00:00Z", "2016-12-31T23:59:60Z", "2024-01-01T00:00:00+24:00", "2024-01-01T00:00:00Z\n"] { assert!(key(token).is_err()); }
    }
}
