//! Admission of private native scalar observations, before semantic decoding.
use weft_core::{
    error::{Diagnostic, Result},
    ir::Family,
};
#[derive(Debug)]
pub struct Budget {
    pub remaining_bytes: usize,
    pub remaining_cells: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Payload {
    Text(String),
    Boolean(bool),
    Numeric {
        native_text: String,
        original_token: String,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Observation {
    /// Physical absence only; original presence meaning must decide legality.
    NoScalar,
    Scalar {
        payload: Payload,
        codec_bytes: Vec<u8>,
        source_bytes: Vec<u8>,
    },
}
fn fail(message: &str) -> Diagnostic {
    Diagnostic::new("WFT-DECODE", "decode", message)
}
fn hex(raw: &str) -> Result<Vec<u8>> {
    if raw.len() % 2 != 0
        || !raw
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(fail("Native byte custody is not canonical lowercase hex"));
    }
    let digit = |byte: u8| {
        if byte <= b'9' {
            byte - b'0'
        } else {
            byte - b'a' + 10
        }
    };
    Ok(raw
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| digit(pair[0]) * 16 + digit(pair[1]))
        .collect())
}
/// Eight cells in custody_projection order; SQL NULL is None, never empty text.
/// The host must separately admit framing, structural/payload preflight, complete
/// visibility and source/native codec procedures. Numeric text is not parsed here.
/// The full input/copy bound is charged before processing; failed work is charged.
pub fn admit(
    cells: &[Option<&str>],
    family: Family,
    expected_codec: &[u8],
    budget: &mut Budget,
) -> Result<Observation> {
    if cells.len() != 8 {
        return Err(fail("Native scalar custody arity differs"));
    }
    let mut bytes = 0usize;
    for (index, value) in cells.iter().enumerate() {
        let length = value.map_or(0, str::len);
        bytes = bytes
            .checked_add(length)
            .and_then(|sum| sum.checked_add(if index >= 6 { length / 2 } else { length }))
            .ok_or_else(|| fail("Native custody size overflow"))?;
    }
    if budget.remaining_cells < 8 || budget.remaining_bytes < bytes {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Native custody admission budget exhausted",
        ));
    }
    budget.remaining_cells -= 8;
    budget.remaining_bytes -= bytes;
    if cells[0] == Some("false") {
        if cells[1..].iter().any(Option::is_some) {
            return Err(fail("Absent scalar retains unexpected payload custody"));
        }
        return Ok(Observation::NoScalar);
    }
    if cells[0] != Some("true") {
        return Err(fail("Native scalar presence text differs"));
    }
    let kind = match family {
        Family::String => "string",
        Family::Boolean => "boolean",
        Family::Integer => "integer",
        Family::Decimal => "decimal",
    };
    if cells[1] != Some(kind) {
        return Err(fail("Native scalar kind differs from selected family"));
    }
    let required =
        |index: usize| cells[index].ok_or_else(|| fail("Native scalar custody cell is NULL"));
    let payload = match family {
        Family::String if cells[3..6].iter().all(Option::is_none) => {
            Payload::Text(required(2)?.into())
        }
        Family::Boolean if cells[2].is_none() && cells[4..6].iter().all(Option::is_none) => {
            Payload::Boolean(match required(3)? {
                "true" => true,
                "false" => false,
                _ => return Err(fail("Native boolean text differs")),
            })
        }
        Family::Integer | Family::Decimal if cells[2..4].iter().all(Option::is_none) => {
            Payload::Numeric {
                native_text: required(4)?.into(),
                original_token: required(5)?.into(),
            }
        }
        _ => return Err(fail("Native scalar custody has mixed payload slots")),
    };
    let codec_bytes = hex(required(6)?)?;
    if codec_bytes != expected_codec {
        return Err(fail(
            "Stored codec bytes differ from original selected codec",
        ));
    }
    let source_bytes = hex(required(7)?)?;
    Ok(Observation::Scalar {
        payload,
        codec_bytes,
        source_bytes,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn budget() -> Budget {
        Budget {
            remaining_bytes: 4096,
            remaining_cells: 80,
        }
    }
    #[test]
    fn preserves_native_original_and_null_empty_distinctions() {
        let numeric = [
            Some("true"),
            Some("decimal"),
            None,
            None,
            Some("0.00"),
            Some("-0.00"),
            Some("00ff"),
            Some("fe00"),
        ];
        assert_eq!(
            admit(&numeric, Family::Decimal, &[0, 255], &mut budget()).unwrap(),
            Observation::Scalar {
                payload: Payload::Numeric {
                    native_text: "0.00".into(),
                    original_token: "-0.00".into()
                },
                codec_bytes: vec![0, 255],
                source_bytes: vec![254, 0],
            }
        );
        let empty = [
            Some("true"),
            Some("string"),
            Some(""),
            None,
            None,
            None,
            Some(""),
            Some(""),
        ];
        assert!(
            matches!(admit(&empty,Family::String,&[],&mut budget()).unwrap(),Observation::Scalar { payload:Payload::Text(text),.. } if text.is_empty())
        );
        let absent = [Some("false"), None, None, None, None, None, None, None];
        assert_eq!(
            admit(&absent, Family::String, &[], &mut budget()).unwrap(),
            Observation::NoScalar
        );
    }
    #[test]
    fn rejects_corrupt_custody_without_granting_semantic_correspondence() {
        let valid = [
            Some("true"),
            Some("integer"),
            None,
            None,
            Some("2"),
            Some("1"),
            Some("00"),
            Some("ff"),
        ];
        assert!(admit(&valid, Family::Integer, &[0], &mut budget()).is_ok());
        for (index, value) in [
            (0, Some("t")),
            (1, Some("decimal")),
            (2, Some("")),
            (4, None),
            (5, None),
            (6, Some("0A")),
            (6, Some("0")),
            (7, Some("zz")),
            (7, None),
        ] {
            let mut corrupt = valid;
            corrupt[index] = value;
            assert!(admit(&corrupt, Family::Integer, &[0], &mut budget()).is_err());
        }
        assert!(admit(&valid, Family::Integer, &[1], &mut budget()).is_err());
        assert!(admit(&valid[..7], Family::Integer, &[0], &mut budget()).is_err());
    }
    #[test]
    fn admits_all_sixteen_recorded_native_custody_vectors() {
        let receipt: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/helix/04-build/evidence/B-005-row-custody-native.json"
        ))
        .unwrap();
        let observations = receipt["results"].as_array().unwrap();
        assert_eq!(observations.len(), 16);
        for observation in observations {
            let cells: Vec<_> = observation["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|cell| {
                    if cell.is_null() {
                        None
                    } else {
                        Some(cell.as_str().unwrap())
                    }
                })
                .collect();
            let family = match cells[1] {
                Some("string") | None => Family::String,
                Some("boolean") => Family::Boolean,
                Some("integer") => Family::Integer,
                Some("decimal") => Family::Decimal,
                _ => panic!("independent native fixture kind"),
            };
            let expected_codec: &[u8] = match observation["case"].as_str().unwrap() {
                "unicode" => &[0, 255],
                "empty" | "absent" => &[],
                _ => &[0],
            };
            let admitted = admit(&cells, family, expected_codec, &mut budget()).unwrap();
            match admitted {
                Observation::NoScalar => assert_eq!(observation["case"], "absent"),
                Observation::Scalar {
                    payload,
                    codec_bytes,
                    source_bytes,
                } => {
                    assert_eq!(codec_bytes, expected_codec);
                    assert_eq!(
                        source_bytes,
                        match observation["case"].as_str().unwrap() {
                            "unicode" => vec![254, 0],
                            "empty" => vec![],
                            _ => vec![255],
                        }
                    );
                    match payload {
                        Payload::Text(value) => assert_eq!(Some(value.as_str()), cells[2]),
                        Payload::Boolean(value) => {
                            assert_eq!(Some(if value { "true" } else { "false" }), cells[3])
                        }
                        Payload::Numeric {
                            native_text,
                            original_token,
                        } => {
                            assert_eq!(Some(native_text.as_str()), cells[4]);
                            assert_eq!(Some(original_token.as_str()), cells[5]);
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn reserves_whole_work_and_charges_failed_and_repeated_admissions() {
        let cells = [
            Some("true"),
            Some("boolean"),
            None,
            Some("false"),
            None,
            None,
            Some("00"),
            Some("ff"),
        ];
        let mut exhausted = Budget {
            remaining_bytes: 0,
            remaining_cells: 8,
        };
        assert_eq!(
            admit(&cells, Family::Boolean, &[0], &mut exhausted)
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
        assert_eq!(exhausted.remaining_cells, 8);
        let mut once = Budget {
            remaining_bytes: 4096,
            remaining_cells: 8,
        };
        assert!(admit(&cells, Family::Boolean, &[1], &mut once).is_err());
        assert_eq!(once.remaining_cells, 0);
        assert!(once.remaining_bytes < 4096);
        assert_eq!(
            admit(&cells, Family::Boolean, &[0], &mut once)
                .unwrap_err()
                .code,
            "WFT-LIMIT"
        );
    }
}
