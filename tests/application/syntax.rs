use weft_core::application_syntax::{parse, Output, Predicate, Value};
#[test]
fn application_constructs_preserve_structure() {
    let q = parse("SELECT c.*, RELATED_KEYS(c.orders, 20) AS orders FROM Customer c WHERE HAS_RELATED(c.orders, KEY(:order_id)) AND (c.region,c.id) > ('east',:cursor) ORDER BY c.region ASC,c.id ASC LIMIT 1000;").unwrap();
    assert!(matches!(&q.outputs[0].output, Output::Entity(n) if n.value == "c"));
    assert!(matches!(
        &q.outputs[1].output,
        Output::Related { bound: 20, .. }
    ));
    assert!(
        matches!(&q.predicates[0], Predicate::HasRelated {key,..} if matches!(&key[0],Value::Parameter(n) if n.value=="order_id"))
    );
    assert!(
        matches!(&q.predicates[1], Predicate::Compare {columns,values,greater:true} if columns.len()==2 && values.len()==2)
    );
    assert_eq!(q.order.len(), 2);
    assert_eq!(q.limit, Some(1000));
}
#[test]
fn count_and_literal_values() {
    let q=parse("SELECT c.name,COUNT(*) AS total FROM Customer c WHERE c.name = 'O''Brien' GROUP BY c.name ORDER BY c.name LIMIT 10").unwrap();
    assert!(matches!(q.outputs[1].output, Output::Count));
    assert_eq!(q.groups.len(), 1);
    assert!(
        matches!(&q.predicates[0],Predicate::Compare{values,..} if matches!(&values[0],Value::Literal(l) if l.value=="O'Brien"))
    );
}
#[test]
fn excluded_and_malformed_constructs_refuse() {
    for sql in [
        "SELECT * FROM Customer c",
        "SELECT c.* AS entity FROM Customer c",
        "SELECT COUNT(c.id) FROM Customer c",
        "SELECT COUNT(DISTINCT c.id) FROM Customer c",
        "SELECT c.id FROM Customer c ORDER BY c.id DESC LIMIT 10",
        "SELECT c.id FROM Customer c LIMIT 0",
        "SELECT c.id FROM Customer c LIMIT 1001",
        "SELECT c.id FROM Customer c LIMIT 1.0",
        "SELECT c.id FROM Customer c LIMIT -1",
        "SELECT c.id FROM Customer c WHERE (c.id,c.region) > (1)",
        "SELECT c.id FROM Customer c WHERE (c.id,c.region) = (1,2)",
        "SELECT c.id FROM Customer c WHERE c.id > :",
        "SELECT c.id FROM Customer c WHERE c.id > :x OR c.id = 2",
        "SELECT RELATED_KEYS(c.orders,0) FROM Customer c",
        "SELECT c.id FROM Customer c LIMIT 10 OFFSET 2",
        "SELECT c.id FROM Customer c; SELECT c.id FROM Customer c",
    ] {
        assert!(parse(sql).is_err(), "{sql}");
    }
}
#[test]
fn explicit_version_does_not_expand_original_parser() {
    for sql in [
        "SELECT c.* FROM Customer c",
        "SELECT COUNT(*) FROM Customer c",
        "SELECT c.id FROM Customer c WHERE c.id > :id ORDER BY c.id LIMIT 10",
    ] {
        assert!(weft_core::syntax::parse(sql).is_err());
        assert!(parse(sql).is_ok());
    }
}
