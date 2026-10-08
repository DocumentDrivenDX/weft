// @covers US-006-AC2 @covers US-006-AC3 @covers US-006-AC4
// Deterministic frontend properties; no native backend/embedding qualification.
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngAlgorithm, RngSeed, TestRunner};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::{atomic::{AtomicUsize, Ordering}, Mutex};
use weft_core::{frontend_json, json::sha256};
fn base() -> Value {
    let cases: Value = serde_json::from_str(include_str!("../../docs/helix/03-test/fixtures/cases.json")).unwrap();
    cases[0]["request"].clone()
}
fn edit(r: &mut Value, f: impl FnOnce(&mut Value)) {
    let mut d: Value = serde_json::from_str(r["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
    f(&mut d); let raw = d.to_string();
    r["modules"][0]["pin"]["sha256"] = json!(sha256(raw.as_bytes()));
    r["modules"][0]["documentJson"] = json!(raw);
}
fn field<'a>(d: &'a mut Value, id: &str) -> &'a mut Value {
    d["modules"][0]["elements"].as_array_mut().unwrap().iter_mut().find(|e| e["id"]==id).unwrap()
}
fn run(r: &Value) -> Value { serde_json::from_str(&frontend_json(&r.to_string())).unwrap() }
fn config(cases: u32, seed: u64) -> Config {
    Config { cases, rng_algorithm: RngAlgorithm::ChaCha, rng_seed: RngSeed::Fixed(seed),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(concat!(env!("CARGO_MANIFEST_DIR"),"/../../tests/qualify-and-evolve/property-regressions.txt")))),
        source_file: Some(file!()), max_shrink_iters: 10000, ..Config::default() }
}
fn report(name: &str, cases: usize, seed: u64, unique: usize) {
    println!("PROPERTY_REPORT {}",json!({"property":name,"successfulGeneratedCases":cases,"seed":seed,"rng":"ChaCha","proptest":"1.11.0","generatorVersion":"weft-properties/0.1.0","uniqueRequestDigests":unique,"layer":"frontend"}));
}
#[test]
fn generated_integer_domains() {
    let seed=0x574546540701; let count=AtomicUsize::new(0); let unique=Mutex::new(BTreeSet::new());
    TestRunner::new(config(4200,seed)).run(&(1u32..=64,any::<bool>(),any::<u128>(),0u8..4), |(bits,signed,raw,mode)| {
        // Independent mathematical bounds use i128; no compiler arithmetic helper.
        let min=if signed {-(1i128<<(bits-1))} else {0};
        let max=if signed {(1i128<<(bits-1))-1} else {(1i128<<bits)-1};
        let n=match mode {0=>min+(raw%(1u128<<bits)) as i128,1=>max,2=>min-1,_=>max+1};
        let valid=n>=min&&n<=max;
        let mut request=base();edit(&mut request,|d|field(d,"customer-id")["facets"]=json!({"integerWidth":{"bits":bits,"signed":signed}}));
        request["sql"]=json!(format!("SELECT c.id FROM Customer c WHERE c.id = {n}"));
        unique.lock().unwrap().insert(sha256(request.to_string().as_bytes()));
        let response=run(&request);
        prop_assert_eq!(&response["status"],if valid {"resolved"} else {"blocked"});
        if valid {prop_assert_eq!(&response["logicalPlan"]["root"]["input"]["predicate"]["right"]["value"], &json!(n.to_string()));}
        else {prop_assert_eq!(&response["diagnostics"][0]["code"],"WFT-NUMERIC-DOMAIN");prop_assert!(response.get("logicalPlan").is_none());}
        count.fetch_add(1,Ordering::SeqCst);Ok(())
    }).unwrap();
    assert_eq!(count.load(Ordering::SeqCst),4200);report("integer-domains",4200,seed,unique.lock().unwrap().len());
}
fn decimal(coefficient:u128,scale:u32,negative:bool)->String {
    let factor=10u128.pow(scale);let sign=if negative {"-"} else {""};
    if scale==0 {format!("{sign}{coefficient}")} else {format!("{sign}{}.{:0width$}",coefficient/factor,coefficient%factor,width=scale as usize)}
}
#[test]
fn generated_decimal_domains() {
    let seed=0x574546540702;let count=AtomicUsize::new(0);let unique=Mutex::new(BTreeSet::new());
    TestRunner::new(config(3200,seed)).run(&(1u32..=28,any::<u32>(),any::<u128>(),any::<bool>(),0u8..5),|(precision,s,raw,negative,mode)| {
        let scale=s%(precision+1);let range=10u128.pow(precision);let coefficient=raw%range;
        let (text,valid)=match mode {
            0=>(decimal(coefficient,scale,negative),true),
            1=>(decimal(range-1,scale,negative),true),
            2=>(decimal(range,scale,negative),false),
            3=>{let coefficient=if coefficient%10==0 {coefficient+1} else {coefficient};(decimal(coefficient,scale+1,negative),false)},
            _=>{let text=decimal(coefficient,scale,negative);(format!("{text}{}",if scale==0 {".0"} else {"0"}),true)}
        };
        let mut request=base();edit(&mut request,|d|field(d,"order-total")["facets"]=json!({"precision":precision,"scale":scale}));
        request["sql"]=json!(format!("SELECT o.total FROM Orders o WHERE o.total = {text}"));unique.lock().unwrap().insert(sha256(request.to_string().as_bytes()));
        let response=run(&request);prop_assert_eq!(&response["status"],if valid {"resolved"} else {"blocked"});
        if valid {prop_assert_eq!(&response["logicalPlan"]["root"]["input"]["predicate"]["right"]["value"],&json!(text));}
        else {prop_assert_eq!(&response["diagnostics"][0]["code"],"WFT-NUMERIC-DOMAIN");prop_assert!(response.get("logicalPlan").is_none());}
        count.fetch_add(1,Ordering::SeqCst);Ok(())
    }).unwrap();assert_eq!(count.load(Ordering::SeqCst),3200);report("decimal-domains",3200,seed,unique.lock().unwrap().len());
}
fn text()->impl Strategy<Value=String> {
    proptest::collection::vec(proptest::sample::select(vec!['é','e','\u{301}','😀','\'','"','\\','\n',' ','x',';','-']),0..16).prop_map(|chars|chars.into_iter().collect())
}
#[test]
fn generated_unicode_retention_pins_and_selected_meaning() {
    let seed=0x574546540703;let count=AtomicUsize::new(0);let unique=Mutex::new(BTreeSet::new());
    TestRunner::new(config(1600,seed)).run(&(text(),any::<u128>()),|(text,opaque)| {
        let mut request=base();edit(&mut request,|d|d["extensions"]["future.vendor"]=json!({"text":text,"number":opaque.to_string(),"retained":[null,false,{"new":"unknown"}]}));
        request["sql"]=json!(format!("SELECT c.name FROM Customer c WHERE c.name = '{}'",text.replace('\'',"''")));
        unique.lock().unwrap().insert(sha256(request.to_string().as_bytes()));
        let response=run(&request);prop_assert_eq!(&response["status"],"resolved");prop_assert_eq!(&response["retainedModules"],&request["modules"]);
        prop_assert_eq!(&response["logicalPlan"]["root"]["input"]["predicate"]["right"]["value"],&json!(text));
        let mut stale=request.clone();let raw=format!("{} ",stale["modules"][0]["documentJson"].as_str().unwrap());stale["modules"][0]["documentJson"]=json!(raw);
        let refused=run(&stale);prop_assert_eq!(&refused["diagnostics"][0]["code"],"WFT-PIN");prop_assert!(refused.get("logicalPlan").is_none());
        edit(&mut request,|d|field(d,"customer-name")["facets"]=json!({"futureMeaning":opaque.to_string()}));
        let refused=run(&request);prop_assert_eq!(&refused["diagnostics"][0]["code"],"WFT-TYPE");prop_assert!(refused.get("logicalPlan").is_none());
        count.fetch_add(1,Ordering::SeqCst);Ok(())
    }).unwrap();assert_eq!(count.load(Ordering::SeqCst),1600);report("unicode-retention-pins-meaning",1600,seed,unique.lock().unwrap().len());
}
#[test]
fn generated_hostile_json_and_multi_statement_inputs() {
    let seed=0x574546540704;let count=AtomicUsize::new(0);let unique=Mutex::new(BTreeSet::new());
    TestRunner::new(config(1000,seed)).run(&(text(),any::<u64>()),|(key,n)| {
        let key=serde_json::to_string(&key).unwrap();let duplicate=format!("{{{key}:{n},{key}:0}}");unique.lock().unwrap().insert(sha256(duplicate.as_bytes()));
        prop_assert_eq!(weft_core::json::checked_json(&duplicate).unwrap_err(),"WFT-JSON-DUPLICATE");
        let response:Value=serde_json::from_str(&weft_core::compile::Compiler::default().compile_json(&duplicate)).unwrap();
        prop_assert_eq!(&response["status"],"blocked");prop_assert!(response.get("sql").is_none());prop_assert!(response.get("parameters").is_none());
        let mut request=base();request["sql"]=json!(format!("SELECT c.id FROM Customer c; DROP TABLE malicious_{n}"));
        let refused=run(&request);prop_assert_eq!(&refused["status"],"blocked");prop_assert!(refused.get("logicalPlan").is_none());
        count.fetch_add(1,Ordering::SeqCst);Ok(())
    }).unwrap();assert_eq!(count.load(Ordering::SeqCst),1000);report("hostile-json-multi-statement",1000,seed,unique.lock().unwrap().len());
}

