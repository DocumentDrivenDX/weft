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
/// Original-property custody, still awaiting its selected semantic decoder.
#[derive(Debug)]
pub struct SelectedObservation<'a> {
    pub observation: Observation,
    pub presence_bytes: &'a [u8],
}
/// Derive family/codec from the original property; observations cannot select
/// their own logical type. Physical absence remains uninterpreted here.
pub fn admit_property<'a>(
    property: &'a crate::property_definition::PropertyAdmission,
    access: &crate::registered_access::Access<'_>,
    cells: &[Option<&str>],
    budget: &mut Budget,
) -> Result<SelectedObservation<'a>> {
    access.verify_property(property)?;
    if !matches!(access.location, crate::registered_access::Location::Row(_))
        || !matches!(
            property.home,
            crate::property_definition::HomeAdmission::Row { .. }
        )
    {
        return Err(fail(
            "Native scalar observation requires its original row home",
        ));
    }
    // Checks graph custody and full descriptor closure, without granting a codec.
    crate::result_definition::property_column(property, 1, "weft_native_custody")?;
    let descriptor = property
        .value
        .descriptors()
        .iter()
        .find(|descriptor| descriptor.identity == property.identity)
        .ok_or_else(|| fail("Native observation lacks original root descriptor"))?;
    let weft_core::application_model::Shape::Scalar { logical_type } = &descriptor.shape else {
        return Err(fail(
            "Compound native observation requires its selected tree decoder",
        ));
    };
    let codec = property
        .value
        .graph
        .artifacts
        .get(&format!(
            "/nodes/{}/codecDefinition",
            property.value.graph.root,
        ))
        .ok_or_else(|| fail("Native observation lacks original root codec bytes"))?;
    let observation = admit(cells, logical_type.family.clone(), codec, budget)?;
    Ok(SelectedObservation {
        observation,
        presence_bytes: property.value.presence.original_json.as_bytes(),
    })
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
/// Private complete-state row. Native text and decoded byte custody coexist;
/// topology, selected codecs and source meanings must be admitted separately.
#[derive(Debug, PartialEq, Eq)]
pub struct TreeRow {
    pub cells: [Option<String>; 23],
    pub bytes: std::collections::BTreeMap<usize, Vec<u8>>,
}
/// Admit a host-framed 23-cell bag with a whole-input allocation reservation.
/// Duplicate/orphan rows remain intact for the subsequent structural gate.
pub fn admit_tree_rows(rows: &[Vec<Option<&str>>], budget: &mut Budget) -> Result<Vec<TreeRow>> {
    const BYTE_CELLS: [usize; 7] = [7, 8, 9, 19, 20, 21, 22];
    if rows.iter().any(|row| row.len() != 23) {
        return Err(fail("Native tree custody arity differs"));
    }
    let cell_count = rows
        .len()
        .checked_mul(23)
        .ok_or_else(|| fail("Native tree cell count overflow"))?;
    let mut byte_count = 0usize;
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let length = cell.map_or(0, str::len);
            byte_count = byte_count
                .checked_add(length)
                .and_then(|n| n.checked_add(length))
                .and_then(|n| {
                    n.checked_add(if BYTE_CELLS.contains(&i) {
                        length / 2
                    } else {
                        0
                    })
                })
                .ok_or_else(|| fail("Native tree custody size overflow"))?;
        }
    }
    if cell_count > budget.remaining_cells || byte_count > budget.remaining_bytes {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Native tree custody reservation exhausted",
        ));
    }
    budget.remaining_cells -= cell_count;
    budget.remaining_bytes -= byte_count;
    let mut admitted = Vec::with_capacity(rows.len());
    for row in rows {
        for i in [0, 1, 2, 5, 10, 11] {
            if let Some(value) = row[i] {
                let parsed = value
                    .parse::<i64>()
                    .map_err(|_| fail("Native tree integer custody is invalid"))?;
                if parsed.to_string() != value {
                    return Err(fail("Native tree integer custody is noncanonical"));
                }
            }
        }
        let mut bytes = std::collections::BTreeMap::new();
        for i in BYTE_CELLS {
            if let Some(raw) = row[i] {
                bytes.insert(i, hex(raw)?);
            }
        }
        let cells = std::array::from_fn(|i| row[i].map(str::to_owned));
        admitted.push(TreeRow { cells, bytes });
    }
    Ok(admitted)
}
/// Physical complete-tree index. Original UMF shape/identity/codec admission
/// remains required before interpreting a node as a logical value.
pub struct TreeIndex<'a> {
    pub root: &'a TreeRow,
    pub children: std::collections::BTreeMap<&'a str, Vec<&'a TreeRow>>,
}
pub struct TreeBudget {
    pub remaining_nodes: usize,
    pub max_depth: usize,
}
pub fn index_tree<'a>(
    rows: &'a [TreeRow],
    state: &str,
    root: &str,
    budget: &mut TreeBudget,
) -> Result<TreeIndex<'a>> {
    use std::collections::{BTreeMap, BTreeSet};
    if rows.len() > budget.remaining_nodes {
        return Err(Diagnostic::new(
            "WFT-LIMIT",
            "decode",
            "Native tree index budget exhausted",
        ));
    }
    budget.remaining_nodes -= rows.len();
    let mut nodes = BTreeMap::new();
    for row in rows {
        for i in [0, 1, 2, 5, 10, 11] {
            if let Some(value) = &row.cells[i] {
                let native = value
                    .parse::<i64>()
                    .map_err(|_| fail("Native tree index integer differs"))?;
                if native.to_string() != *value {
                    return Err(fail("Native tree index integer is noncanonical"));
                }
            }
        }
        if row.cells[0].as_deref() != Some(state) {
            return Err(fail(
                "Native tree contains an orphan payload or foreign state",
            ));
        }
        let id = row.cells[1]
            .as_deref()
            .ok_or_else(|| fail("Native tree node ID missing"))?;
        if nodes.insert(id, row).is_some() {
            return Err(fail("Native tree repeats a node/payload occurrence"));
        }
        if row.cells[8].is_none() || row.cells[9].is_none() {
            return Err(fail("Native tree lacks original definition/source custody"));
        }
        let kind = row.cells[3]
            .as_deref()
            .ok_or_else(|| fail("Native tree value kind missing"))?;
        if !matches!(
            kind,
            "scalar" | "null" | "sequence" | "map" | "structured" | "record"
        ) {
            return Err(fail("Native tree value kind unsupported"));
        }
        if kind == "scalar" {
            if row.cells[10].as_deref() != Some(state)
                || row.cells[11].as_deref() != Some(id)
                || row.cells[12].is_none()
                || row.cells[21].is_none()
                || row.cells[22].is_none()
            {
                return Err(fail("Native scalar payload ownership/custody differs"));
            }
        } else if row.cells[10..].iter().any(Option::is_some) {
            return Err(fail("Native container/null node retains scalar payload"));
        }
    }
    let root_row = *nodes
        .get(root)
        .ok_or_else(|| fail("Native tree root missing"))?;
    let mut children: BTreeMap<&str, Vec<&TreeRow>> = BTreeMap::new();
    for (id, row) in &nodes {
        let slot = row.cells[4].as_deref();
        if *id == root {
            if row.cells[2].is_some()
                || slot != Some("root")
                || row.cells[5..8].iter().any(Option::is_some)
            {
                return Err(fail("Native root slot differs"));
            }
            continue;
        }
        let parent = row.cells[2]
            .as_deref()
            .ok_or_else(|| fail("Native nonroot has no parent"))?;
        let parent_row = nodes
            .get(parent)
            .ok_or_else(|| fail("Native node parent missing"))?;
        let valid = match parent_row.cells[3].as_deref() {
            Some("sequence") => {
                slot == Some("sequence")
                    && row.cells[5]
                        .as_deref()
                        .is_some_and(|v| v.parse::<u64>().is_ok())
                    && row.cells[6].is_none()
                    && row.cells[7].is_none()
            }
            Some("map") => {
                slot == Some("map")
                    && row.cells[5].is_none()
                    && row.cells[6].is_some()
                    && row.cells[7].is_none()
            }
            Some("structured" | "record") => {
                slot == Some("record")
                    && row.cells[5].is_none()
                    && row.cells[6].is_none()
                    && row.cells[7].is_some()
            }
            _ => false,
        };
        if !valid {
            return Err(fail("Native child slot conflicts with parent shape"));
        }
        children.entry(parent).or_default().push(row);
    }
    for (parent, children) in &mut children {
        let kind = nodes[parent].cells[3].as_deref();
        if kind == Some("sequence") {
            children.sort_by_key(|row| row.cells[5].as_ref().unwrap().parse::<u64>().unwrap());
            if children
                .iter()
                .enumerate()
                .any(|(i, row)| row.cells[5].as_ref().unwrap().parse::<u64>().unwrap() != i as u64)
            {
                return Err(fail("Native sequence ordinals are not unique and dense"));
            }
        } else {
            let index = if kind == Some("map") { 6 } else { 7 };
            let mut keys = BTreeSet::new();
            if children
                .iter()
                .any(|row| !keys.insert(row.cells[index].as_deref().unwrap()))
            {
                return Err(fail("Native container repeats a member slot"));
            }
        }
    }
    let mut pending = vec![(root, 0usize)];
    let mut reached = BTreeSet::new();
    while let Some((id, depth)) = pending.pop() {
        if depth > budget.max_depth {
            return Err(Diagnostic::new(
                "WFT-LIMIT",
                "decode",
                "Native tree depth exhausted",
            ));
        }
        if !reached.insert(id) {
            return Err(fail("Native tree cycle detected"));
        }
        if let Some(kids) = children.get(id) {
            for row in kids {
                pending.push((row.cells[1].as_deref().unwrap(), depth + 1));
            }
        }
    }
    if reached.len() != nodes.len() {
        return Err(fail(
            "Native tree contains unreachable nodes or a disconnected cycle",
        ));
    }
    Ok(TreeIndex {
        root: root_row,
        children,
    })
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
    fn tree_index_refuses_corrupt_links_slots_cycles_and_work_bounds() {
        let receipt: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/helix/04-build/evidence/B-005-row-tree-custody-native.json"
        ))
        .unwrap();
        let case = receipt["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["case"] == "descendant")
            .unwrap();
        let raw: Vec<Vec<Option<&str>>> = case["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_array().unwrap().iter().map(|v| v.as_str()).collect())
            .collect();
        for (cell, value) in [
            (0, "2"),
            (1, "10"),
            (2, "999"),
            (4, "map"),
            (5, "1"),
            (5, "01"),
            (10, "2"),
            (11, "999"),
        ] {
            let mut rows = admit_tree_rows(&raw, &mut budget()).unwrap();
            let child = rows
                .iter_mut()
                .find(|r| r.cells[1].as_deref() == Some("11"))
                .unwrap();
            child.cells[cell] = Some(value.into());
            assert!(index_tree(
                &rows,
                "1",
                "10",
                &mut TreeBudget {
                    remaining_nodes: 100,
                    max_depth: 32
                }
            )
            .is_err());
        }
        let mut rows = admit_tree_rows(&raw, &mut budget()).unwrap();
        let child = rows
            .iter_mut()
            .find(|r| r.cells[1].as_deref() == Some("11"))
            .unwrap();
        child.cells[2] = Some("11".into());
        child.cells[3] = Some("sequence".into());
        for cell in &mut child.cells[10..] {
            *cell = None;
        }
        assert!(index_tree(
            &rows,
            "1",
            "10",
            &mut TreeBudget {
                remaining_nodes: 100,
                max_depth: 32
            }
        )
        .is_err());
        let rows = admit_tree_rows(&raw, &mut budget()).unwrap();
        let mut limited = TreeBudget {
            remaining_nodes: 1,
            max_depth: 32,
        };
        assert_eq!(
            index_tree(&rows, "1", "10", &mut limited)
                .err()
                .unwrap()
                .code,
            "WFT-LIMIT"
        );
        assert_eq!(limited.remaining_nodes, 1);
        assert_eq!(
            index_tree(
                &rows,
                "1",
                "10",
                &mut TreeBudget {
                    remaining_nodes: 2,
                    max_depth: 0
                }
            )
            .err()
            .unwrap()
            .code,
            "WFT-LIMIT"
        );
    }
    #[test]
    fn native_bags_require_complete_unique_reachable_tree_structure() {
        let receipt: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/helix/04-build/evidence/B-005-row-tree-custody-native.json"
        ))
        .unwrap();
        for case in receipt["results"].as_array().unwrap() {
            let rows: Vec<Vec<Option<&str>>> = case["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_array().unwrap().iter().map(|v| v.as_str()).collect())
                .collect();
            let admitted = admit_tree_rows(&rows, &mut budget()).unwrap();
            let result = index_tree(
                &admitted,
                "1",
                "10",
                &mut TreeBudget {
                    remaining_nodes: 100,
                    max_depth: 32,
                },
            );
            assert_eq!(
                result.is_ok(),
                matches!(case["case"].as_str().unwrap(), "container" | "descendant")
            );
        }
    }
    #[test]
    fn admits_all_ten_recorded_complete_state_native_bags() {
        let receipt: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/helix/04-build/evidence/B-005-row-tree-custody-native.json"
        ))
        .unwrap();
        let cases = receipt["results"].as_array().unwrap();
        assert_eq!(cases.len(), 10);
        for case in cases {
            let rows: Vec<Vec<Option<&str>>> = case["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row.as_array().unwrap().iter().map(|v| v.as_str()).collect())
                .collect();
            let admitted = admit_tree_rows(&rows, &mut budget()).unwrap();
            assert_eq!(
                admitted.len(),
                case["observedRows"].as_u64().unwrap() as usize
            );
            for (original, row) in rows.iter().zip(admitted) {
                for (i, cell) in original.iter().enumerate() {
                    assert_eq!(row.cells[i].as_deref(), *cell);
                }
            }
        }
    }
    #[test]
    fn tree_rows_reserve_whole_input_and_preserve_unknown_and_orphan_custody() {
        let mut row = vec![None; 23];
        row[10] = Some("1");
        row[11] = Some("9007199254740993");
        row[12] = Some("future-kind");
        row[13] = Some("");
        row[21] = Some("00ff");
        let rows = vec![row.clone(), row.clone()];
        let mut work = Budget {
            remaining_bytes: 4096,
            remaining_cells: 46,
        };
        let admitted = admit_tree_rows(&rows, &mut work).unwrap();
        assert_eq!(admitted.len(), 2);
        assert_eq!(work.remaining_cells, 0);
        assert_eq!(admitted[0].cells[1], None);
        assert_eq!(admitted[0].cells[13], Some("".into()));
        assert_eq!(admitted[0].bytes[&21], vec![0, 255]);
        assert_eq!(admitted[0], admitted[1]);
        let mut limited = Budget {
            remaining_bytes: 4096,
            remaining_cells: 45,
        };
        assert_eq!(
            admit_tree_rows(&rows, &mut limited).unwrap_err().code,
            "WFT-LIMIT"
        );
        assert_eq!(limited.remaining_cells, 45);
        row[21] = Some("0A");
        let mut failed = Budget {
            remaining_bytes: 4096,
            remaining_cells: 23,
        };
        assert!(admit_tree_rows(&[row.clone()], &mut failed).is_err());
        assert_eq!(failed.remaining_cells, 0);
        row[21] = Some("00");
        for token in ["01", "+1", "9223372036854775808"] {
            row[11] = Some(token);
            assert!(admit_tree_rows(&[row.clone()], &mut budget()).is_err());
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
