// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded transport corpus for actual Python/WASM export parity. No native I/O.
use serde_json::{json, Value};
fn main() {
    let request: Value = serde_json::from_str(include_str!(
        "../../../tests/engine-catalog/temporal-request.json"
    ))
    .unwrap();
    let compile = |page: bool| {
        let mut packet = json!({"version":if page{"weft.engine-page/0.4"}else{"weft.engine-read/0.4"},"compiler":"truss-runtime-temporal","request":request});
        if page {
            packet["selectedKey"] = json!({"module":"m","record":"R","key":"pair"});
            packet["pageSize"] = json!(2)
        }
        packet.to_string()
    };
    let mut cases = vec![];
    for page in [false, true] {
        let packet = compile(page);
        let name = if page {
            "compile_engine_page_json"
        } else {
            "compile_engine_read_json"
        };
        let response = weft_runtime::engine_temporal::compile_json(&packet, page);
        let program: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(program["status"], "compiled", "{program}");
        cases.push(json!({"name":name,"arguments":[packet],"expected":program}));
        for shape in [true, false] {
            let name = match (page, shape) {
                (false, true) => "check_engine_temporal_read_shape_json",
                (true, true) => "check_engine_temporal_page_shape_json",
                (false, false) => "check_engine_temporal_read_json",
                (true, false) => "check_engine_temporal_page_json",
            };
            let mut current = request.clone();
            if !shape {
                current["temporalParameters"]["since"]["value"] = json!("1970-01-01T01:00:00+01:00")
            }
            let mut args = vec![program.to_string(), current.to_string()];
            if page && !shape {
                args.push("null".into())
            }
            let expected: Value = serde_json::from_str(&weft_runtime::engine_temporal::check_json(
                &args[0],
                &args[1],
                if args.len() == 3 {
                    Some(&args[2])
                } else {
                    None
                },
                page,
                shape,
            ))
            .unwrap();
            assert_eq!(expected["status"], "checked");
            cases.push(json!({"name":name,"arguments":args,"expected":expected}));
            let mut forged = program.clone();
            forged["artifact"]["sourceDomainChecks"]["sources"][0]["requirementsParameter"] =
                json!(1);
            args[0] = forged.to_string();
            let expected: Value = serde_json::from_str(&weft_runtime::engine_temporal::check_json(
                &args[0],
                &args[1],
                if args.len() == 3 {
                    Some(&args[2])
                } else {
                    None
                },
                page,
                shape,
            ))
            .unwrap();
            assert_eq!(expected["status"], "blocked");
            cases.push(json!({"name":name,"arguments":args,"expected":expected}));
        }
        let name = if page {
            "check_engine_temporal_page_shape_json"
        } else {
            "check_engine_temporal_read_shape_json"
        };
        let args = vec![
            serde_json::to_string_pretty(&program).unwrap(),
            serde_json::to_string_pretty(&request).unwrap(),
        ];
        let expected: Value = serde_json::from_str(&weft_runtime::engine_temporal::check_json(
            &args[0], &args[1], None, page, true,
        ))
        .unwrap();
        assert_eq!(expected["status"], "checked");
        cases.push(json!({"name":name,"arguments":args,"expected":expected}));
        if page {
            let mut after = json!({"version":"weft.engine-temporal-after/0.2","selectedKey":{"module":"m","record":"R","key":"pair"},"sortValues":[{"state":"present","value":"1969-12-31T23:59:59.500000"},{"state":"present","value":"2000-02-29"},{"state":"present","value":"alpha"},{"state":"present","value":"beta"}],"keyValues":["alpha","beta"]});
            for bad in [false, true] {
                if bad {
                    after["keyValues"][1] = json!("forged");
                }
                let args = vec![program.to_string(), request.to_string(), after.to_string()];
                let expected: Value =
                    serde_json::from_str(&weft_runtime::engine_temporal::check_json(
                        &args[0],
                        &args[1],
                        Some(&args[2]),
                        true,
                        false,
                    ))
                    .unwrap();
                assert_eq!(expected["status"], if bad { "blocked" } else { "checked" });
                cases.push(json!({"name":"check_engine_temporal_page_json","arguments":args,"expected":expected}));
            }
        }
    }
    println!(
        "{}",
        json!({"format":"weft.temporal-boundary-corpus/0.1","scope":"Rust-generated pure compiler expected results; not a source, wheel, native-execution or release receipt","cases":cases})
    );
}
