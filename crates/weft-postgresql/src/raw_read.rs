//! Experimental raw PostgreSQL 18 reads. No connection or execution authority.
//! WR-B7 / WR-A14: one schema, declared to-one joins, closed read-only SQL AST.
use serde::Serialize;
use serde_json::{json, Value as Json};
use sqlparser::ast::*;
use std::collections::{BTreeMap, BTreeSet};
use weft_core::raw_read::{AdmittedRead, Carrier, Parameter, Request, Table};
#[path="raw_read_projection.rs"]
mod native_projection;

trait PgCarrier {
    fn sql(&self) -> &str;
}
impl PgCarrier for Carrier {
    fn sql(&self) -> &str {
        match self {
            Self::Integer => "numeric",
            Self::Decimal => "numeric",
            Self::String => "text",
            Self::Boolean => "bool",
        }
    }
}

type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Output {
    name: String,
    carrier: Json,
    nullable: bool,
    #[serde(flatten)]
    metadata: serde_json::Map<String,Json>,
}
#[derive(Clone)]
struct Scalar {
    sql: String,
    carrier: Carrier,
    nullable: bool,
    aggregate: bool,
    fields: BTreeSet<(String, String)>,
}
struct Lower<'a> {
    admitted: &'a AdmittedRead,
    term_indexes: BTreeMap<String,Json>,
    used_columns: std::cell::RefCell<BTreeSet<(String,String)>>,
    used_functions: std::cell::RefCell<BTreeSet<String>>,
    used_operators: std::cell::RefCell<BTreeSet<String>>,
    request: &'a Request,
    scans: BTreeMap<String, (&'a Table, bool)>,
    slots: Vec<String>,
    used: BTreeSet<String>,
    group_values: Option<BTreeSet<String>>,
}
fn ident(s: &str) -> Result<String> {
    if s.is_empty() || s.len() > 63 || s.contains('\0') {
        return Err("identifier outside PostgreSQL profile");
    }
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}
fn name(i: &Ident) -> String {
    if i.quote_style.is_some() {
        i.value.clone()
    } else {
        i.value.to_ascii_lowercase()
    }
}
fn object(n: &ObjectName) -> Result<Vec<String>> {
    n.0.iter()
        .map(|p| match p {
            ObjectNamePart::Identifier(i) => Ok(name(i)),
            _ => Err("dynamic object name"),
        })
        .collect()
}
fn term_index_metadata(index:&weft_core::search::AdmittedSearch,table:&Table,column:&weft_core::raw_read::Column,schema:&Option<String>,configuration:&str)->Result<Json> {
    let language=configuration.strip_prefix("pg_catalog.").ok_or("term configuration namespace")?;
    if !matches!(language,"simple"|"english") {return Err("term analyzer unsupported")};
    let aliases=["asciiword","word","numword","email","url","host","sfloat","version","hword_numpart","hword_part","hword_asciipart","numhword","asciihword","hword","url_path","file","float","int","uint"];
    let mut expected=aliases.iter().map(|alias|{
        let stem=language=="english"&&matches!(*alias,"asciiword"|"word"|"hword_part"|"hword_asciipart"|"asciihword"|"hword");
        json!([alias,1,"pg_catalog",if stem{"english_stem"}else{"simple"},"pg_catalog",if stem{"snowball"}else{"simple"},if stem{json!("language = 'english', stopwords = 'english'")}else{Json::Null}])
    }).collect::<Vec<_>>();
    expected.sort_by(|a,b|a[0].as_str().unwrap().cmp(b[0].as_str().unwrap()));
    let expected=Json::Array(expected).to_string().replace('\'',"''");
    let render=|dbapi:bool| {
        let slot=|n:u8|if dbapi{format!("%(__umf_term_{n})s::pg_catalog.text")}else{format!("${n}::pg_catalog.text")};
        let app=slot(1);let index=slot(2);let table=slot(3);let language=slot(4);let column=slot(5);
        let config=format!("EXISTS(SELECT 1 FROM pg_catalog.pg_ts_config AS cfg JOIN pg_catalog.pg_namespace AS cn ON cn.oid=cfg.cfgnamespace JOIN pg_catalog.pg_ts_parser AS parser ON parser.oid=cfg.cfgparser JOIN pg_catalog.pg_namespace AS pn ON pn.oid=parser.prsnamespace WHERE cn.nspname='pg_catalog' AND cfg.cfgname={language} AND pn.nspname='pg_catalog' AND parser.prsname='default' AND (SELECT pg_catalog.jsonb_agg(pg_catalog.jsonb_build_array(tt.alias,m.mapseqno,dn.nspname,d.dictname,tn.nspname,template.tmplname,d.dictinitoption) ORDER BY tt.alias COLLATE pg_catalog.\"C\",m.mapseqno) FROM pg_catalog.pg_ts_config_map AS m JOIN pg_catalog.ts_token_type(parser.oid) AS tt ON tt.tokid=m.maptokentype JOIN pg_catalog.pg_ts_dict AS d ON d.oid=m.mapdict JOIN pg_catalog.pg_namespace AS dn ON dn.oid=d.dictnamespace JOIN pg_catalog.pg_ts_template AS template ON template.oid=d.dicttemplate JOIN pg_catalog.pg_namespace AS tn ON tn.oid=template.tmplnamespace WHERE m.mapcfg=cfg.oid)='{expected}'::pg_catalog.jsonb)");
        let pattern=if dbapi{"to_tsvector(%%L::regconfig, COALESCE(%%I, %%L::text))"}else{"to_tsvector(%L::regconfig, COALESCE(%I, %L::text))"};
        format!("SELECT (i.indisvalid AND i.indisready AND i.indislive AND i.indnkeyatts=1 AND i.indnatts=1 AND i.indpred IS NULL AND am.amname='gin' AND opc.opcname='tsvector_ops' AND opns.nspname='pg_catalog' AND i.indkey[0]=0 AND i.indexprs IS NOT NULL AND pg_catalog.pg_get_indexdef(i.indexrelid,1,true)=pg_catalog.format('{pattern}',{language},{column},''::pg_catalog.text) AND {config}) AS correspondence FROM pg_catalog.pg_index AS i JOIN pg_catalog.pg_class AS idx ON idx.oid=i.indexrelid JOIN pg_catalog.pg_namespace AS ins ON ins.oid=idx.relnamespace JOIN pg_catalog.pg_class AS t ON t.oid=i.indrelid JOIN pg_catalog.pg_namespace AS ns ON ns.oid=t.relnamespace JOIN pg_catalog.pg_am AS am ON am.oid=idx.relam JOIN pg_catalog.pg_opclass AS opc ON opc.oid=i.indclass[0] JOIN pg_catalog.pg_namespace AS opns ON opns.oid=opc.opcnamespace WHERE ins.nspname={app} AND ns.nspname={app} AND idx.relname={index} AND t.relname={table}")
    };
    let literal=|text:&str|json!({"source":{"literal":{"string":text}},"carrier":"string"});
    let schema_parameter=match schema {Some(s)=>literal(s),None=>json!({"source":{"appSchema":true},"carrier":"string"})};
    let mut parameters=vec![schema_parameter,literal(index.binding()["native"].as_str().ok_or("term native index required")?),literal(&table.native),literal(language),literal(&column.native)];
    for (i,p) in parameters.iter_mut().enumerate(){p["name"]=json!(format!("__umf_term_{}",i+1));}
    let names=parameters.iter().map(|p|p["name"].clone()).collect::<Vec<_>>();
    Ok(json!({"source":index.request().source,"index":index.index(),"nativeBinding":index.binding(),"logicalTable":table.logical,"nativeTable":table.native,"nativeColumn":column.native,"configuration":configuration,"nativeAdmission":{"sql":render(false),"dbapiSql":render(true),"dbapiParameterOrder":names,"parameters":parameters}}))
}
impl<'a> Lower<'a> {
    fn table(&mut self, f: &TableFactor, left: bool) -> Result<(String, String)> {
        let TableFactor::Table {
            name: n,
            alias: Some(alias),
            args: None,
            with_hints,
            version: None,
            with_ordinality: false,
            partitions,
            json_path: None,
            sample: None,
            index_hints,
        } = f
        else {
            return Err("only aliased bound tables are supported");
        };
        if !with_hints.is_empty()
            || !partitions.is_empty()
            || !index_hints.is_empty()
            || !alias.columns.is_empty()
        {
            return Err("table modifiers");
        }
        let logical = object(n)?;
        let table = self
            .request
            .binding
            .tables
            .iter()
            .find(|t| t.logical == logical)
            .ok_or("unbound table")?;
        let a = name(&alias.name);
        if self.scans.contains_key(&a) {
            return Err("duplicate alias");
        }
        self.scans.insert(a.clone(), (table, left));
        let native = match &self.request.binding.schema {
            Some(schema) => format!("{}.{}", ident(schema)?, ident(&table.native)?),
            None => ident(&table.native)?,
        };
        Ok((a.clone(), format!("{} AS {}", native, ident(&a)?)))
    }
    fn column(&self, e: &Expr) -> Result<Scalar> {
        let Expr::CompoundIdentifier(parts) = e else {
            return Err("columns require explicit table aliases");
        };
        if parts.len() != 2 {
            return Err("column path");
        }
        let a = name(&parts[0]);
        let f = name(&parts[1]);
        let (t, left) = self.scans.get(&a).ok_or("unknown alias")?;
        let c = t
            .columns
            .iter()
            .find(|c| c.logical == f)
            .ok_or("unknown column")?;
        self.used_columns.borrow_mut().insert((a.clone(),f.clone()));
        Ok(Scalar {
            sql: if c.carrier==Carrier::String {format!("{}.{} COLLATE pg_catalog.\"C\"", ident(&a)?,ident(&c.native)?)} else {format!("{}.{}", ident(&a)?, ident(&c.native)?)},
            carrier: c.carrier.clone(),
            nullable: *left || c.nullable,
            aggregate: false,
            fields: BTreeSet::from([(a, f)]),
        })
    }
    fn value(&mut self, e: &Expr, aggregates: bool) -> Result<Scalar> {
        if let Expr::Nested(inner) = e {
            return self.value(inner, aggregates);
        }
        if matches!(e, Expr::CompoundIdentifier(_)) {
            return self.column(e);
        }
        if let Expr::Value(v) = e {
            if let Value::Placeholder(p) = &v.value {
                let pname = p.strip_prefix(':').ok_or("only named bound parameters")?;
                let parameter = self
                    .request
                    .parameters
                    .iter()
                    .find(|x| x.name == pname)
                    .ok_or("undeclared parameter")?;
                self.used.insert(pname.to_owned());
                self.slots.push(pname.to_owned());
                return Ok(Scalar {
                    sql: format!(
                        "${}::pg_catalog.{}",
                        self.slots.len(),
                        parameter.carrier.sql()
                    ),
                    carrier: parameter.carrier.clone(),
                    nullable: parameter.nullable,
                    aggregate: false,
                    fields: BTreeSet::new(),
                });
            }
        }
        if let Expr::Value(v)=e { match &v.value {
            Value::SingleQuotedString(text)=>{if text.contains('\0'){return Err("native string literal contains NUL")};return Ok(Scalar{sql:format!("(E'{}' COLLATE pg_catalog.\"C\")",text.replace('\\',"\\\\").replace('\'',"''")),carrier:Carrier::String,nullable:false,aggregate:false,fields:BTreeSet::new()})},
            Value::Boolean(value)=>return Ok(Scalar{sql:if *value{"TRUE"}else{"FALSE"}.into(),carrier:Carrier::Boolean,nullable:false,aggregate:false,fields:BTreeSet::new()}),
            Value::Number(token,false)=>{let integer=token.bytes().all(|c|c.is_ascii_digit()||matches!(c,b'-'|b'+'));return Ok(Scalar{sql:format!("({token})::pg_catalog.numeric"),carrier:if integer{Carrier::Integer}else{Carrier::Decimal},nullable:false,aggregate:false,fields:BTreeSet::new()})},
            _=>{}
        }}
        let Expr::Function(f) = e else {
            return Err("unsupported scalar expression; use a bound parameter");
        };
        if f.uses_odbc_syntax
            || !matches!(f.parameters, FunctionArguments::None)
            || !f.within_group.is_empty()
            || f.filter.is_some()
            || f.null_treatment.is_some()
            || f.over.is_some()
        {
            return Err("function outside read-only aggregate profile");
        }
        let fname = object(&f.name)?;
        if fname.len() != 1 {
            return Err("qualified/custom functions are not admitted");
        }
        let op = fname[0].as_str();
        if op=="bucket_width" {
            let FunctionArguments::List(args)=&f.args else {return Err("bucket arguments")};
            if args.duplicate_treatment.is_some() || !args.clauses.is_empty() || args.args.len()!=2 {return Err("bucket requires a field and positive numeric literal width")};
            let (FunctionArg::Unnamed(FunctionArgExpr::Expr(field)),FunctionArg::Unnamed(FunctionArgExpr::Expr(width)))=(&args.args[0],&args.args[1]) else {return Err("bucket requires a field and positive numeric literal width")};
            let Expr::Value(width_value)=width else {return Err("bucket width must be a positive numeric literal")};
            let Value::Number(token,false)=&width_value.value else {return Err("bucket width must be a positive numeric literal")};
            if token.is_empty() || token.bytes().filter(|c|*c==b'.').count()>1 || !token.bytes().all(|c|c.is_ascii_digit()||c==b'.') || !token.bytes().any(|c|matches!(c,b'1'..=b'9')) {return Err("bucket width must be a positive finite numeric literal")};
            let c=self.column(field)?;let width=self.value(width,false)?;
            if !matches!(c.carrier,Carrier::Integer|Carrier::Decimal) {return Err("numeric bucket field required")};
            let value=format!("{}::pg_catalog.numeric",c.sql);
            self.used_functions.borrow_mut().extend(["pg_catalog.div","pg_catalog.mod"].map(str::to_string));
            self.used_operators.borrow_mut().extend(["-","*","<","<>"].map(str::to_string));
            let quotient=format!("pg_catalog.div({value}, {})",width.sql);
            let remainder=format!("pg_catalog.mod({value}, {})",width.sql);
            let sql=format!("(({quotient} - CASE WHEN {value}<0 AND {remainder}<>0 THEN 1 ELSE 0 END) * {})",width.sql);
            return Ok(Scalar{sql,carrier:if c.carrier==Carrier::Integer&&width.carrier==Carrier::Integer {Carrier::Integer} else {Carrier::Decimal},nullable:c.nullable,aggregate:false,fields:c.fields});
        }
        if !aggregates || !["count", "sum", "avg", "avg_scale", "min", "max"].contains(&op) {
            return Err("function outside read-only aggregate profile");
        }
        let FunctionArguments::List(args) = &f.args else {
            return Err("aggregate arguments");
        };
        if op == "avg_scale" {
            if args.duplicate_treatment.is_some() || !args.clauses.is_empty() || args.args.len()!=2 {
                return Err("AVG_SCALE requires a field and literal result scale");
            }
            let (FunctionArg::Unnamed(FunctionArgExpr::Expr(field)), FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Value(scale)))) = (&args.args[0],&args.args[1]) else {
                return Err("AVG_SCALE requires a field and literal result scale");
            };
            let Value::Number(token,false)=&scale.value else {return Err("AVG_SCALE scale must be an integer literal")};
            if token.is_empty() || !token.bytes().all(|c|c.is_ascii_digit()) {return Err("AVG_SCALE scale must be an integer literal")};
            let scale:u16=token.parse().map_err(|_|"AVG_SCALE scale budget")?;
            if scale>1000 {return Err("AVG_SCALE scale budget")};
            let c=self.column(field)?;
            if !matches!(c.carrier,Carrier::Integer|Carrier::Decimal) {return Err("numeric aggregate input required")};
            self.used_functions.borrow_mut().extend(["pg_catalog.count","pg_catalog.sum","pg_catalog.div","pg_catalog.mod","pg_catalog.abs","pg_catalog.round","pg_catalog.sign"].map(str::to_string));
            self.used_operators.borrow_mut().extend(["+","*",">","="].map(str::to_string));
            let power=format!("1{}", "0".repeat(usize::from(scale)));
            let factor=if scale==0 {"1".into()} else {format!("0.{}1", "0".repeat(usize::from(scale)-1))};
            let count=format!("pg_catalog.count({})::pg_catalog.numeric",c.sql);
            let numerator=format!("(pg_catalog.sum({}::pg_catalog.numeric) * {power}::pg_catalog.numeric)",c.sql);
            let quotient=format!("pg_catalog.div({numerator}, {count})");
            let remainder=format!("pg_catalog.abs(pg_catalog.mod({numerator}, {count}))");
            // Integer quotient and exact remainder select the nearest integer;
            // exact half ties choose its even neighbor for either sign. Decimal
            // multiplication restores the scale without rounded division.
            let sql=format!("CASE WHEN {count}=0 THEN NULL::pg_catalog.numeric ELSE pg_catalog.round(({quotient} + CASE WHEN (2*{remainder})>{count} OR ((2*{remainder})={count} AND pg_catalog.mod(pg_catalog.abs({quotient}),2)=1) THEN pg_catalog.sign({numerator}) ELSE 0 END) * {factor}::pg_catalog.numeric, {scale}) END");
            return Ok(Scalar{sql,carrier:Carrier::Decimal,nullable:true,aggregate:true,fields:c.fields});
        }
        let distinct=match args.duplicate_treatment {
            None=>false,
            Some(DuplicateTreatment::Distinct) if op=="count"=>true,
            _=>return Err("aggregate modifiers"),
        };
        if !args.clauses.is_empty() || args.args.len() != 1 {
            return Err("aggregate modifiers");
        }
        let FunctionArg::Unnamed(arg) = &args.args[0] else {
            return Err("named aggregate argument");
        };
        let (sql, carrier, fields) = match arg {
            FunctionArgExpr::Wildcard if op == "count" && !distinct => {
                ("*".into(), Carrier::Integer, BTreeSet::new())
            }
            FunctionArgExpr::Expr(e) => {
                let c = self.column(e)?;
                if ["sum", "avg"].contains(&op)
                    && !matches!(c.carrier, Carrier::Integer | Carrier::Decimal)
                {
                    return Err("numeric aggregate input required");
                }
                if ["min", "max"].contains(&op) && matches!(c.carrier, Carrier::Boolean) {
                    return Err("ordered aggregate input required");
                }
                let carrier = if op == "count" {
                    Carrier::Integer
                } else if op == "avg" {
                    Carrier::Decimal
                } else {
                    c.carrier.clone()
                };
                (c.sql, carrier, c.fields)
            }
            _ => return Err("aggregate input"),
        };
        self.used_functions.borrow_mut().insert(format!("pg_catalog.{op}"));
        let rendered = if distinct {
            format!("pg_catalog.count(DISTINCT {sql})")
        } else if op == "sum" {
            format!("pg_catalog.sum({sql}::pg_catalog.numeric)")
        } else {
            format!("pg_catalog.{op}({sql})")
        };
        Ok(Scalar {
            sql: rendered,
            carrier,
            nullable: op != "count",
            aggregate: true,
            fields,
        })
    }
    fn predicate_value(&mut self, e: &Expr, aggregates: bool) -> Result<Scalar> {
        let c = self.value(e, aggregates)?;
        if aggregates
            && !c.aggregate
            && !c.fields.is_empty()
            && self.group_values.as_ref().is_some_and(|g| !g.contains(&c.sql))
        {
            return Err("ungrouped HAVING column");
        };
        Ok(c)
    }
    fn predicate(&mut self, e: &Expr, aggregates: bool) -> Result<String> {
        if let Expr::BinaryOp{op,..}=e {if !matches!(op,BinaryOperator::And|BinaryOperator::Or){self.used_operators.borrow_mut().insert(op.to_string());}}
        match e {
            Expr::Nested(e) => Ok(format!("({})", self.predicate(e, aggregates)?)),
            Expr::BinaryOp { left, op, right }
                if matches!(op, BinaryOperator::And | BinaryOperator::Or) =>
            {
                Ok(format!(
                    "({} {} {})",
                    self.predicate(left, aggregates)?,
                    op,
                    self.predicate(right, aggregates)?
                ))
            }
            Expr::BinaryOp { left, op, right }
                if matches!(
                    op,
                    BinaryOperator::Eq
                        | BinaryOperator::NotEq
                        | BinaryOperator::Gt
                        | BinaryOperator::GtEq
                        | BinaryOperator::Lt
                        | BinaryOperator::LtEq
                ) =>
            {
                let l = self.predicate_value(left, aggregates)?;
                let r = self.predicate_value(right, aggregates)?;
                if l.carrier != r.carrier {
                    return Err("comparison carrier mismatch");
                }
                Ok(format!("{} {} {}", l.sql, op, r.sql))
            }
            Expr::Function(f) => {
                let function=object(&f.name)?;
                let is_term=function==["matches_all_terms"] || function==["matches_any_term"];
                if (function != ["starts_with"] && !is_term)
                    || f.uses_odbc_syntax
                    || !matches!(f.parameters, FunctionArguments::None)
                    || !f.within_group.is_empty()
                    || f.filter.is_some()
                    || f.null_treatment.is_some()
                    || f.over.is_some()
                {
                    return Err("unsupported predicate function");
                }
                let FunctionArguments::List(args) = &f.args else {
                    return Err("prefix arguments");
                };
                if args.duplicate_treatment.is_some() || !args.clauses.is_empty() || args.args.len() != 2 {
                    return Err("prefix requires exactly two unmodified arguments");
                }
                let (FunctionArg::Unnamed(FunctionArgExpr::Expr(field)), FunctionArg::Unnamed(FunctionArgExpr::Expr(prefix))) = (&args.args[0], &args.args[1]) else {
                    return Err("prefix requires a field and scalar value");
                };
                fn unnest(e: &Expr) -> &Expr { if let Expr::Nested(inner) = e { unnest(inner) } else { e } }
                if !matches!(unnest(field), Expr::CompoundIdentifier(_)) {
                    return Err("prefix requires an explicit field");
                }
                if !matches!(unnest(prefix), Expr::Value(v) if matches!(v.value, Value::SingleQuotedString(_) | Value::Placeholder(_))) {
                    return Err("prefix requires a literal or declared parameter");
                }
                let source_field=field;
                let field = self.predicate_value(field, aggregates)?;
                let prefix = self.predicate_value(prefix, aggregates)?;
                if field.carrier != Carrier::String || prefix.carrier != Carrier::String {
                    return Err("prefix string carriers required");
                }
                if is_term {
                    let Expr::CompoundIdentifier(parts)=unnest(source_field) else {return Err("term source field required")};
                    let alias=name(&parts[0]);let logical=name(&parts[1]);
                    let table=self.scans.get(&alias).ok_or("term source alias absent")?.0;
                    let index=weft_core::search::admit_terms(self.admitted,&table.logical,&logical)?;
                    let language=index.index()["keyword"]["analyzer"]["language"].as_str().ok_or("explicit term analyzer required")?;
                    let configuration=match language {"english"=>"pg_catalog.english","simple"=>"pg_catalog.simple",_=>return Err("unsupported term analyzer")};
                    let column=table.columns.iter().find(|c|c.logical==logical).ok_or("term native column absent")?;
                    let identity=weft_core::mutation::canonical(&json!({"source":index.request().source,"table":table.logical,"field":logical}));
                    let metadata=term_index_metadata(&index,table,column,&self.request.binding.schema,configuration)?;
                    self.term_indexes.insert(identity,metadata);
                    self.used_functions.borrow_mut().extend(["pg_catalog.tsvector_to_array","pg_catalog.to_tsvector"].map(str::to_string));
                    self.used_operators.borrow_mut().insert(if function==["matches_all_terms"]{"<@"}else{"&&"}.into());
                    let field_terms=format!("pg_catalog.tsvector_to_array(pg_catalog.to_tsvector('{configuration}'::pg_catalog.regconfig, {}))",field.sql);
                    let query_terms=format!("pg_catalog.tsvector_to_array(pg_catalog.to_tsvector('{configuration}'::pg_catalog.regconfig, {}))",prefix.sql);
                    return Ok(if function==["matches_all_terms"] {format!("({query_terms} OPERATOR(pg_catalog.<@) {field_terms})")} else {format!("({query_terms} OPERATOR(pg_catalog.&&) {field_terms})")});
                }
                // PostgreSQL's strict prefix function compares the actual UTF-8
                // text bytes. No pattern is formed and NULL remains UNKNOWN.
                self.used_functions.borrow_mut().insert("pg_catalog.starts_with".into());
                Ok(format!("pg_catalog.starts_with({} COLLATE pg_catalog.\"C\", {} COLLATE pg_catalog.\"C\")", field.sql, prefix.sql))
            }
            Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => {
                fn is_null(expr: &Expr) -> bool {
                    match expr {
                        Expr::Value(value) => matches!(value.value, Value::Null),
                        Expr::Nested(inner) => is_null(inner),
                        _ => false,
                    }
                }
                let distinct = matches!(e, Expr::IsDistinctFrom(_, _));
                match (is_null(left), is_null(right)) {
                    (true, true) => return Ok(if distinct { "FALSE" } else { "TRUE" }.into()),
                    (true, false) | (false, true) => {
                        let scalar = self.predicate_value(
                            if is_null(left) { right } else { left }, aggregates,
                        )?;
                        return Ok(format!("{} IS {}NULL", scalar.sql, if distinct { "NOT " } else { "" }));
                    }
                    (false, false) => {}
                }
                let l = self.predicate_value(left, aggregates)?;
                let r = self.predicate_value(right, aggregates)?;
                if l.carrier != r.carrier {
                    return Err("comparison carrier mismatch");
                }
                let op = if distinct {
                    "IS DISTINCT FROM"
                } else {
                    "IS NOT DISTINCT FROM"
                };
                Ok(format!("{} {} {}", l.sql, op, r.sql))
            }
            Expr::Between{expr,negated,low,high}=>{let value=self.predicate_value(expr,aggregates)?;let low=self.predicate_value(low,aggregates)?;let high=self.predicate_value(high,aggregates)?;if value.carrier!=low.carrier||value.carrier!=high.carrier{return Err("BETWEEN carrier mismatch")};Ok(format!("{} {}BETWEEN {} AND {}",value.sql,if *negated{"NOT "}else{""},low.sql,high.sql))},
            Expr::InList{expr,list,negated}=>{let value=self.predicate_value(expr,aggregates)?;if list.len()>1000{return Err("IN list budget")};if list.is_empty(){return Ok(if *negated{"TRUE"}else{"FALSE"}.into())};let items=list.iter().map(|expr|{let item=self.predicate_value(expr,aggregates)?;if item.carrier!=value.carrier{return Err("IN carrier mismatch")};Ok(item.sql)}).collect::<Result<Vec<_>>>()?;Ok(format!("{} {}IN ({})",value.sql,if *negated{"NOT "}else{""},items.join(", ")))},
            Expr::IsNull(e) => Ok(format!(
                "{} IS NULL",
                self.predicate_value(e, aggregates)?.sql
            )),
            Expr::IsNotNull(e) => Ok(format!(
                "{} IS NOT NULL",
                self.predicate_value(e, aggregates)?.sql
            )),
            Expr::UnaryOp {
                op: UnaryOperator::Not,
                expr,
            } => Ok(format!("NOT ({})", self.predicate(expr, aggregates)?)),
            _ => Err("unsupported predicate"),
        }
    }
    fn join_pairs(&self, e: &Expr, new_alias: &str) -> Result<(String, BTreeSet<[String; 2]>)> {
        fn pairs(e: &Expr) -> Result<Vec<(&Expr, &Expr)>> {
            match e {
                Expr::Nested(e) => pairs(e),
                Expr::BinaryOp {
                    left,
                    op: BinaryOperator::And,
                    right,
                } => {
                    let mut x = pairs(left)?;
                    x.extend(pairs(right)?);
                    Ok(x)
                }
                Expr::BinaryOp {
                    left,
                    op: BinaryOperator::Eq,
                    right,
                } => Ok(vec![(left, right)]),
                _ => Err("join must match declared relationship equality"),
            }
        }
        let mut old = None;
        let mut result = BTreeSet::new();
        for (l, r) in pairs(e)? {
            let a = self.column(l)?;
            let b = self.column(r)?;
            if a.carrier != b.carrier {
                return Err("join carrier mismatch");
            }
            let (a, b) = (
                a.fields.iter().next().unwrap(),
                b.fields.iter().next().unwrap(),
            );
            let (source, target) = if b.0 == new_alias {
                (a, b)
            } else if a.0 == new_alias {
                (b, a)
            } else {
                return Err("join must reach new scan");
            };
            if source.0==new_alias{return Err("join must connect an existing source scan")}
            if old.as_ref().is_some_and(|x| x != &source.0) {
                return Err("join spans multiple sources");
            };
            old = Some(source.0.clone());
            if !result.insert([source.1.clone(), target.1.clone()]) {
                return Err("duplicate join component");
            }
        }
        Ok((old.ok_or("empty join")?, result))
    }
}
pub(crate) fn lower(admitted: &AdmittedRead) -> Result<Json> { lower_profile(admitted,true) }
pub(crate) fn lower_search_input(admitted:&AdmittedRead)->Result<Json>{lower_profile(admitted,false)}
fn lower_profile(admitted: &AdmittedRead, bounded:bool) -> Result<Json> {
    let r = admitted.request();
    let q = admitted.query();
    if r.snapshots.is_some() || r.version != "weft.raw-read/0.1"
        || !["postgresql18", "lakebase18"].contains(&r.target.as_str())
    {
        return Err("unsupported target or request version");
    }
    if r.binding.version != "umf.raw-read-binding/0.1"
        || r.binding.model_sha256.len() != 64
        || !r
            .binding
            .model_sha256
            .bytes()
            .all(|c| c.is_ascii_hexdigit())
        || !(1..=1000).contains(&r.row_cap)
    {
        return Err("binding version/model pin or row cap");
    }
    if let Some(schema) = &r.binding.schema {
        ident(schema)?;
    }
    match &r.binding.schema_binding {
        Some(weft_core::raw_read::SchemaBinding::Qualified) if r.binding.schema.is_none()=>return Err("qualified schemaBinding requires explicit schema"),
        Some(weft_core::raw_read::SchemaBinding::SearchPath) if r.binding.schema.is_some()=>return Err("search-path schemaBinding requires runtime app schema"),
        _=>()
    }
    if r.sql.len() > 65536 || r.parameters.len() > 64 || r.binding.tables.len() > 256 {
        return Err("input limit");
    }
    let mut names = BTreeSet::new();
    for p in &r.parameters {
        if p.name.starts_with("__umf_")
            || p.name.is_empty()
            || !p
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            || !names.insert(&p.name)
        {
            return Err("parameter identity");
        }
    }
    let mut logical = BTreeSet::new();
    let mut physical = BTreeSet::new();
    for t in &r.binding.tables {
        ident(&t.native)?;
        if t.logical.len() != 2
            || !logical.insert(&t.logical)
            || !physical.insert(&t.native)
            || t.key.is_empty()
        {
            return Err("ambiguous/invalid table binding");
        }
        let mut cols = BTreeSet::new();
        let mut native = BTreeSet::new();
        for c in &t.columns {
            ident(&c.native)?;
            if !cols.insert(&c.logical) || !native.insert(&c.native) {
                return Err("duplicate column binding");
            }
        }
        if t.key.iter().any(|k| {
            !cols.contains(k) || t.columns.iter().find(|c| &c.logical == k).unwrap().nullable
        }) {
            return Err("invalid unique key");
        }
    }
    if q.with.is_some()
        || q.fetch.is_some()
        || !q.locks.is_empty()
        || q.for_clause.is_some()
        || q.settings.is_some()
        || q.format_clause.is_some()
        || !q.pipe_operators.is_empty()
    {
        return Err("query modifier outside read-only profile");
    }
    let SetExpr::Select(s) = q.body.as_ref() else {
        return Err("set operations outside profile");
    };
    if !s.optimizer_hints.is_empty()
        || s.distinct.is_some()
        || s.select_modifiers.is_some()
        || s.top.is_some()
        || s.exclude.is_some()
        || s.into.is_some()
        || !s.lateral_views.is_empty()
        || s.prewhere.is_some()
        || !s.connect_by.is_empty()
        || !s.cluster_by.is_empty()
        || !s.distribute_by.is_empty()
        || !s.sort_by.is_empty()
        || !s.named_window.is_empty()
        || s.qualify.is_some()
        || s.value_table_mode.is_some()
    {
        return Err("SELECT modifier outside profile");
    }
    if s.from.len() != 1
        || s.from[0].joins.len() > 3
        || s.projection.is_empty()
        || s.projection.len() > 256
    {
        return Err("one root and at most three joins required");
    }
    let mut l = Lower {
        admitted,
        term_indexes: BTreeMap::new(),
        used_columns: Default::default(),
        used_functions: Default::default(),
        used_operators: Default::default(),
        request: r,
        scans: BTreeMap::new(),
        slots: vec![],
        used: BTreeSet::new(),
        group_values: None,
    };
    let (_, mut from) = l.table(&s.from[0].relation, false)?;
    let mut scan_parents:BTreeMap<String,String>=BTreeMap::new();
    for j in &s.from[0].joins {
        if j.global{return Err("GLOBAL join modifier outside PostgreSQL profile")}
        let (left, on) = match &j.join_operator {
            JoinOperator::Inner(JoinConstraint::On(on))
            | JoinOperator::Join(JoinConstraint::On(on)) => (false, on),
            JoinOperator::LeftOuter(JoinConstraint::On(on))
            | JoinOperator::Left(JoinConstraint::On(on)) => (true, on),
            _ => return Err("inner/left ON joins only"),
        };
        let (a, sql) = l.table(&j.relation, left)?;
        let (old, pairs) = l.join_pairs(on, &a)?;
        let source = l.scans[&old].0;
        let target = l.scans[&a].0;
        let rel = r
            .binding
            .relationships
            .iter()
            .find(|rel| {
                rel.to_one
                    && rel.source == source.logical
                    && rel.target == target.logical
                    && rel.pairs.iter().cloned().collect::<BTreeSet<_>>() == pairs
            })
            .ok_or("join is not a declared to-one relationship")?;
        if rel.id.is_empty()
            || rel.pairs.iter().map(|p| p[1].clone()).collect::<Vec<_>>() != target.key
        {
            return Err("relationship does not cover declared target unique key");
        }
        from += &format!(
            " {} JOIN {} ON {}",
            if left { "LEFT" } else { "INNER" },
            sql,
            l.predicate(on, false)?
        );
        // Equality to a required target Key rejects NULL source tuples. An
        // INNER edge proves this source and its exact ancestor chain present;
        // independent LEFT siblings and descendants remain nullable.
        if !left {
            let mut ancestor=Some(old.clone());
            while let Some(alias)=ancestor {
                l.scans.get_mut(&alias).ok_or("join ancestor scan absent")?.1=false;
                ancestor=scan_parents.get(&alias).cloned();
            }
        }
        scan_parents.insert(a,old);
    }
    let GroupByExpr::Expressions(group, modifiers) = &s.group_by else {
        return Err("grouping sets unsupported");
    };
    if !modifiers.is_empty() {
        return Err("group modifiers");
    }
    let mut groups = vec![];
    let mut group_values = BTreeSet::new();
    for e in group {
        let c = l.value(e,false)?;
        if c.fields.is_empty() || c.aggregate {return Err("group key must be a field or numeric bucket")};
        group_values.insert(c.sql.clone());
        groups.push(c.sql)
    }
    let mut native_columns=BTreeMap::new();
    if r.binding.native_columns.is_some(){
        for (alias,(table,_)) in &l.scans {
            for (logical,c) in native_projection::columns(admitted,table)? {
                native_columns.insert((alias.clone(),logical),c);
            }
        }
    }
    let mut outputs = vec![];
    let mut projection = vec![];
    let mut scalars = vec![];
    let mut output_names = BTreeSet::new();
    for item in &s.projection {
        let (expr, alias) = match item {
            SelectItem::ExprWithAlias { expr, alias } => (expr, name(alias)),
            SelectItem::UnnamedExpr(e) => {
                let Expr::CompoundIdentifier(p) = e else {
                    return Err("aggregate results require explicit aliases");
                };
                (e, name(p.last().ok_or("column")?))
            }
            _ => return Err("explicit projection required"),
        };
        if !output_names.insert(alias.clone()) {
            return Err("duplicate output name");
        }
        if let Expr::CompoundIdentifier(parts)=expr {if parts.len()==2 {
            if let Some(c)=native_columns.get(&(name(&parts[0]),name(&parts[1]))) {
                if matches!(c.metadata["carrier"].as_str(),Some("binary"|"vector")) {
                    l.used_columns.borrow_mut().insert((name(&parts[0]),name(&parts[1])));
                    projection.push(format!("{}.{} AS {}",ident(&name(&parts[0]))?,ident(&c.native)?,ident(&alias)?));
                    let mut metadata=c.metadata.as_object().unwrap().clone();let carrier=metadata.remove("carrier").unwrap();let nullable=metadata.remove("nullable").unwrap().as_bool().unwrap()||l.scans[&name(&parts[0])].1;
                    outputs.push(Output{name:alias,carrier,nullable,metadata});scalars.push(None);continue;
                }
            }
        }}
        let c = l.value(expr, true)?;
        projection.push(format!("{} AS {}", c.sql, ident(&alias)?));
        let mut metadata=serde_json::Map::new();
        if let Expr::CompoundIdentifier(parts)=expr {if parts.len()==2 {if let Some(n)=native_columns.get(&(name(&parts[0]),name(&parts[1]))) {metadata=n.metadata.as_object().unwrap().clone();metadata.remove("carrier");metadata.remove("nullable");}}}
        outputs.push(Output {
            name: alias,
            carrier: serde_json::to_value(&c.carrier).unwrap(),
            nullable: c.nullable,
            metadata,
        });
        scalars.push(Some(c));
    }
    let aggregate = scalars.iter().flatten().any(|c| c.aggregate) || !groups.is_empty();
    if aggregate
        && scalars
            .iter()
            .any(|c| c.as_ref().is_none_or(|c| !c.aggregate && !c.fields.is_empty() && !group_values.contains(&c.sql)))
    {
        return Err("ungrouped projected column");
    }
    let mut cursor_parameters: Vec<Parameter> = vec![];
    let mut cursor_outputs: Vec<Output> = vec![];
    let mut cursor_ordering: Vec<Json> = vec![];
    let mut after_filter = None;
    if r.keyset {
        if aggregate || s.having.is_some() {
            return Err("aggregate paging requires a separate profile");
        }
        let order = q.order_by.as_ref().ok_or("keyset requires ordering")?;
        let OrderByKind::Expressions(items) = &order.kind else {
            return Err("keyset ordering");
        };
        if items.is_empty() || items.len() > 16 {
            return Err("keyset ordering bounds");
        }
        let root_alias = match &s.from[0].relation {
            TableFactor::Table { alias: Some(a), .. } => name(&a.name),
            _ => return Err("root alias"),
        };
        let root = l.scans[&root_alias].0;
        let mut seen = BTreeSet::new();
        let mut equal = vec![];
        let mut after = vec![];
        l.slots.push("__umf_has_after".into());
        let has_after = format!("${}::pg_catalog.bool", l.slots.len());
        cursor_parameters.push(Parameter {
            name: "__umf_has_after".into(),
            carrier: Carrier::Boolean,
            nullable: false,
        });
        for (i, item) in items.iter().enumerate() {
            let c = l.column(&item.expr)?;
            let field = c.fields.iter().next().unwrap().clone();
            if !seen.insert(field.clone()) {
                return Err("duplicate keyset ordering column");
            }
            if matches!(c.carrier, Carrier::Boolean) {
                return Err("boolean keyset ordering outside profile");
            }
            let (table, _) = l.scans.get(&field.0).ok_or("missing ordering owner")?;
            cursor_ordering.push(json!({"table": table.logical, "field":field.1,
                "direction":match item.options.sort {None|Some(OrderBySort::Asc)=>"ASC",Some(OrderBySort::Desc)=>"DESC",_=>return Err("keyset operator")},
                "nulls":if item.options.nulls_first.unwrap_or(false){"FIRST"}else{"LAST"}}));
            let pname = format!("__umf_after_{i}");
            l.slots.push(pname.clone());
            let slot = format!("${}::pg_catalog.{}", l.slots.len(), c.carrier.sql());
            cursor_parameters.push(Parameter {
                name: pname,
                carrier: c.carrier.clone(),
                nullable: true,
            });
            let outname = format!("__umf_order_{i}");
            if output_names.contains(&outname) {
                return Err("reserved cursor output name");
            }
            cursor_outputs.push(Output {
                name: outname.clone(),
                carrier: serde_json::to_value(&c.carrier).unwrap(),
                nullable: c.nullable,
                metadata: Default::default(),
            });
            projection.push(format!("{} AS {}", c.sql, ident(&outname)?));
            let cmp = match item.options.sort {
                None | Some(OrderBySort::Asc) => ">",
                Some(OrderBySort::Desc) => "<",
                _ => return Err("keyset operator"),
            };
            let special = if item.options.nulls_first.unwrap_or(false) {
                format!("({slot} IS NULL AND {} IS NOT NULL)", c.sql)
            } else {
                format!("({slot} IS NOT NULL AND {} IS NULL)", c.sql)
            };
            let strict = format!(
                "(({slot} IS NOT NULL AND {} {cmp} {slot}) OR {special})",
                c.sql
            );
            let prefix = if equal.is_empty() {
                String::new()
            } else {
                format!("{} AND ", equal.join(" AND "))
            };
            after.push(format!("({prefix}{strict})"));
            equal.push(format!("{} IS NOT DISTINCT FROM {slot}", c.sql));
        }
        if root
            .key
            .iter()
            .any(|k| !seen.contains(&(root_alias.clone(), k.clone())))
        {
            return Err("keyset ordering must contain the complete root unique key");
        }
        after_filter = Some(format!("({has_after} = FALSE OR ({}))", after.join(" OR ")));
    }
    let mut sql = format!("SELECT {} FROM {}", projection.join(", "), from);
    if let Some(e) = &s.selection {
        sql += &format!(" WHERE {}", l.predicate(e, false)?)
    }
    if let Some(after) = after_filter {
        sql += &format!(
            " {} {}",
            if s.selection.is_some() {
                "AND"
            } else {
                "WHERE"
            },
            after
        );
    }
    if !groups.is_empty() {
        sql += &format!(" GROUP BY {}", groups.join(", "))
    }
    l.group_values = Some(group_values.clone());
    if let Some(e) = &s.having {
        if !aggregate {
            return Err("HAVING requires aggregates");
        };
        sql += &format!(" HAVING {}", l.predicate(e, true)?)
    }
    let mut rendered_order=vec![];
    let mut ordered_values=BTreeSet::new();
    if let Some(order) = &q.order_by {
        if order.interpolate.is_some() {
            return Err("ordering interpolation");
        }
        let OrderByKind::Expressions(items) = &order.kind else {
            return Err("ORDER BY ALL");
        };
        for item in items {
            if item.with_fill.is_some() {
                return Err("ordering fill");
            };
            let c=if let Expr::Identifier(id)=&item.expr {
                let position=outputs.iter().position(|o|o.name==name(id)).ok_or("ordering output alias absent")?;
                scalars[position].clone().ok_or("native projection cannot be ordered")?
            } else {l.value(&item.expr,true)?};
            if aggregate && !c.aggregate && !c.fields.is_empty() && !group_values.contains(&c.sql) {
                return Err("ungrouped ordering");
            }
            ordered_values.insert(c.sql.clone());
            rendered_order.push(format!(
                "{} {} NULLS {}",
                c.sql,
                match item.options.sort {
                    None | Some(OrderBySort::Asc) => "ASC",
                    Some(OrderBySort::Desc) => "DESC",
                    _ => return Err("custom ordering operator"),
                },
                if item.options.nulls_first.unwrap_or(false) {
                    "FIRST"
                } else {
                    "LAST"
                }
            ));
        }
    }
    if aggregate {
        for group in &groups {if ordered_values.insert(group.clone()) {rendered_order.push(format!("{group} ASC NULLS LAST"))}}
    }
    if !rendered_order.is_empty() {sql += &format!(" ORDER BY {}",rendered_order.join(", "))}
    let limit = match &q.limit_clause {
        None => r.row_cap,
        Some(LimitClause::LimitOffset {
            limit: Some(Expr::Value(v)),
            offset: None,
            limit_by,
        }) if limit_by.is_empty() => match &v.value {
            Value::Number(n, false) => n
                .parse::<u16>()
                .ok()
                .filter(|n| *n > 0 && *n <= r.row_cap)
                .ok_or("limit exceeds row cap")?,
            _ => return Err("literal bounded limit required"),
        },
        _ => return Err("offset/limit outside profile"),
    };
    if r.stream_page_size {
        if !bounded||!r.keyset{return Err("stream page size requires bounded keyset profile")}
        let reserved="__umf_page_size".to_string();
        if l.slots.contains(&reserved)||cursor_parameters.iter().any(|p|p.name==reserved){return Err("reserved stream page size collision")}
        l.slots.push(reserved.clone());cursor_parameters.push(Parameter{name:reserved,carrier:Carrier::Integer,nullable:false});
        sql+=&format!(" LIMIT ${}::pg_catalog.int8",l.slots.len());
    } else if bounded {sql += &format!(" LIMIT {}", limit);}
    if l.used.len() != r.parameters.len() {
        return Err("unused declared parameter");
    }
    let mut obligations = json!({"transaction":"read-only","principal":"caller-connection-role","schema":r.binding.schema,"searchPath":if r.binding.schema.is_none(){json!(["pg_catalog","$appSchema"])}else{json!(["pg_catalog"])},"timeout":"per-call","rowCap":if bounded{json!(limit)}else{Json::Null},"rls":"native-caller-role","noBypassRls":true,"baseTablesOnly":true});
    if !native_columns.is_empty(){
        let mut admissions=vec![];let mut seen=BTreeSet::new();
        for ((alias,_),column) in &native_columns {let table=l.scans[alias].0;
            if seen.insert((table.native.clone(),column.native.clone())) {admissions.push(json!({"nativeAdmission":native_projection::admission(column,table,&r.binding.schema)}));}
        }
        obligations["nativeProjection"]=json!({"version":"umf.native-columns/0.1","nativeCatalogBeforeAfter":true,"columns":admissions});
    }
    if r.stream_page_size {obligations["streamPageSize"]=json!({"parameter":"__umf_page_size","minimum":1,"maximum":limit});}
    if let Some(mode)=&r.binding.schema_binding { obligations["schemaBinding"]=serde_json::to_value(mode).map_err(|_|"schema binding serialization")?; }
    if !l.term_indexes.is_empty() {
        obligations["termFilter"]=json!({"version":"umf.raw-term-filter/0.1","nativeCatalogBeforeAfter":true,"indexes":l.term_indexes.values().collect::<Vec<_>>()});
    }
    let lock_sql: Vec<String> = l
        .scans
        .values()
        .map(|(t, _)| {
            let table = match &r.binding.schema {
                Some(schema) => format!("{}.{}", ident(schema).unwrap(), ident(&t.native).unwrap()),
                None => ident(&t.native).unwrap(),
            };
            format!("LOCK TABLE {} IN ACCESS SHARE MODE", table)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let referenced_tables: Vec<Json> = l.scans.iter().map(|(alias,(table,_))| {
        let mut columns=table.columns.iter().map(|c|serde_json::to_value(c).unwrap()).collect::<Vec<_>>();
        for ((owner,logical),c) in &native_columns {if owner==alias&&!table.columns.iter().any(|x|x.logical==*logical) {let mut item=c.metadata.clone();item["logical"]=json!(logical);item["native"]=json!(c.native);columns.push(item);}}
        json!({"logical":table.logical,"native":table.native,"columns":columns})
    }).collect();
    let mut selected_columns:BTreeMap<String,BTreeSet<String>>=BTreeMap::new();
    for(alias,logical)in l.used_columns.borrow().iter(){let table=l.scans[alias].0;let native=table.columns.iter().find(|c|c.logical==*logical).map(|c|c.native.clone()).or_else(||native_columns.get(&(alias.clone(),logical.clone())).map(|c|c.native.clone())).ok_or("used native column missing original mapping")?;selected_columns.entry(table.native.clone()).or_default().insert(native);}
    for(table,_)in l.scans.values(){selected_columns.entry(table.native.clone()).or_default();}
    let privilege_inventory=json!({"version":"weft.native-privileges/0.1","complete":false,"unresolved":["native-function-and-operator-overload-closure"],"functions":l.used_functions.borrow().iter().collect::<Vec<_>>(),"operators":l.used_operators.borrow().iter().collect::<Vec<_>>(),"tables":selected_columns.iter().map(|(table,columns)|json!({"schema":r.binding.schema,"native":table,"privilege":"SELECT","columns":columns,"anyColumn":columns.is_empty()})).collect::<Vec<_>>(),"indexObjects":l.term_indexes.values().map(|i|json!({"native":i["nativeBinding"]["native"],"usage":"native-term-admission"})).collect::<Vec<_>>()});
    let mut result = json!({"version":if bounded{"weft.raw-read/0.1"}else{"weft.search-input/0.1"},"status":"compiled","target":r.target,"sql":sql,"parameters":r.parameters,"parameterOrder":l.slots,"outputs":outputs,"obligations":obligations,"binding":r.binding,"sourceSql":r.sql,"lockSql":lock_sql,"cursorParameters":cursor_parameters,"cursorOutputs":cursor_outputs,"cursorOrdering":cursor_ordering,"referencedTables":referenced_tables});
    result["privilegeInventory"]=privilege_inventory;
    let digest = weft_core::json::sha256(result.to_string().as_bytes());
    result["structuralDigest"] = json!(digest);
    Ok(result)
}
pub fn compile_json(raw: &str) -> String {
    let result = weft_core::raw_read::admit_json(raw).and_then(|admitted| lower(&admitted));
    result.unwrap_or_else(|reason|json!({"version":"weft.raw-read/0.1","status":"blocked","diagnostics":[{"code":"WFT-RAW-READ","message":reason}]})).to_string()
}

include!("raw_read_tests.rs");