#[test]
fn generated_sql_parser_text_and_spans() {
    let seed=0x574546540706;
    let successes=AtomicUsize::new(0);let refusals=AtomicUsize::new(0);
    TestRunner::new(config(5000,seed)).run(&(text(),0u8..4,any::<u64>()),|(text,mode,nonce)| {
        let sql=match mode {
            0=>format!("{text} {nonce}"),
            1=>format!("SELECT c.name FROM Customer c WHERE c.name = '{}'",text.replace('\'',"''")),
            2=>format!("SELECT c.name FROM Customer c WHERE c.name = '{text}{nonce}"),
            _=>format!("SELECT c.name FROM Customer c; {nonce} {text}"),
        };
        let first=weft_core::syntax::parse(&sql);
        let second=weft_core::syntax::parse(&sql);
        prop_assert_eq!(format!("{first:?}"),format!("{second:?}"));
        match first {
            Ok(_)=>{successes.fetch_add(1,Ordering::SeqCst);},
            Err(d)=>{
                if let Some(span)=d.source_span {
                    prop_assert!(span.start<=span.end&&span.end<=sql.len());
                    prop_assert!(sql.is_char_boundary(span.start)&&sql.is_char_boundary(span.end));
                }
                refusals.fetch_add(1,Ordering::SeqCst);
            }
        }
        Ok(())
    }).unwrap();
    assert_eq!(successes.load(Ordering::SeqCst)+refusals.load(Ordering::SeqCst),5000);
    assert!(successes.load(Ordering::SeqCst)>0&&refusals.load(Ordering::SeqCst)>0);
    println!("PARSER_REPORT {}",json!({"generator":"weft-parser-text/0.1.0","proptest":"1.11.0","seed":seed,"cases":5000,"accepted":successes.load(Ordering::SeqCst),"refused":refusals.load(Ordering::SeqCst),"scope":"syntax parse determinism, no panic and UTF-8 diagnostic spans; no resolution or backend qualification"}));
}
