// SPDX-License-Identifier: MIT OR Apache-2.0
//! Finite proleptic Gregorian dates and exact microsecond UTC instants.
//! No ambient timezone, locale, floating point or rounding participates.
type Result<T> = std::result::Result<T, &'static str>;
fn leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}
fn days(y: i32, m: i32) -> i32 {
    match m {
        2 => {
            if leap(y) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
fn number(s: &str) -> Result<i32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("temporal numeric component differs");
    }
    s.parse().map_err(|_| "temporal component exceeds capacity")
}
fn components(s: &str) -> Result<(i32, i32, i32)> {
    if s.len() != 10 || !s.is_ascii() || &s[4..5] != "-" || &s[7..8] != "-" {
        return Err("finite ISO date required");
    }
    let (y, m, d) = (number(&s[..4])?, number(&s[5..7])?, number(&s[8..])?);
    if !(1..=9999).contains(&y) || !(1..=12).contains(&m) || !(1..=days(y, m)).contains(&d) {
        return Err("Gregorian date outside native profile");
    }
    Ok((y, m, d))
}
pub(super) fn date(s: &str) -> Result<String> {
    components(s)?;
    Ok(s.into())
}
pub(super) fn instant(s: &str) -> Result<String> {
    if !(20..=128).contains(&s.len())
        || !s.is_ascii()
        || &s[10..11] != "T"
        || &s[13..14] != ":"
        || &s[16..17] != ":"
    {
        return Err("explicit ISO instant required");
    }
    let (mut y, mut m, mut d) = components(&s[..10])?;
    let (h, min, sec) = (
        number(&s[11..13])?,
        number(&s[14..16])?,
        number(&s[17..19])?,
    );
    if h > 23 || min > 59 || sec > 59 {
        return Err("clock outside native instant profile");
    }
    let mut rest = &s[19..];
    let mut fraction = String::new();
    if rest.starts_with('.') {
        let n = rest[1..].bytes().take_while(|b| b.is_ascii_digit()).count();
        if n == 0 {
            return Err("empty instant fraction");
        }
        let digits = &rest[1..=n];
        if n > 6 && !digits[6..].bytes().all(|b| b == b'0') {
            return Err("instant would round at native microseconds");
        }
        fraction.push_str(&digits[..n.min(6)]);
        rest = &rest[n + 1..];
    }
    while fraction.len() < 6 {
        fraction.push('0');
    }
    let offset = if rest == "Z" {
        0
    } else {
        if rest.len() != 6 || !matches!(&rest[..1], "+" | "-") || &rest[3..4] != ":" {
            return Err("explicit instant offset required");
        }
        let (hours, minutes) = (number(&rest[1..3])?, number(&rest[4..])?);
        if hours > 23 || minutes > 59 || rest == "-00:00" {
            return Err("unknown or invalid instant offset");
        }
        (hours * 3600 + minutes * 60) * if rest.starts_with('-') { -1 } else { 1 }
    };
    let mut seconds = h * 3600 + min * 60 + sec - offset;
    if seconds < 0 {
        seconds += 86400;
        d -= 1;
        if d == 0 {
            m -= 1;
            if m == 0 {
                y -= 1;
                m = 12;
            }
            d = days(y, m);
        }
    } else if seconds >= 86400 {
        seconds -= 86400;
        d += 1;
        if d > days(y, m) {
            d = 1;
            m += 1;
            if m == 13 {
                y += 1;
                m = 1;
            }
        }
    }
    if !(1..=9999).contains(&y) {
        return Err("UTC instant outside finite native year profile");
    }
    Ok(format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{fraction}Z",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_finite_temporal_boundaries() {
        for s in ["0001-01-01", "2000-02-29", "9999-12-31"] {
            assert_eq!(date(s).unwrap(), s);
        }
        for s in [
            "0000-01-01",
            "1900-02-29",
            "2000-02-30",
            "infinity",
            "2026-1-01",
            "2026-01-01x",
        ] {
            assert!(date(s).is_err(), "{s}");
        }
        assert_eq!(
            instant("2026-01-01T00:30:00.123456000+01:00").unwrap(),
            "2025-12-31T23:30:00.123456Z"
        );
        assert_eq!(
            instant("2000-02-28T23:30:00-01:00").unwrap(),
            "2000-02-29T00:30:00.000000Z"
        );
        assert_eq!(
            instant("0001-01-01T00:00:00Z").unwrap(),
            "0001-01-01T00:00:00.000000Z"
        );
        for s in [
            "0001-01-01T00:00:00+00:01",
            "9999-12-31T23:59:59-00:01",
            "2026-01-01T00:00:00.0000001Z",
            "2026-01-01T00:00:00-00:00",
            "2026-01-01T00:00:60Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:00",
            "2026-01-01T00:00:00+24:00",
        ] {
            assert!(instant(s).is_err(), "{s}");
        }
    }
}
