//! Private PostgreSQL declaration correspondence. No SQL, native admission or release.
use crate::{
    error::{Diagnostic, Result},
    security_backend::{SecurityBackendContext, SecurityManifest},
    security_ir::{Disposition, Expression, Term},
    security_ontology::SecurityRef,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
const PROFILE: &str = "weft.security.record-homes/0.1.0";
const TEXT: &str = "weft.security.pg-text-c-utf8/0.1.0";
const BOOL: &str = "weft.security.pg-bool/0.1.0";
const INT: &str = "weft.security.pg-int8-lexical/0.1.0";
#[jsonschema::validator(path = "../../spec/upstream/weft-record-homes-spike-0.1.0.schema.json")]
struct Shape;
fn fail() -> Diagnostic {
    Diagnostic::new(
        "WFT-SECURITY-LOWERING-UNSUPPORTED",
        "capability",
        "Record-home declaration correspondence refused",
    )
}
struct Budget {
    work: usize,
    text: usize,
}
impl Budget {
    fn charge(&mut self, n: usize) -> Result<()> {
        if self.work == 0 || n > self.text {
            return Err(fail());
        }
        self.work -= 1;
        self.text -= n;
        Ok(())
    }
    fn value(&mut self, v: &Value, depth: usize) -> Result<()> {
        self.charge(0)?;
        if depth > 64 {
            return Err(fail());
        }
        match v {
            Value::String(s) => self.charge(s.len())?,
            Value::Number(n) => self.charge(n.as_str().len())?,
            Value::Array(a) => {
                for v in a {
                    self.value(v, depth + 1)?;
                }
            }
            Value::Object(o) => {
                for (k, v) in o {
                    self.charge(k.len())?;
                    self.value(v, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn reference(&mut self, r: &SecurityRef) -> Result<()> {
        self.charge(r.document_id.len())?;
        self.charge(r.module_id.len())?;
        self.charge(r.element_id.len())
    }
}
fn matching<'a>(
    values: &'a [Value],
    member: &str,
    expected: &str,
    b: &mut Budget,
) -> Result<&'a Value> {
    for value in values {
        let candidate = identity(&value[member])?;
        b.charge(candidate.len())?;
        if candidate == expected {
            return Ok(value);
        }
    }
    Err(fail())
}
fn make_ref(document: &str, module: &str, element: &str, b: &mut Budget) -> Result<SecurityRef> {
    b.charge(document.len())?;
    b.charge(module.len())?;
    b.charge(element.len())?;
    Ok(SecurityRef {
        document_id: document.into(),
        module_id: module.into(),
        element_id: element.into(),
    })
}
fn identity(v: &Value) -> Result<&str> {
    let s = v.as_str().ok_or_else(fail)?;
    if s.is_empty() || s.chars().count() > 4096 || s.contains('\0') {
        return Err(fail());
    }
    Ok(s)
}
fn identifier(v: &Value) -> Result<&str> {
    let s = identity(v)?;
    if s.len() > 63 {
        return Err(fail());
    }
    Ok(s)
}
fn empty_extensions(v: &Value) -> Result<()> {
    if v.get("extensions")
        .is_some_and(|x| x.as_object().is_none_or(|o| !o.is_empty()))
    {
        return Err(fail());
    }
    Ok(())
}
struct Index<'a> {
    revisions: BTreeMap<&'a str, &'a str>,
    elements: BTreeMap<SecurityRef, &'a Value>,
    types: BTreeMap<SecurityRef, &'a Value>,
}
impl<'a> Index<'a> {
    fn build(ctx: &'a SecurityBackendContext<'_>, b: &mut Budget) -> Result<Self> {
        let mut revisions = BTreeMap::new();
        let mut elements = BTreeMap::new();
        let mut types = BTreeMap::new();
        for (doc, input) in ctx.catalog().documents.iter().zip(&ctx.catalog().inputs) {
            b.charge(input.document_json.len())?;
            b.charge(input.pin.document_id.len())?;
            b.charge(input.pin.revision.len())?;
            if revisions
                .insert(input.pin.document_id.as_str(), input.pin.revision.as_str())
                .is_some()
            {
                return Err(fail());
            }
            let mut selected = BTreeSet::new();
            for id in &input.selected_module_ids {
                b.charge(id.len())?;
                selected.insert(id.as_str());
            }
            for m in doc["modules"].as_array().ok_or_else(fail)? {
                b.charge(0)?;
                let mid = identity(&m["id"])?;
                if !selected.contains(mid) {
                    continue;
                }
                for e in m["elements"].as_array().ok_or_else(fail)? {
                    let r = make_ref(&input.pin.document_id, mid, identity(&e["id"])?, b)?;
                    if elements.insert(r, e).is_some() {
                        return Err(fail());
                    }
                }
            }
        }
        let ontology = ctx.logical_plan().source().ontology();
        for name in ["entities", "associations"] {
            for t in ontology[name].as_array().ok_or_else(fail)? {
                b.value(&t["type"], 1)?;
                let r = SecurityRef::read(&t["type"]).map_err(|_| fail())?;
                if types.insert(r, t).is_some() {
                    return Err(fail());
                }
            }
        }
        Ok(Self {
            revisions,
            elements,
            types,
        })
    }
    fn reference(&self, v: &Value, b: &mut Budget) -> Result<SecurityRef> {
        b.charge(0)?;
        let r = make_ref(
            identity(&v["documentId"])?,
            identity(&v["module"])?,
            identity(&v["element"])?,
            b,
        )?;
        if self.revisions.get(r.document_id.as_str()).copied() != Some(identity(&v["revision"])?)
            || !self.elements.contains_key(&r)
        {
            return Err(fail());
        }
        Ok(r)
    }
    fn element(&self, r: &SecurityRef) -> Result<&'a Value> {
        self.elements.get(r).copied().ok_or_else(fail)
    }
    fn key(&self, r: &SecurityRef, b: &mut Budget) -> Result<(&'a str, Vec<SecurityRef>)> {
        b.reference(r)?;
        let t = self.types.get(r).ok_or_else(fail)?;
        let id = identity(&t["keyId"])?;
        let e = self.element(r)?;
        let key = matching(e["keys"].as_array().ok_or_else(fail)?, "id", id, b)?;
        let mut out = Vec::new();
        for f in key["fields"].as_array().ok_or_else(fail)? {
            b.value(f, 1)?;
            let r = make_ref(
                &r.document_id,
                identity(&f["module"])?,
                identity(&f["element"])?,
                b,
            )?;
            out.push(r);
        }
        Ok((id, out))
    }
}
/// Exact bounded integer normalization for the signed64 native image. No floating point.
fn integer(token: &str, b: &mut Budget) -> Result<String> {
    b.charge(token.len())?;
    if token.len() > 4096 {
        return Err(fail());
    }
    let (negative, body) = token
        .strip_prefix('-')
        .map_or((false, token), |s| (true, s));
    let mut exponent_split = body.split(['e', 'E']);
    let mantissa = exponent_split.next().ok_or_else(fail)?;
    let exp = exponent_split.next().unwrap_or("0");
    if exponent_split.next().is_some() {
        return Err(fail());
    }
    let digits_exp = exp.strip_prefix(['+', '-']).unwrap_or(exp);
    if digits_exp.is_empty() || !digits_exp.bytes().all(|x| x.is_ascii_digit()) {
        return Err(fail());
    }
    let mut mantissa_split = mantissa.split('.');
    let whole = mantissa_split.next().ok_or_else(fail)?;
    let fraction = mantissa_split.next();
    if mantissa_split.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|x| x.is_ascii_digit())
        || (whole.len() > 1 && whole.starts_with('0'))
    {
        return Err(fail());
    }
    if fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|x| x.is_ascii_digit())) {
        return Err(fail());
    }
    let fraction = fraction.unwrap_or("");
    b.charge(whole.len() + fraction.len())?;
    let joined = format!("{whole}{fraction}");
    let digits = joined.trim_start_matches('0');
    if digits.is_empty() {
        b.charge(1)?;
        return Ok("0".into());
    }
    let exp: i64 = exp.parse().map_err(|_| fail())?;
    let shift = exp.checked_sub(fraction.len() as i64).ok_or_else(fail)?;
    let (base, zeros) = if shift < 0 {
        let cut = shift.checked_neg().ok_or_else(fail)? as u64;
        if cut > digits.len() as u64 {
            return Err(fail());
        }
        let at = digits.len() - cut as usize;
        if !digits[at..].bytes().all(|x| x == b'0') {
            return Err(fail());
        }
        (&digits[..at], 0usize)
    } else {
        let zeros = usize::try_from(shift).map_err(|_| fail())?;
        if zeros > 19 {
            return Err(fail());
        }
        (digits, zeros)
    };
    let len = base.len().checked_add(zeros).ok_or_else(fail)?;
    if len == 0 || len > 19 {
        return Err(fail());
    }
    b.charge(len + usize::from(negative))?;
    let mut out = String::with_capacity(len + usize::from(negative));
    if negative {
        out.push('-');
    }
    out.push_str(base);
    out.extend(std::iter::repeat_n('0', zeros));
    let _: i64 = out.parse().map_err(|_| fail())?;
    Ok(out)
}
fn codec(domain: &Value) -> Result<&'static str> {
    if domain["cardinality"] != "one" || domain["nullability"] != "required" {
        return Err(fail());
    }
    match domain["scalarType"].as_str() {
        Some("string") => Ok(TEXT),
        Some("boolean") => Ok(BOOL),
        Some("integer")
            if domain["facets"]["integerWidth"]["bits"] == 64
                && domain["facets"]["integerWidth"]["signed"] == true =>
        {
            Ok(INT)
        }
        _ => Err(fail()),
    }
}
fn literal_image(codec: &str, v: &Value, b: &mut Budget) -> Result<String> {
    b.value(v, 1)?;
    let o = v.as_object().ok_or_else(fail)?;
    if o.len() != 1 {
        return Err(fail());
    }
    match codec {
        TEXT => {
            let s = v["string"].as_str().ok_or_else(fail)?;
            if s.contains('\0') {
                return Err(fail());
            }
            b.charge(s.len())?;
            Ok(s.into())
        }
        BOOL => {
            b.charge(5)?;
            Ok(v["boolean"].as_bool().ok_or_else(fail)?.to_string())
        }
        INT => integer(v["integerToken"].as_str().ok_or_else(fail)?, b),
        _ => Err(fail()),
    }
}
fn owner_domain(index: &Index<'_>, r: &SecurityRef, b: &mut Budget) -> Result<Value> {
    let f = index.element(r)?;
    b.value(f, 1)?;
    empty_extensions(f)?;
    let d = crate::security_ontology::domain(f).map_err(|_| fail())?;
    let c = codec(&d)?;
    if let Some(a) = d["allowedValues"].as_array() {
        for v in a {
            literal_image(c, v, b)?;
        }
    }
    if let Some(range) = d["facets"].get("range") {
        for k in ["min", "max"] {
            if let Some(v) = range.get(k) {
                literal_image(c, v, b)?;
            }
        }
    }
    Ok(d)
}
struct Home<'a> {
    source: &'a Value,
    fields: BTreeMap<SecurityRef, &'a Value>,
}
fn fields<'a>(
    v: &'a Value,
    target: &SecurityRef,
    index: &Index<'_>,
    b: &mut Budget,
) -> Result<BTreeMap<SecurityRef, &'a Value>> {
    let t = index.types.get(target).ok_or_else(fail)?;
    let mut members = BTreeSet::new();
    for f in t["fields"].as_array().ok_or_else(fail)? {
        b.value(&f["ref"], 1)?;
        members.insert(SecurityRef::read(&f["ref"]).map_err(|_| fail())?);
    }
    let mut out = BTreeMap::new();
    let mut columns = BTreeSet::new();
    for f in v.as_array().ok_or_else(fail)? {
        let r = index.reference(&f["field"], b)?;
        let d = owner_domain(index, &r, b)?;
        if !members.contains(&r)
            || f["sourceDomain"] != d
            || f["codec"] != codec(&d)?
            || !columns.insert(identifier(&f["column"])?)
            || out.insert(r, f).is_some()
        {
            return Err(fail());
        }
    }
    Ok(out)
}
#[derive(PartialEq, Eq)]
struct SourceImage<'a> {
    schema: &'a str,
    name: &'a str,
    kind: &'a str,
    discriminator: Option<(&'a str, &'a str, String)>,
}
fn source<'a>(
    v: &'a Value,
    fields: &BTreeMap<SecurityRef, &Value>,
    b: &mut Budget,
) -> Result<SourceImage<'a>> {
    let discriminator = if v["discriminator"].is_null() {
        None
    } else {
        let d = &v["discriminator"];
        let col = identifier(&d["column"])?;
        for f in fields.values() {
            b.charge(0)?;
            if f["column"] == col {
                return Err(fail());
            }
        }
        let c = identity(&d["codec"])?;
        Some((col, c, literal_image(c, &d["literal"], b)?))
    };
    Ok(SourceImage {
        schema: identifier(&v["schema"])?,
        name: identifier(&v["name"])?,
        kind: identity(&v["kind"])?,
        discriminator,
    })
}
fn required_fields<T: std::borrow::Borrow<SecurityRef>>(
    home: &Home<'_>,
    fields: impl IntoIterator<Item = T>,
    b: &mut Budget,
) -> Result<()> {
    for item in fields {
        let r = item.borrow();
        b.reference(r)?;
        if !home.fields.contains_key(r) {
            return Err(fail());
        }
    }
    Ok(())
}
fn known_expression(e: &Expression, index: &Index<'_>, b: &mut Budget, depth: usize) -> Result<()> {
    b.charge(0)?;
    if depth > 64 {
        return Err(fail());
    }
    match e {
        Expression::Literal(_) => {}
        Expression::Equal(a, c) => {
            for t in [a, c] {
                b.charge(0)?;
                if let Term::Constant {
                    field,
                    domain,
                    literal,
                } = t
                {
                    let d = owner_domain(index, field, b)?;
                    if &d != domain {
                        return Err(fail());
                    }
                    literal_image(codec(&d)?, literal, b)?;
                } else if matches!(t, Term::Context { .. }) {
                    return Err(fail());
                }
            }
        }
        Expression::And(args) | Expression::Or(args) => {
            for a in args {
                known_expression(a, index, b, depth + 1)?;
            }
        }
        Expression::Not(a) => known_expression(a, index, b, depth + 1)?,
        Expression::Exists { condition, .. } => known_expression(condition, index, b, depth + 1)?,
    }
    Ok(())
}
fn application_value(v: &crate::application_ir::Value, b: &mut Budget) -> Result<()> {
    use crate::{application_ir::Value as V, ir::Family};
    let (value, t) = match v {
        V::Literal {
            value,
            logical_type,
            ..
        }
        | V::Parameter {
            value,
            logical_type,
            ..
        } => (value, logical_type),
        V::Field { .. } => return Ok(()),
    };
    b.charge(value.len())?;
    if t.nullable {
        return Err(fail());
    }
    match t.family {
        Family::String => {
            if value.contains('\0') {
                return Err(fail());
            }
        }
        Family::Boolean => {
            if value != "true" && value != "false" {
                return Err(fail());
            }
        }
        Family::Integer => {
            integer(value, b)?;
        }
        _ => return Err(fail()),
    }
    Ok(())
}
fn predicate(p: &crate::application_ir::Predicate, b: &mut Budget) -> Result<()> {
    b.charge(0)?;
    use crate::application_ir::Predicate as P;
    match p {
        P::Equal { right, .. } => application_value(right, b)?,
        P::LexicographicGreater { values, .. } => {
            for v in values {
                application_value(v, b)?;
            }
        }
        P::HasRelated { .. } => return Err(fail()),
    }
    Ok(())
}
fn parse_binding(raw: &str, b: &mut Budget) -> Result<Value> {
    b.charge(raw.len())?;
    let value = crate::json::checked_json_bounded_charged(raw, 4 * 1024 * 1024, 64, &mut b.work)
        .map_err(|_| fail())?;
    b.value(&value, 1)?;
    Ok(value)
}
/// Correspondence only. The return value is not a native/authorization receipt.
#[allow(dead_code)]
pub(crate) fn check(ctx: &SecurityBackendContext<'_>, m: &SecurityManifest) -> Result<()> {
    check_budget(
        ctx,
        m,
        &mut Budget {
            work: 1_000_000,
            text: 16_000_000,
        },
    )
}
fn check_budget(
    ctx: &SecurityBackendContext<'_>,
    m: &SecurityManifest,
    b: &mut Budget,
) -> Result<()> {
    let value = parse_binding(ctx.binding_json(), b)?;
    if !Shape::is_valid(&value)
        || m.interface_version != "weft-security-backend/0.1.0"
        || m.binding_profile != PROFILE
        || m.backend_id != ctx.backend_id()
        || m.backend_version != ctx.backend_version()
        || m.source_profiles.len() != 1
    {
        return Err(fail());
    }
    let p = &m.source_profiles[0];
    if (
        &*p.dialect,
        &*p.application_ir,
        &*p.policy,
        &*p.ontology,
        &*p.security_ir,
    ) != (
        "weft-sql/0.2.0",
        "weft-ir/0.2.0",
        "0.1.0",
        "0.1.0",
        "weft.security.logical-ir/0.1.0",
    ) {
        return Err(fail());
    }
    if m.target_profiles.len() > 256 {
        return Err(fail());
    }
    for t in &m.target_profiles {
        b.charge(t.id.len())?;
    }
    for text in [
        &m.interface_version,
        &m.binding_profile,
        &m.backend_id,
        &m.backend_version,
        &p.dialect,
        &p.application_ir,
        &p.policy,
        &p.ontology,
        &p.security_ir,
    ] {
        b.charge(text.len())?;
    }
    let selected: Vec<_> = m
        .target_profiles
        .iter()
        .filter_map(|p| Some(b.charge(p.id.len()).map(|_| p)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|p| p.id == ctx.target_profile())
        .collect();
    if selected.len() != 1
        || selected[0].engine != "postgresql"
        || selected[0].engine_version != "17.9"
        || selected[0]
            .session_settings
            .as_object()
            .is_none_or(|o| !o.is_empty())
    {
        return Err(fail());
    }
    if value["selection"]["backendId"] != ctx.backend_id()
        || value["selection"]["backendVersion"] != ctx.backend_version()
        || value["selection"]["targetProfile"] != ctx.target_profile()
    {
        return Err(fail());
    }
    let pins = value["modelPins"].as_array().ok_or_else(fail)?;
    if pins.len() != ctx.catalog().inputs.len() {
        return Err(fail());
    }
    let mut seen = BTreeSet::new();
    for pin in pins {
        b.charge(0)?;
        if !seen.insert(identity(&pin["documentId"])?) {
            return Err(fail());
        }
        let mut actual = None;
        for input in &ctx.catalog().inputs {
            b.charge(input.pin.document_id.len())?;
            if input.pin.document_id == pin["documentId"] {
                actual = Some(input);
                break;
            }
        }
        let actual = actual.ok_or_else(fail)?;
        if pin["revision"] != actual.pin.revision
            || pin["umfVersion"] != actual.pin.umf_version
            || pin["sha256"] != actual.pin.sha256
        {
            return Err(fail());
        }
    }
    let index = Index::build(ctx, b)?;
    let mut homes = BTreeMap::new();
    let mut sources = Vec::new();
    for t in value["types"].as_array().ok_or_else(fail)? {
        let r = index.reference(&t["type"], b)?;
        let record = index.element(&r)?;
        b.value(record, 1)?;
        empty_extensions(record)?;
        let actual = index.types.get(&r).ok_or_else(fail)?;
        let fs = fields(&t["fields"], &r, &index, b)?;
        let (kid, key) = index.key(&r, b)?;
        let declared = t["key"]["fields"]
            .as_array()
            .ok_or_else(fail)?
            .iter()
            .map(|f| index.reference(f, b))
            .collect::<Result<Vec<_>>>()?;
        if t["key"]["id"] != kid || declared != key {
            return Err(fail());
        }
        let home = Home {
            source: &t["source"],
            fields: fs,
        };
        required_fields(&home, key, b)?;
        let endpoints = t["endpoints"].as_array().ok_or_else(fail)?;
        let actual_endpoints = actual.get("endpoints").and_then(Value::as_array);
        if endpoints.len() != actual_endpoints.map_or(0, Vec::len) {
            return Err(fail());
        }
        let mut roles = BTreeSet::new();
        for ep in endpoints {
            let role = identity(&ep["role"])?;
            if !roles.insert(role) {
                return Err(fail());
            }
            let original = matching(actual_endpoints.ok_or_else(fail)?, "role", role, b)?;
            let target = index.reference(&ep["target"], b)?;
            b.value(&original["target"], 1)?;
            if target != SecurityRef::read(&original["target"]).map_err(|_| fail())? {
                return Err(fail());
            }
            let (tkid, tkey) = index.key(&target, b)?;
            if ep["targetKeyId"] != tkid {
                return Err(fail());
            }
            let fields = ep["fields"]
                .as_array()
                .ok_or_else(fail)?
                .iter()
                .map(|f| index.reference(f, b))
                .collect::<Result<Vec<_>>>()?;
            let mut expected = Vec::new();
            for f in original["fields"].as_array().ok_or_else(fail)? {
                b.value(f, 1)?;
                expected.push(SecurityRef::read(f).map_err(|_| fail())?);
            }
            if fields != expected || fields.len() != tkey.len() {
                return Err(fail());
            }
            required_fields(&home, fields.iter(), b)?;
            for (f, k) in fields.iter().zip(&tkey) {
                if owner_domain(&index, f, b)? != owner_domain(&index, k, b)? {
                    return Err(fail());
                }
            }
        }
        let image = source(home.source, &home.fields, b)?;
        b.reference(&r)?;
        sources.push((r.clone(), image));
        if homes.insert(r, home).is_some() {
            return Err(fail());
        }
    }
    // Every provided association endpoint still requires its target home and Key.
    for r in homes.keys() {
        let t = index.types.get(r).ok_or_else(fail)?;
        if let Some(endpoints) = t["endpoints"].as_array() {
            for e in endpoints {
                b.value(&e["target"], 1)?;
                let target = SecurityRef::read(&e["target"]).map_err(|_| fail())?;
                let h = homes.get(&target).ok_or_else(fail)?;
                required_fields(h, index.key(&target, b)?.1, b)?;
            }
        }
    }
    let mut carriers = BTreeMap::new();
    for c in value["queryCarriers"].as_array().ok_or_else(fail)? {
        let r = index.reference(&c["type"], b)?;
        if !homes.contains_key(&r) {
            return Err(fail());
        }
        let h = Home {
            source: &c["source"],
            fields: fields(&c["fields"], &r, &index, b)?,
        };
        required_fields(&h, index.key(&r, b)?.1, b)?;
        let image = source(h.source, &h.fields, b)?;
        b.reference(&r)?;
        sources.push((r.clone(), image));
        if carriers.insert(r, h).is_some() {
            return Err(fail());
        }
    }
    for (i, (r, s)) in sources.iter().enumerate() {
        for (q, t) in &sources[..i] {
            b.reference(r)?;
            b.reference(q)?;
            if s.schema != t.schema || s.name != t.name {
                continue;
            }
            if s.kind != t.kind {
                return Err(fail());
            }
            if r == q {
                if s != t {
                    return Err(fail());
                }
            } else {
                match (&s.discriminator, &t.discriminator) {
                    (Some((sc, sk, sv)), Some((tc, tk, tv)))
                        if sc == tc && sk == tk && sv != tv => {}
                    _ => return Err(fail()),
                }
            }
        }
    }
    let subject = &value["subject"];
    let s = index.reference(&subject["type"], b)?;
    b.value(&ctx.logical_plan().source().ontology()["subject"], 1)?;
    if s != SecurityRef::read(&ctx.logical_plan().source().ontology()["subject"])
        .map_err(|_| fail())?
    {
        return Err(fail());
    }
    let sh = homes.get(&s).ok_or_else(fail)?;
    let login = identifier(&subject["login"]["column"])?;
    for f in sh.fields.values() {
        b.charge(0)?;
        if f["column"] == login && f["codec"] != TEXT {
            return Err(fail());
        }
    }
    for scan in ctx.requirements().scans() {
        b.charge(scan.inventory().scan().len())?;
        let target = scan.inventory().target();
        let home = homes.get(target).ok_or_else(fail)?;
        let carrier = carriers.get(target).ok_or_else(fail)?;
        required_fields(
            home,
            scan.inventory()
                .projection_fields()
                .iter()
                .chain(scan.inventory().query_fields()),
            b,
        )?;
        required_fields(
            carrier,
            scan.inventory()
                .projection_fields()
                .iter()
                .chain(scan.inventory().query_fields()),
            b,
        )?;
        for action in scan.actions() {
            b.charge(0)?;
            let inv = action.inventory();
            if !inv.context().is_empty() {
                return Err(fail());
            }
            for (target, (id, fields)) in inv.keys() {
                b.charge(id.len())?;
                let h = homes.get(target).ok_or_else(fail)?;
                let (actual_id, actual_key) = index.key(target, b)?;
                if id != actual_id || fields != &actual_key {
                    return Err(fail());
                }
                required_fields(h, fields.iter(), b)?;
            }
            for (target, fs) in inv.fields() {
                let h = homes.get(target).ok_or_else(fail)?;
                required_fields(h, fs.iter(), b)?;
            }
            for a in inv.associations() {
                b.reference(a)?;
                if !homes.contains_key(a) {
                    return Err(fail());
                }
            }
            for rule in action.rules() {
                known_expression(&rule.condition, &index, b, 1)?;
                for (field, d) in &rule.disclosure {
                    let domain = owner_domain(&index, field, b)?;
                    if let Disposition::Transformed {
                        output_field,
                        domain: output_domain,
                        literal,
                        ..
                    } = d
                    {
                        let actual = owner_domain(&index, output_field, b)?;
                        if &actual != output_domain {
                            return Err(fail());
                        }
                        literal_image(codec(&actual)?, literal, b)?;
                    } else {
                        codec(&domain)?;
                    }
                }
            }
        }
    }
    let app = ctx.query().application_plan();
    for p in &app.filters {
        predicate(p, b)?;
    }
    for j in &app.joins {
        b.charge(j.right.occurrence.len())?;
        for p in &j.on {
            predicate(p, b)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn sr(id: &str) -> Value {
        json!({"documentId":"domain","moduleId":"m","elementId":id})
    }
    fn with_case(
        sql: &str,
        fixture: impl FnOnce(&mut Value),
        binding: impl FnOnce(&mut Value),
        run: impl FnOnce(&SecurityBackendContext<'_>, &SecurityManifest),
    ) {
        with_raw(
            sql,
            fixture,
            |mut v| {
                binding(&mut v);
                v.to_string()
            },
            run,
        )
    }
    fn with_raw(
        sql: &str,
        mutate: impl FnOnce(&mut Value),
        binding: impl FnOnce(Value) -> String,
        run: impl FnOnce(&SecurityBackendContext<'_>, &SecurityManifest),
    ) {
        let mut f: Value =
            serde_json::from_str(include_str!("../tests/security-source-fixture.json")).unwrap();
        mutate(&mut f);
        let source = &f["resolution"]["documents"][0];
        let mut doc = source["document"].clone();
        for e in doc["modules"][0]["elements"].as_array_mut().unwrap() {
            e["name"] = e["id"].clone();
            if e["scalarType"] == "integer" {
                e["facets"] = json!({"integerWidth":{"bits":64,"signed":true}});
            }
        }
        let raw = doc.to_string();
        let inputs=serde_json::from_value(json!([{"documentJson":raw,"pin":{"documentId":doc["id"],"revision":source["revision"],"umfVersion":"0.8.0","sha256":crate::json::sha256(raw.as_bytes())},"selectedModuleIds":["m"]}])).unwrap();
        let catalog = crate::model::Catalog::prepare_security(inputs).unwrap();
        let policy = f["policy"].to_string();
        let ontology = f["resolution"]["ontology"].to_string();
        let packet =
            crate::security_source::SecuritySourcePacket::read(&policy, &ontology, &catalog)
                .unwrap();
        let plan = crate::security_ir::SecurityLogicalPlan::read(packet, &catalog).unwrap();
        let qualified = |r: &Value| json!({"documentId":r["documentId"],"revision":source["revision"],"module":r["moduleId"],"element":r["elementId"]});
        let mut types = Vec::new();
        for kind in ["entities", "associations"] {
            for t in f["resolution"]["ontology"][kind].as_array().unwrap() {
                let id = t["type"]["elementId"].as_str().unwrap();
                let record = doc["modules"][0]["elements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["id"] == id)
                    .unwrap();
                let key = record["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|k| k["id"] == t["keyId"])
                    .unwrap();
                let mut fields = Vec::new();
                for item in t["fields"].as_array().unwrap() {
                    let r = &item["ref"];
                    let field = doc["modules"][0]["elements"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|e| e["id"] == r["elementId"])
                        .unwrap();
                    let d = crate::security_ontology::domain(field).unwrap();
                    let c = match d["scalarType"].as_str().unwrap() {
                        "boolean" => BOOL,
                        "integer" => INT,
                        _ => TEXT,
                    };
                    fields.push(json!({"field":qualified(r),"column":r["elementId"],"codec":c,"sourceDomain":d}));
                }
                let endpoints=t.get("endpoints").and_then(Value::as_array).map(|es|es.iter().map(|e|json!({"role":e["role"],"target":qualified(&e["target"]),"targetKeyId":"pk","fields":e["fields"].as_array().unwrap().iter().map(qualified).collect::<Vec<_>>()})).collect::<Vec<_>>()).unwrap_or_default();
                types.push(json!({"type":qualified(&t["type"]),"source":{"schema":"public","name":id,"kind":"table","discriminator":null},"key":{"id":t["keyId"],"fields":key["fields"].as_array().unwrap().iter().map(|r|qualified(&json!({"documentId":"domain","moduleId":r["module"],"elementId":r["element"]}))).collect::<Vec<_>>()},"fields":fields,"endpoints":endpoints}));
            }
        }
        let resource = types
            .iter()
            .find(|t| t["type"]["element"] == "Resource")
            .unwrap();
        let carrier = json!({"type":resource["type"],"source":resource["source"],"fields":resource["fields"]});
        let binding = binding(
            json!({"version":PROFILE,"selection":{"backendId":"fixture","backendVersion":"v1","targetProfile":"pg"},"nativeSemantics":{"encoding":"UTF8","textCollation":"C"},"modelPins":catalog.pins(),"subject":{"type":qualified(&sr("Staff")),"mechanism":"weft.security.pg-session-user/0.1.0","login":{"column":"login","codec":TEXT}},"types":types,"queryCarriers":[carrier],"context":[]}),
        );
        let p=json!({"version":"weft.security.query-profile/0.1.0","id":"homes","revision":"p1","action":"read","modelPins":catalog.pins(),"policySha256":crate::json::sha256(policy.as_bytes()),"ontologySha256":crate::json::sha256(ontology.as_bytes()),"binding":{"backendId":"fixture","backendVersion":"v1","targetProfile":"pg","sha256":crate::json::sha256(binding.as_bytes())},"targets":[sr("Resource")],"bindings":[]}).to_string();
        let profile = crate::security_query_profile::SecurityQueryProfile::read(
            &p, &plan, &catalog, &binding, "fixture", "v1", "pg",
        )
        .unwrap();
        let query = crate::security_query_uses::SecurityResolvedQuery::resolve(
            sql,
            &catalog,
            &plan,
            Default::default(),
            None,
        )
        .unwrap();
        let uses = profile
            .admit_resolved_query(&query, &plan, &catalog, &binding, "fixture", "v1", "pg")
            .unwrap();
        let ctx = SecurityBackendContext::new(
            &catalog, &plan, &query, &uses, &binding, "fixture", "v1", "pg",
        )
        .unwrap();
        let m=crate::security_backend::validate_security_manifest_json(&json!({"interfaceVersion":"weft-security-backend/0.1.0","backendId":"fixture","backendVersion":"v1","sourceProfiles":[{"dialect":"weft-sql/0.2.0","applicationIr":"weft-ir/0.2.0","policy":"0.1.0","ontology":"0.1.0","securityIr":"weft.security.logical-ir/0.1.0"}],"bindingProfile":PROFILE,"targetProfiles":[{"id":"pg","engine":"postgresql","engineVersion":"17.9","sessionSettings":{},"storageLayoutRevision":"fixture","publicationRevision":"fixture"}],"capabilities":[{"id":"fixture","targetProfiles":["pg"],"languageProfiles":[{"dialectProfile":"weft-sql/0.2.0","irVersion":"weft-ir/0.2.0"}],"logicalDomain":{"profile":"fixture-uninterpreted"},"resultDomain":{"profile":"fixture-uninterpreted"},"constraints":[],"obligations":[],"status":"candidate","evidence":["fixture-only"]}],"evidence":["fixture-only"]}).to_string()).unwrap();
        run(&ctx, &m)
    }
    fn home<'a>(v: &'a mut Value, id: &str) -> &'a mut Value {
        v["types"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|t| t["type"]["element"] == id)
            .unwrap()
    }
    fn natural(f: &mut Value) {
        f["resolution"]["documents"][0]["revision"] = json!("schema-natural-1");
        f["resolution"]["ontology"]["documents"][0]["revision"] = json!("schema-natural-1");
        for (id, removed, parts) in [
            (
                "Assignment",
                "assignmentId",
                ["assignmentStaff", "assignmentProject"],
            ),
            ("Ownership", "ownerId", ["ownerResource", "ownerProject"]),
        ] {
            let record = f["resolution"]["documents"][0]["document"]["modules"][0]["elements"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|e| e["id"] == id)
                .unwrap();
            record["members"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["element"] != removed);
            record["keys"][0]["fields"] = json!(parts.map(|id| json!({"module":"m","element":id})));
            let t = f["resolution"]["ontology"]["associations"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|t| t["type"]["elementId"] == id)
                .unwrap();
            t["fields"]
                .as_array_mut()
                .unwrap()
                .retain(|r| r["ref"]["elementId"] != removed);
        }
    }
    #[test]
    fn actual_raw_typed_and_discriminated_homes_keep_natural_keys_and_scan_occurrences() {
        for variant in 0..3 {
            with_case(
                "SELECT r.salary FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId",
                natural,
                |v| {
                    if variant > 0 {
                        for t in v["types"].as_array_mut().unwrap() {
                            if variant == 1 {
                                t["source"]["kind"] = json!("view");
                            } else {
                                t["source"]["name"] = json!("nodes");
                                t["source"]["discriminator"] = json!({"column":"type_tag","codec":TEXT,"literal":{"string":t["type"]["element"]}});
                            }
                        }
                        let source = home(v, "Resource")["source"].clone();
                        v["queryCarriers"][0]["source"] = source;
                    }
                },
                |ctx, m| {
                    assert_eq!(ctx.requirements().scans().len(), 2);
                    assert!(check(ctx, m).is_ok());
                },
            );
        }
        with_case(
            "SELECT COUNT(*) FROM Resource r",
            |_| {},
            |_| {},
            |ctx, m| {
                assert_eq!(ctx.requirements().scans().len(), 1);
                assert!(check(ctx, m).is_ok());
            },
        );
    }
    #[test]
    fn complete_actual_keys_endpoints_fields_and_revisions_cannot_be_substituted() {
        for case in 0..9 {
            with_case(
                "SELECT r.salary FROM Resource r",
                natural,
                |v| match case {
                    0 => home(v, "Assignment")["key"]["fields"]
                        .as_array_mut()
                        .unwrap()
                        .reverse(),
                    1 => home(v, "Assignment")["endpoints"][0]["role"] = json!("foreign"),
                    2 => {
                        let target = home(v, "Project")["type"].clone();
                        home(v, "Assignment")["endpoints"][0]["target"] = target;
                    }
                    3 => home(v, "Assignment")["endpoints"][0]["targetKeyId"] = json!("foreign"),
                    4 => {
                        home(v, "Assignment")["endpoints"][0]["fields"][0]["element"] =
                            json!("assignmentProject")
                    }
                    5 => home(v, "Assignment")["fields"]
                        .as_array_mut()
                        .unwrap()
                        .retain(|f| f["field"]["element"] != "active"),
                    6 => home(v, "Staff")["type"]["revision"] = json!("foreign"),
                    7 => {
                        home(v, "Resource")["fields"][0]["sourceDomain"]["allowedValues"] =
                            json!([])
                    }
                    _ => {
                        v["types"]
                            .as_array_mut()
                            .unwrap()
                            .retain(|t| t["type"]["element"] != "Project");
                    }
                },
                |ctx, m| {
                    let e = check(ctx, m).unwrap_err();
                    assert_eq!(
                        (e.code.as_str(), e.phase.as_str()),
                        ("WFT-SECURITY-LOWERING-UNSUPPORTED", "capability")
                    );
                },
            );
        }
    }
    #[test]
    fn source_reuse_partition_carrier_and_subject_correspondence_are_distinct() {
        for case in 0..9 {
            with_case(
                "SELECT r.salary FROM Resource r",
                |_| {},
                |v| match case {
                    0 => {
                        let s = home(v, "Resource")["source"].clone();
                        home(v, "Project")["source"] = s;
                    }
                    1 => {
                        let c = v["queryCarriers"][0].clone();
                        v["queryCarriers"].as_array_mut().unwrap().push(c);
                    }
                    2 => v["queryCarriers"][0]["fields"]
                        .as_array_mut()
                        .unwrap()
                        .retain(|f| f["field"]["element"] != "salary"),
                    3 => {
                        let t = home(v, "Project")["type"].clone();
                        v["subject"]["type"] = t;
                    }
                    4 => v["subject"]["login"]["column"] = json!("😀".repeat(16)),
                    5 => home(v, "Resource")["source"]["name"] = json!("x\0y"),
                    6 => {
                        home(v, "Resource")["source"]["discriminator"] =
                            json!({"column":"resourceId","codec":TEXT,"literal":{"string":"x"}});
                        v["queryCarriers"][0]["source"] = home(v, "Resource")["source"].clone();
                    }
                    7 => {
                        home(v, "Resource")["source"]["discriminator"] = json!({"column":"tag","codec":INT,"literal":{"integerToken":"9223372036854775808"}});
                        v["queryCarriers"][0]["source"] = home(v, "Resource")["source"].clone();
                    }
                    _ => {
                        let field = home(v, "Resource")["fields"][0].clone();
                        home(v, "Resource")["fields"]
                            .as_array_mut()
                            .unwrap()
                            .push(field);
                    }
                },
                |ctx, m| assert!(check(ctx, m).is_err()),
            );
        }
        with_case(
            "SELECT r.salary FROM Resource r",
            |_| {},
            |v| home(v, "Resource")["source"]["name"] = json!("x\"; DROP TABLE x; --"),
            |ctx, m| assert!(check(ctx, m).is_ok()),
        );
    }
    #[test]
    fn exact_registration_pins_and_closed_transport_refuse() {
        with_case(
            "SELECT r.salary FROM Resource r",
            |_| {},
            |_| {},
            |ctx, m| {
                for case in 0..9 {
                    let mut m = m.clone();
                    match case {
                        0 => m.binding_profile = "unknown".into(),
                        1 => m.source_profiles[0].security_ir = "unknown".into(),
                        2 => m.target_profiles[0].engine = "unknown".into(),
                        3 => m.target_profiles[0].engine_version = "17.8".into(),
                        4 => m.target_profiles[0].session_settings = json!({"unknown":true}),
                        5 => m.backend_id = "other".into(),
                        6 => m.interface_version = "unknown".into(),
                        7 => m.target_profiles.push(m.target_profiles[0].clone()),
                        _ => m.source_profiles.clear(),
                    };
                    assert!(check(ctx, &m).is_err());
                }
            },
        );
        for case in 0..3 {
            with_raw(
                "SELECT r.salary FROM Resource r",
                |_| {},
                |mut v| match case {
                    0 => {
                        v["modelPins"][0]["sha256"] = json!("0".repeat(64));
                        v.to_string()
                    }
                    1 => "[]".into(),
                    2 => {
                        v["unknown"] = json!(true);
                        v.to_string()
                    }
                    _ => {
                        let s = v.to_string();
                        format!("{{\"version\":\"{PROFILE}\",{}", &s[1..])
                    }
                },
                |ctx, m| assert!(check(ctx, m).is_err()),
            );
        }
    }
    #[test]
    fn known_literal_native_images_survive_false_branches_and_empty_results() {
        for literal in ["good", "x\0y"] {
            with_case(
                "SELECT r.salary FROM Resource r",
                |f| {
                    let constant = |s: &str| json!({"kind":"constant","field":sr("staffId"),"value":{"string":s}});
                    f["policy"]["rules"][1]["condition"] = json!({"op":"and","args":[{"op":"literal","value":false},{"op":"eq","left":constant(literal),"right":constant("good")}]});
                },
                |_| {},
                |ctx, m| assert_eq!(check(ctx, m).is_ok(), literal == "good"),
            );
        }
        with_case(
            "SELECT r.salary FROM Resource r",
            |f| {
                for e in f["resolution"]["documents"][0]["document"]["modules"][0]["elements"]
                    .as_array_mut()
                    .unwrap()
                {
                    if e["id"] == "staffId" || e["id"] == "assignmentStaff" {
                        e["allowedValues"] = json!([{"string":"x\0y"}]);
                    }
                }
            },
            |_| {},
            |ctx, m| assert!(check(ctx, m).is_err()),
        );
        with_case(
            "SELECT r.salary FROM Resource r WHERE r.resourceId='good'",
            |_| {},
            |_| {},
            |ctx, m| assert!(check(ctx, m).is_ok()),
        );
        let value = crate::application_ir::Value::Literal {
            value: "x\0y".into(),
            logical_type: crate::ir::LogicalType {
                family: crate::ir::Family::String,
                facets: json!({}),
                nullable: false,
            },
            span: crate::ir::Span { start: 0, end: 3 },
        };
        assert!(
            application_value(
                &value,
                &mut Budget {
                    work: 100,
                    text: 1000
                }
            )
            .is_err()
        );
    }
    #[test]
    fn metadata_only_transforms_and_selected_unknown_meaning_are_checked() {
        for mode in 0..3 {
            with_case(
                "SELECT r.salary FROM Resource r",
                |f| {
                    f["resolution"]["documents"][0]["document"]["modules"][0]["elements"].as_array_mut().unwrap().push(json!({"id":"maskOutput","kind":"field","scalarType":"string","cardinality":"one","nullability":if mode==2{"absent-allowed"}else{"required"},"extensions":{}}));
                    f["policy"]["rules"][0]["condition"] = json!({"op":"literal","value":false});
                    f["policy"]["rules"][0]["disclosure"][0]["disposition"] = json!({"kind":"transformed","transform":"constant","version":"0.1.0","field":sr("maskOutput"),"value":if mode==2{Value::Null}else{json!({"string":if mode==0{"good"}else{"x\0y"}})}});
                },
                |_| {},
                |ctx, m| assert_eq!(check(ctx, m).is_ok(), mode == 0),
            );
        }
        for selected in [false, true] {
            with_case(
                "SELECT r.salary FROM Resource r",
                |f| {
                    let elements =
                        f["resolution"]["documents"][0]["document"]["modules"][0]["elements"]
                            .as_array_mut()
                            .unwrap();
                    if selected {
                        elements.iter_mut().find(|e| e["id"] == "Staff").unwrap()["extensions"] =
                            json!({"urn:unknown":{"meaning":"unknown"}});
                    } else {
                        elements.push(json!({"id":"unmapped","kind":"field","scalarType":"string","nullability":"required","cardinality":"one","extensions":{"urn:unknown":{"meaning":"unknown"}}}));
                    }
                },
                |_| {},
                |ctx, m| assert_eq!(check(ctx, m).is_ok(), !selected),
            );
        }
    }

    fn composite_project(f: &mut Value) {
        natural(f);
        f["resolution"]["documents"][0]["revision"] = json!("schema-composite-1");
        f["resolution"]["ontology"]["documents"][0]["revision"] = json!("schema-composite-1");
        for (record_id, field_id) in [
            ("Project", "projectTenant"),
            ("Assignment", "assignmentTenant"),
            ("Ownership", "ownerTenant"),
        ] {
            let elements = f["resolution"]["documents"][0]["document"]["modules"][0]["elements"]
                .as_array_mut()
                .unwrap();
            let record = elements.iter_mut().find(|e| e["id"] == record_id).unwrap();
            let local = json!({"module":"m","element":field_id});
            record["members"]
                .as_array_mut()
                .unwrap()
                .push(local.clone());
            record["keys"][0]["fields"]
                .as_array_mut()
                .unwrap()
                .push(local);
            elements.push(json!({"id":field_id,"kind":"field","scalarType":"string","cardinality":"one","nullability":"required","extensions":{}}));
            let kind = if record_id == "Project" {
                "entities"
            } else {
                "associations"
            };
            let t = f["resolution"]["ontology"][kind]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|t| t["type"]["elementId"] == record_id)
                .unwrap();
            t["fields"]
                .as_array_mut()
                .unwrap()
                .push(json!({"ref":sr(field_id),"protection":"unprotected"}));
            if record_id != "Project" {
                let endpoint = t["endpoints"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|e| e["role"] == "project")
                    .unwrap();
                endpoint["fields"]
                    .as_array_mut()
                    .unwrap()
                    .push(sr(field_id));
            }
        }
    }
    #[test]
    fn same_domain_composite_endpoints_keep_position_and_own_association_key() {
        for reversed in [false, true] {
            with_case(
                "SELECT r.salary FROM Resource r",
                composite_project,
                |v| {
                    assert_eq!(
                        home(v, "Assignment")["key"]["fields"]
                            .as_array()
                            .unwrap()
                            .len(),
                        3
                    );
                    if reversed {
                        home(v, "Assignment")["endpoints"][1]["fields"]
                            .as_array_mut()
                            .unwrap()
                            .reverse();
                    }
                },
                |ctx, m| assert_eq!(check(ctx, m).is_ok(), !reversed),
            );
        }
    }
    #[test]
    fn actual_context_dependencies_refuse_even_under_false_branches() {
        for used in [false, true] {
            with_case(
                "SELECT r.salary FROM Resource r",
                |f| {
                    f["resolution"]["ontology"]["context"] = json!([sr("active")]);
                    if used {
                        f["policy"]["rules"][1]["condition"] = json!({"op":"and","args":[{"op":"literal","value":false},{"op":"eq","left":{"kind":"context","field":sr("active")},"right":{"kind":"constant","field":sr("active"),"value":{"boolean":true}}}]});
                    }
                },
                |_| {},
                |ctx, m| assert_eq!(check(ctx, m).is_ok(), !used),
            );
        }
    }
    #[test]
    fn exact_integer_native_image_never_rounds_or_expands_unboundedly() {
        for (token, expected) in [
            ("1e0", "1"),
            ("1.00e2", "100"),
            ("-0", "0"),
            ("0e9999999999999999999999999999999999999", "0"),
            ("9223372036854775807", "9223372036854775807"),
            ("-9223372036854775808", "-9223372036854775808"),
        ] {
            assert_eq!(
                integer(
                    token,
                    &mut Budget {
                        work: 100,
                        text: 10000
                    }
                )
                .unwrap(),
                expected
            );
        }
        for token in [
            "9223372036854775808",
            "-9223372036854775809",
            "1.2",
            "1e9999999999999999999999999999999",
            "01",
            "1e-9999",
            "1.",
            "1e",
            "+1",
        ] {
            assert!(
                integer(
                    token,
                    &mut Budget {
                        work: 100,
                        text: 10000
                    }
                )
                .is_err()
            );
        }
        assert!(
            integer(
                &"1".repeat(4097),
                &mut Budget {
                    work: 100,
                    text: 10000
                }
            )
            .is_err()
        );
    }
    #[test]
    fn late_key_and_role_matches_consume_each_comparison() {
        for member in ["id", "role"] {
            let values = vec![
                serde_json::json!({member: "aa"}),
                serde_json::json!({member: "bb"}),
                serde_json::json!({member: "cc"}),
            ];
            assert!(matching(&values, member, "cc", &mut Budget { work: 2, text: 6 }).is_err());
            assert!(matching(&values, member, "cc", &mut Budget { work: 3, text: 5 }).is_err());
            let mut b = Budget { work: 3, text: 6 };
            assert!(matching(&values, member, "cc", &mut b).is_ok());
            assert_eq!((b.work, b.text), (0, 0));
        }
        with_case(
            "SELECT r.salary FROM Resource r",
            |_| {},
            |v| {
                home(v, "Resource")["source"]["discriminator"] =
                    json!({"column":"tag","codec":INT,"literal":{"integerToken":"0"}});
                v["queryCarriers"][0]["source"] = home(v, "Resource")["source"].clone();
            },
            |ctx, m| assert!(check(ctx, m).is_ok()),
        );
    }
    #[test]
    fn single_budget_refuses_before_partial_correspondence() {
        with_case(
            "SELECT r.salary FROM Resource r",
            |_| {},
            |_| {},
            |ctx, m| {
                let raw = ctx.binding_json();
                let duplicate = format!("{{\"version\":\"{PROFILE}\",{}", &raw[1..]);
                assert!(
                    parse_binding(
                        &duplicate,
                        &mut Budget {
                            work: 1_000_000,
                            text: 16_000_000
                        }
                    )
                    .is_err()
                );
                let mut b = Budget {
                    work: 1_000_000,
                    text: 16_000_000,
                };
                check_budget(ctx, m, &mut b).unwrap();
                let w = 1_000_000 - b.work;
                let t = 16_000_000 - b.text;
                assert!(check_budget(ctx, m, &mut Budget { work: w, text: t }).is_ok());
                for (w, t) in [(w - 1, t), (w, t - 1), (0, t), (w, 0)] {
                    assert!(check_budget(ctx, m, &mut Budget { work: w, text: t }).is_err());
                }
            },
        );
    }
}
