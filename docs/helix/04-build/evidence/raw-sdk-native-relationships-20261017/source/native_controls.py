"""Native owner-admission controls; no SDK end-to-end qualification claim.

Run only inside an explicitly leased, owned PostgreSQL18 fixture cluster.
UMF_PG_TEST_DSN must name a nonsuperuser, NOBYPASSRLS role with CREATE schema.
"""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import uuid

import psycopg
from psycopg import sql
import weft

ROOT = Path(__file__).resolve().parent
MODULE_SHA = "41a907705278dd80f07c1761303948fb24cb59ab57e7d4f57695836e18518583"
checks: list[dict[str, object]] = []
schemas: list[str] = []


def compile_program(name: str, schema: str) -> dict:
    request = json.loads((ROOT / f"{name}-request.json").read_text())
    request["binding"]["schema"] = schema
    program = json.loads(weft.compile_raw_read_json(json.dumps(request)))
    assert program["status"] == "compiled", program
    (ROOT / f"actual-{schema}-program.json").write_text(json.dumps(program, indent=2) + "\n")
    return program


def admissions(program: dict) -> list[tuple[str, dict]]:
    obligations = program["obligations"]
    entries = [("relationship", x["nativeAdmission"]) for x in obligations["nativeRelationships"]["relationships"]]
    entries += [("alternate-key", x["nativeAdmission"]) for x in obligations.get("nativeRelationshipKeys", {}).get("keys", [])]
    return entries


def admit(connection: psycopg.Connection, program: dict) -> list[tuple[str, bool]]:
    results = []
    for kind, admission in admissions(program):
        values = {}
        for item in admission["parameters"]:
            assert set(item) == {"name", "source", "carrier"} and item["carrier"] == "string"
            assert set(item["source"]) == {"literal"} and set(item["source"]["literal"]) == {"string"}
            values[item["name"]] = item["source"]["literal"]["string"]
        assert list(values) == admission["dbapiParameterOrder"]
        rows = connection.execute(admission["dbapiSql"], values).fetchmany(2)
        assert len(rows) == 1 and len(rows[0]) == 1 and type(rows[0][0]) is bool, rows
        results.append((kind, rows[0][0]))
    return results


def check(connection: psycopg.Connection, name: str, program: dict, expected: bool,
          *, isolation: str = "REPEATABLE READ", readonly: bool = True, execute: bool = False) -> None:
    assert isolation in {"REPEATABLE READ", "READ COMMITTED"}
    with connection.transaction():
        connection.execute(f"SET TRANSACTION ISOLATION LEVEL {isolation} {'READ ONLY' if readonly else 'READ WRITE'}")
        before = admit(connection, program)
        assert all(value is expected for _, value in before), (name, before, expected)
        row_count = None
        if execute:
            assert expected and not program["parameterOrder"]
            # Exactly original owner SQL, with no placeholder or SQL rewriting.
            row_count = len(connection.execute(program["sql"]).fetchall())
        after = admit(connection, program)
        assert after == before
    checks.append({"name": name, "expected": expected, "before": before, "after": after, "rowCount": row_count})


def ddl(connection: psycopg.Connection, statement: str, schema: str) -> None:
    connection.execute(sql.SQL(statement).format(s=sql.Identifier(schema)))


def new_schema(connection: psycopg.Connection, label: str) -> str:
    schema = f"weft_r13_{label}_{uuid.uuid4().hex[:8]}"
    ddl(connection, "CREATE SCHEMA {s}", schema)
    schemas.append(schema)
    return schema


def clone_opclass(connection: psycopg.Connection, schema: str) -> None:
    # PostgreSQL restricts operator-class creation to superusers. This separate
    # owned-cluster admin connection creates only this temporary fixture object;
    # every semantic admission runs as the constrained principal above.
    with psycopg.connect(os.environ["UMF_PG_ADMIN_DSN"], autocommit=True) as admin:
        ddl(admin, """CREATE OPERATOR CLASS {s}.text_clone_ops FOR TYPE text USING btree AS
          OPERATOR 1 pg_catalog.< (text,text), OPERATOR 2 pg_catalog.<= (text,text),
          OPERATOR 3 pg_catalog.= (text,text), OPERATOR 4 pg_catalog.>= (text,text),
          OPERATOR 5 pg_catalog.> (text,text), FUNCTION 1 pg_catalog.bttextcmp(text,text)""", schema)


def alternate_tables(connection: psycopg.Connection, schema: str, *, casefold: bool = False,
                     custom_target: bool = False, custom_source: bool = False) -> None:
    if casefold:
        ddl(connection, "CREATE COLLATION {s}.casefold (provider=icu,locale='und-u-ks-level2',deterministic=false)", schema)
        collation = sql.SQL("{s}.casefold").format(s=sql.Identifier(schema))
    else:
        collation = sql.SQL('pg_catalog."C"')
    connection.execute(sql.SQL("CREATE TABLE {s}.customers(id bigint PRIMARY KEY,name text NOT NULL,code text COLLATE {c} NOT NULL)").format(s=sql.Identifier(schema), c=collation))
    connection.execute(sql.SQL("CREATE TABLE {s}.orders(id bigint PRIMARY KEY,customer_id bigint,status text NOT NULL,amount numeric NOT NULL,customer_code text COLLATE {c} NOT NULL)").format(s=sql.Identifier(schema), c=collation))
    if custom_target or custom_source:
        clone_opclass(connection, schema)
    ddl(connection, "CREATE UNIQUE INDEX customers_code_key ON {s}.customers(code)", schema) if not custom_target else ddl(connection, "CREATE UNIQUE INDEX customers_code_key ON {s}.customers(code {s}.text_clone_ops)", schema)
    if custom_source:
        ddl(connection, "CREATE UNIQUE INDEX orders_code_key ON {s}.orders(customer_code {s}.text_clone_ops)", schema)
    ddl(connection, "ALTER TABLE {s}.orders ADD CONSTRAINT orders_customer_fk FOREIGN KEY(customer_code) REFERENCES {s}.customers(code) NOT DEFERRABLE", schema)
    ddl(connection, "INSERT INTO {s}.customers VALUES(1,'one','Alpha'),(2,'two','Beta')", schema)
    source = "alpha" if casefold else "Alpha"
    connection.execute(sql.SQL("INSERT INTO {s}.orders VALUES(10,1,'open',1.0,%s),(20,2,'open',2.0,'Beta')").format(s=sql.Identifier(schema)), (source,))


