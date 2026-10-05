//! B-001 experiment only: narrow projection/equality slice, not the public compiler.
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{json, value::RawValue, Value};
use sha2::{Digest, Sha256};
use sqlparser::{
    dialect::GenericDialect,
    parser::Parser,
    tokenizer::{Token, Tokenizer, Whitespace},
};
use std::{collections::BTreeSet, fmt};

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
type Result<T> = std::result::Result<T, &'static str>;

struct RawObject(Vec<(String, Box<RawValue>)>);
impl<'de> Deserialize<'de> for RawObject {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct ObjectVisitor;
        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = RawObject;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<M: MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, Box<RawValue>>()? {
                    entries.push(entry);
                }
                Ok(RawObject(entries))
            }
        }
        d.deserialize_map(ObjectVisitor)
    }
}
// Traverse raw nested values so even arbitrary-size unknown numbers are never f64.
fn inspect_json(raw: &str, depth: usize, count: &mut usize) -> Result<()> {
    if depth > 128 || *count >= 100_000 {
        return Err("WFT-LIMIT");
    }
    *count += 1;
    match raw.trim_start().as_bytes().first() {
        Some(b'{') => {
            let object: RawObject = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            let mut keys = BTreeSet::new();
            for (key, value) in object.0 {
                if !keys.insert(key) {
                    return Err("WFT-JSON-DUPLICATE");
                }
                inspect_json(value.get(), depth + 1, count)?;
            }
        }
        Some(b'[') => {
            let values: Vec<Box<RawValue>> = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
            for value in values {
                inspect_json(value.get(), depth + 1, count)?;
            }
        }
        _ => {
            let _: Value = serde_json::from_str(raw).map_err(|_| "WFT-INPUT")?;
        }
    }
    Ok(())
}
fn checked_json(raw: &str) -> Result<Value> {
    inspect_json(raw, 0, &mut 0)?;
    serde_json::from_str(raw).map_err(|_| "WFT-INPUT")
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key].as_str().ok_or("WFT-INPUT")
}
fn identifier(token: &Token) -> Result<(String, bool)> {
    if let Token::Word(w) = token {
        if let Some('"') = w.quote_style {
            return Ok((w.value.clone(), true));
        }
        if w.quote_style.is_some()
            || !w
                .value
                .bytes()
                .enumerate()
                .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
        {
            return Err("WFT-UNSUPPORTED");
        }
        if [
            "SELECT", "FROM", "WHERE", "AND", "AS", "JOIN", "SUM", "ORDER", "LIMIT",
        ]
        .contains(&w.value.to_uppercase().as_str())
        {
            return Err("WFT-UNSUPPORTED");
        }
        return Ok((w.value.to_ascii_lowercase(), false));
    }
    Err("WFT-UNSUPPORTED")
}
fn matches_name(name: &str, id: &(String, bool)) -> bool {
    if id.1 {
        name == id.0
    } else {
        name.to_ascii_lowercase() == id.0
    }
}
struct Cursor {
    tokens: Vec<Token>,
    index: usize,
}
impl Cursor {
    fn take(&mut self) -> Result<Token> {
        let token = self.tokens.get(self.index).cloned().ok_or("WFT-SYNTAX")?;
        self.index += 1;
        Ok(token)
    }
    fn expect(&mut self, t: Token) -> Result<()> {
        if self.take()? == t {
            Ok(())
        } else {
            Err("WFT-UNSUPPORTED")
        }
    }
    fn keyword(&mut self, word: &str) -> Result<()> {
        match self.take()? {
            Token::Word(w) if w.quote_style.is_none() && w.value.eq_ignore_ascii_case(word) => {
                Ok(())
            }
            _ => Err("WFT-UNSUPPORTED"),
        }
    }
    fn column(&mut self) -> Result<((String, bool), (String, bool))> {
        let a = identifier(&self.take()?)?;
        self.expect(Token::Period)?;
        Ok((a, identifier(&self.take()?)?))
    }
}
fn select_field<'a>(module: &'a Value, record: &Value, name: &(String, bool)) -> Result<&'a Value> {
    let elements = module["elements"].as_array().ok_or("WFT-MODEL")?;
    let members = record["members"].as_array().ok_or("WFT-MODEL")?;
    let fields: Vec<&Value> = elements
        .iter()
        .filter(|e| {
            matches_name(e["name"].as_str().unwrap_or(""), name)
                && members
                    .iter()
                    .any(|r| r["module"] == module["id"] && r["element"] == e["id"])
        })
        .collect();
    if fields.len() > 1 {
        return Err("WFT-NAME-AMBIGUOUS");
    }
    let field = *fields.first().ok_or("WFT-NAME-MISSING")?;
    if field["kind"] != "field"
        || field["cardinality"] != "one"
        || field["nullability"] != "required"
    {
        return Err("WFT-TYPE");
    }
    let family = field["scalarType"].as_str().ok_or("WFT-TYPE")?;
    if !["boolean", "string", "integer", "decimal"].contains(&family) {
        return Err("WFT-TYPE");
    }
    if field.get("recordType").is_some() || field.get("itemType").is_some() {
        return Err("WFT-TYPE");
    }
    if let Some(facets) = field.get("facets") {
        let object = facets.as_object().ok_or("WFT-TYPE")?;
        let allowed: &[&str] = match family {
            "integer" => &["integerWidth"],
            "decimal" => &["precision", "scale"],
            _ => &[],
        };
        if object.keys().any(|k| !allowed.contains(&k.as_str())) {
            return Err("WFT-TYPE");
        }
    }
    if family == "integer" {
        let width = &field["facets"]["integerWidth"];
        let bits = width["bits"].as_u64().ok_or("WFT-TYPE")?;
        if !(1..=64).contains(&bits)
            || !width["signed"].is_boolean()
            || width.as_object().map_or(true, |o| o.len() != 2)
        {
            return Err("WFT-TYPE");
        }
    }
    if family == "decimal" {
        let p = field["facets"]["precision"].as_u64().ok_or("WFT-TYPE")?;
        let s = field["facets"]["scale"].as_u64().ok_or("WFT-TYPE")?;
        if !(1..=28).contains(&p) || s > p {
            return Err("WFT-TYPE");
        }
    }
    Ok(field)
}
fn field_report(field: &Value) -> Value {
    json!({"element":field["id"],"family":field["scalarType"],"facets":field.get("facets").cloned().unwrap_or(json!({}))})
}
fn literal(cur: &mut Cursor, field: &Value) -> Result<Value> {
    let first = cur.take()?;
    let value = match first {
        Token::Minus => match cur.take()? {
            Token::Number(n, _) => format!("-{n}"),
            _ => return Err("WFT-UNSUPPORTED"),
        },
        Token::Number(n, false) => n,
        Token::SingleQuotedString(s) => {
            if field["scalarType"] != "string" || s.contains('\0') {
                return Err("WFT-TYPE");
            }
            return Ok(json!({"family":"string","value":s}));
        }
        Token::Word(w)
            if w.quote_style.is_none()
                && (w.value.eq_ignore_ascii_case("true")
                    || w.value.eq_ignore_ascii_case("false")) =>
        {
            if field["scalarType"] != "boolean" {
                return Err("WFT-TYPE");
            }
            return Ok(json!({"family":"boolean","value":w.value.to_ascii_lowercase()}));
        }
        _ => return Err("WFT-UNSUPPORTED"),
    };
    let unsigned = value.strip_prefix('-').unwrap_or(&value);
    if !unsigned.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return Err("WFT-UNSUPPORTED");
    }
    match field["scalarType"].as_str() {
        Some("integer") => {
            let n = value.parse::<i128>().map_err(|_| "WFT-NUMERIC-DOMAIN")?;
            let width = &field["facets"]["integerWidth"];
            let bits = width["bits"].as_u64().unwrap();
            let signed = width["signed"].as_bool().unwrap();
            let (min, max) = if signed {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            };
            if n < min || n > max {
                return Err("WFT-NUMERIC-DOMAIN");
            }
        }
        Some("decimal") => {
            let scale = field["facets"]["scale"].as_u64().unwrap() as usize;
            let precision = field["facets"]["precision"].as_u64().unwrap() as usize;
            let mut parts = unsigned.split('.');
            let whole = parts.next().unwrap();
            let frac = parts.next().unwrap_or("").trim_end_matches('0');
            if parts.next().is_some() || whole.is_empty() || frac.len() > scale {
                return Err("WFT-NUMERIC-DOMAIN");
            }
            let coeff = format!("{whole}{frac}{}", "0".repeat(scale - frac.len()));
            if coeff.trim_start_matches('0').len() > precision {
                return Err("WFT-NUMERIC-DOMAIN");
            }
        }
        _ => return Err("WFT-TYPE"),
    }
    Ok(json!({"family":field["scalarType"],"value":value}))
}
fn compile(request: &str) -> Result<Value> {
    if request.len() > 16 * 1024 * 1024 {
        return Err("WFT-LIMIT");
    }
    let req = checked_json(request)?;
    if req["interfaceVersion"] != "weft-compile/0.1.0" || req["dialect"] != "weft-sql/0.1.0" {
        return Err("WFT-VERSION");
    }
    let sql = text(&req, "sql")?;
    if sql.len() > 65536 {
        return Err("WFT-LIMIT");
    }
    let modules = req["modules"].as_array().ok_or("WFT-INPUT")?;
    if modules.len() != 1 {
        return Err("WFT-UNSUPPORTED");
    }
    let supplied = &modules[0];
    let raw = text(supplied, "documentJson")?;
    if raw.len() > 4 * 1024 * 1024 {
        return Err("WFT-LIMIT");
    }
    if supplied["pin"]["sha256"] != sha256(raw.as_bytes()) {
        return Err("WFT-PIN");
    }
    let doc = checked_json(raw)?;
    if doc["id"] != supplied["pin"]["documentId"] {
        return Err("WFT-PIN");
    }
    if doc["umf"] != "0.7.0" || supplied["pin"]["umfVersion"] != "0.7.0" {
        return Err("WFT-MODEL-VERSION");
    }
    let selected = supplied["selectedModuleIds"]
        .as_array()
        .ok_or("WFT-INPUT")?;
    if selected.len() != 1 {
        return Err("WFT-UNSUPPORTED");
    }
    let candidates: Vec<&Value> = doc["modules"]
        .as_array()
        .ok_or("WFT-MODEL")?
        .iter()
        .filter(|m| m["id"] == selected[0])
        .collect();
    if candidates.len() != 1 {
        return Err("WFT-MODEL");
    }
    let module = candidates[0];
    let binding = text(&req["target"], "bindingJson")?;
    if req["target"]["bindingSha256"] != sha256(binding.as_bytes()) {
        return Err("WFT-PIN");
    }
    let binding_value = checked_json(binding)?;
    if binding_value["fixtureOnly"] != true
        || req["target"]["backendId"] != "spike.synthetic"
        || req["options"]["allowCandidate"] != true
    {
        return Err("WFT-CAPABILITY");
    }
    let tokens = Tokenizer::new(&GenericDialect {}, sql)
        .tokenize()
        .map_err(|_| "WFT-SYNTAX")?;
    let mut filtered = Vec::new();
    for t in tokens {
        match t {
            Token::Whitespace(Whitespace::Space | Whitespace::Newline | Whitespace::Tab) => {}
            Token::Whitespace(_) => return Err("WFT-UNSUPPORTED"),
            _ => filtered.push(t),
        }
    }
    if filtered.len() > 4096 {
        return Err("WFT-LIMIT");
    }
    // Evaluate sqlparser independently, then narrow its accepted surface explicitly.
    let ast = Parser::parse_sql(&GenericDialect {}, sql).map_err(|_| "WFT-SYNTAX")?;
    if ast.len() != 1 {
        return Err("WFT-UNSUPPORTED");
    }
    let mut cur = Cursor {
        tokens: filtered,
        index: 0,
    };
    cur.keyword("SELECT")?;
    let (projection_alias, projection) = cur.column()?;
    cur.keyword("FROM")?;
    let relation = identifier(&cur.take()?)?;
    if cur
        .tokens
        .get(cur.index)
        .is_some_and(|t| matches!(t,Token::Word(w) if w.value.eq_ignore_ascii_case("AS")))
    {
        cur.keyword("AS")?;
    }
    let alias = identifier(&cur.take()?)?;
    if alias != projection_alias {
        return Err("WFT-NAME-MISSING");
    }
    let records: Vec<&Value> = module["elements"]
        .as_array()
        .ok_or("WFT-MODEL")?
        .iter()
        .filter(|e| {
            e["kind"] == "record" && matches_name(e["name"].as_str().unwrap_or(""), &relation)
        })
        .collect();
    if records.len() > 1 {
        return Err("WFT-NAME-AMBIGUOUS");
    }
    let record = *records.first().ok_or("WFT-NAME-MISSING")?;
    let field = select_field(module, record, &projection)?;
    let mut predicates = Vec::new();
    if cur.index < cur.tokens.len() && cur.tokens[cur.index] != Token::SemiColon {
        cur.keyword("WHERE")?;
        loop {
            let (a, n) = cur.column()?;
            if a != alias {
                return Err("WFT-NAME-MISSING");
            }
            let f = select_field(module, record, &n)?;
            cur.expect(Token::Eq)?;
            let value = literal(&mut cur, f)?;
            predicates.push(json!({"field":field_report(f),"literal":value}));
            if cur.index == cur.tokens.len() || cur.tokens[cur.index] == Token::SemiColon {
                break;
            }
            cur.keyword("AND")?;
        }
    }
    if cur.tokens.get(cur.index) == Some(&Token::SemiColon) {
        cur.index += 1;
    }
    if cur.index != cur.tokens.len() {
        return Err("WFT-UNSUPPORTED");
    }
    // Inert synthetic physical emitter. It claims no Truss/Ashlar mapping or results.
    let mut target_sql = "SELECT fixture.value AS value FROM fixture".to_string();
    let mut params = Vec::new();
    if !predicates.is_empty() {
        target_sql.push_str(" WHERE ");
        target_sql.push_str(
            &(1..=predicates.len())
                .map(|i| format!("fixture.p{i} = :p{i}"))
                .collect::<Vec<_>>()
                .join(" AND "),
        );
        params = predicates.iter().map(|p| p["literal"].clone()).collect();
    }
    Ok(
        json!({"reportVersion":"weft-spike/0.1.0","status":"compiled","qualification":"synthetic-candidate",
  "coreSourceSha256":sha256(include_bytes!("lib.rs")),"cargoLockSha256":sha256(include_bytes!("../../../../Cargo.lock")),
  "modelPin":supplied["pin"],"retainedDocumentJson":raw,"retainedBindingJson":binding,
  "resolved":{"module":module["id"],"record":record["id"],"projection":field_report(field),"predicates":predicates},
  "sql":target_sql,"parameters":params}),
    )
}
pub fn compile_json(request: &str) -> String {
    let response = match compile(request) {
        Ok(v) => v,
        Err(code) => json!({"reportVersion":"weft-spike/0.1.0","status":"blocked","code":code}),
    };
    serde_json::to_string(&response).expect("bounded JSON report serialization")
}