def degree_tables(connection: psycopg.Connection, schema: str) -> None:
    ddl(connection, "CREATE TABLE {s}.customers(id bigint PRIMARY KEY,name text NOT NULL)", schema)
    ddl(connection, "CREATE TABLE {s}.orders(id bigint PRIMARY KEY,customer_id bigint REFERENCES {s}.customers(id) NOT DEFERRABLE,status text NOT NULL,amount numeric NOT NULL)", schema)
    ddl(connection, "INSERT INTO {s}.customers VALUES(1,'one'),(2,'two')", schema)
    ddl(connection, "INSERT INTO {s}.orders VALUES(10,1,'open',1),(11,1,'open',1),(20,2,'open',1),(30,NULL,'open',1)", schema)


def main() -> None:
    native = next(Path(weft.__file__).parent.glob("weft*.so"))
    assert hashlib.sha256(native.read_bytes()).hexdigest() == MODULE_SHA, "exact R13 native module required"
    with psycopg.connect(os.environ["UMF_PG_TEST_DSN"], autocommit=True) as connection:
        principal = connection.execute("SELECT rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user").fetchone()
        assert principal == (False, False), "owned constrained fixture principal required"
        completed = False
        try:
            schema = new_schema(connection, "binary")
            alternate_tables(connection, schema)
            check(connection, "deterministic-default-opclass-positive", compile_program("alternate", schema), True, execute=True)

            schema = new_schema(connection, "casefold")
            alternate_tables(connection, schema, casefold=True)
            check(connection, "native-casefold-fk-and-alternate-key-refused", compile_program("alternate", schema), False)

            schema = new_schema(connection, "custom_target")
            alternate_tables(connection, schema, custom_target=True)
            check(connection, "native-custom-target-opclass-refused", compile_program("alternate", schema), False)

            schema = new_schema(connection, "custom_source")
            alternate_tables(connection, schema, custom_source=True)
            program = compile_program("inverse-unique", schema)
            # Target alternate Key remains valid; only the source max-one proof refuses.
            with connection.transaction():
                connection.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
                observed = admit(connection, program)
                assert observed == [("relationship", False), ("alternate-key", True)], observed
            checks.append({"name": "native-custom-inverse-unique-opclass-refused", "observed": observed})

            schema = new_schema(connection, "degree")
            degree_tables(connection, schema)
            program = compile_program("degree", schema)
            assert program["obligations"]["readIsolation"] == "repeatable-read"
            check(connection, "finite-degree-positive-zero-fk-not-an-edge", program, True, execute=True)
            check(connection, "finite-degree-read-committed-refused", program, False, isolation="READ COMMITTED")
            check(connection, "finite-degree-read-write-refused", program, False, readonly=False)
            ddl(connection, "DELETE FROM {s}.orders WHERE id=20", schema)
            check(connection, "finite-degree-zero-neighbor-target-refused", program, False)
            ddl(connection, "INSERT INTO {s}.orders VALUES(20,2,'open',1),(12,1,'open',1)", schema)
            check(connection, "finite-degree-excess-neighbor-target-refused", program, False)
            ddl(connection, "DELETE FROM {s}.orders WHERE id=12", schema)
            ddl(connection, "ALTER TABLE {s}.customers ENABLE ROW LEVEL SECURITY", schema)
            ddl(connection, "ALTER TABLE {s}.customers FORCE ROW LEVEL SECURITY", schema)
            ddl(connection, "CREATE POLICY only_first ON {s}.customers USING(id=1)", schema)
            check(connection, "finite-degree-filtered-native-rls-scope-refused", program, False)
            ddl(connection, "ALTER TABLE {s}.customers DISABLE ROW LEVEL SECURITY", schema)
            check(connection, "finite-degree-restored-positive", program, True, execute=True)
            assert len(checks) == 11
            completed = True
        finally:
            for schema in reversed(schemas):
                ddl(connection, "DROP SCHEMA {s} CASCADE", schema)
            remaining = connection.execute("SELECT nspname FROM pg_catalog.pg_namespace WHERE nspname = ANY(%s)", (schemas,)).fetchall()
            assert not remaining, remaining
            (ROOT / "native-result.json").write_text(json.dumps({"status": "passed" if completed else "failed", "checks": checks, "checkCount": len(checks), "schemasRemoved": True, "moduleSha256": MODULE_SHA, "temporal": "HELD", "scope": "original owner Boolean admission and unchanged SQL; not SDK E2E"}, indent=2) + "\n")


if __name__ == "__main__":
    main()
