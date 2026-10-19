// SPDX-License-Identifier: MIT OR Apache-2.0
//! Parameter transport for compiler-emitted PostgreSQL; no value interpolation.
use serde_json::{json, Value};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};
type Result<T> = std::result::Result<T, &'static str>;

fn render(sql: &str, slots: &[Value]) -> Result<(String, Vec<Value>)> {
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location().map_err(|_| "DBAPI emitted SQL tokenization failed")?;
    let mut characters = sql.char_indices().peekable();
    let (mut line, mut column) = (1u64, 1u64);
    let (mut output, mut order, mut copied) = (String::new(), Vec::new(), 0);
    for token in tokens {
        let Token::Placeholder(placeholder) = token.token else { continue };
        let digits = placeholder.strip_prefix('$').ok_or("DBAPI requires emitted numeric placeholders")?;
        if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err("DBAPI placeholder is not a canonical positive slot");
        }
        let index = digits.parse::<usize>().map_err(|_| "DBAPI placeholder slot overflow")?;
        let name = slots.get(index - 1).filter(|v| v.as_str().is_some())
            .ok_or("DBAPI placeholder has no original parameter slot")?;
        // Token locations count Unicode scalars, not UTF-8 bytes. Walk the
        // original string once; never reconstruct SQL from token Display text.
        let location = (token.span.start.line, token.span.start.column);
        while (line, column) < location {
            let (_, ch) = characters.next().ok_or("DBAPI placeholder location exceeds SQL")?;
            if ch == '\n' { line += 1; column = 1; } else { column += 1; }
        }
        let start = characters.peek().map(|(i,_)| *i).ok_or("DBAPI placeholder location absent")?;
        let end = start.checked_add(placeholder.len()).ok_or("DBAPI placeholder span overflow")?;
        if (line, column) != location || start < copied || sql.get(start..end) != Some(placeholder.as_str()) {
            return Err("DBAPI placeholder span differs from original emitted bytes");
        }
        output.push_str(&sql[copied..start].replace('%', "%%"));
        output.push_str("%s");
        order.push(name.clone());
        copied = end;
    }
    output.push_str(&sql[copied..].replace('%', "%%"));
    Ok((output, order))
}

pub(crate) fn attach(program: &mut Value) -> Result<()> {
    let sql = program["sql"].as_str().ok_or("DBAPI emitted SQL absent")?;
    let slots = program["parameterOrder"].as_array().ok_or("DBAPI original parameter order absent")?;
    let (sql, order) = render(sql, slots)?;
    program["dbapiSql"] = json!(sql);
    program["dbapiParameterOrder"] = json!(order);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_quoted_bytes_and_maps_occurrences_in_emitted_order() {
        let sql = "SELECT E'\\\\é $1 50% it''s', \"$2é%\", $$dollar $1 %$$, $q$ $2 % $q$, $2, $1, $2 -- $1 %\r\n /* $2 % /* nested $1 */ */ + $1";
        let (actual, order) = render(sql, &[json!("first"), json!("second")]).unwrap();
        assert_eq!(actual, "SELECT E'\\\\é $1 50%% it''s', \"$2é%%\", $$dollar $1 %%$$, $q$ $2 %% $q$, %s, %s, %s -- $1 %%\r\n /* $2 %% /* nested $1 */ */ + %s");
        assert_eq!(order, vec![json!("second"),json!("first"),json!("second"),json!("first")]);
    }
    #[test]
    fn percent_only_queries_still_require_empty_parameter_tuple() {
        assert_eq!(render("SELECT '100% $1', 5 % 2", &[]).unwrap(), ("SELECT '100%% $1', 5 %% 2".to_owned(), vec![]));
    }
    #[test]
    fn invalid_slots_and_broken_quoted_input_refuse() {
        for sql in ["SELECT $0", "SELECT $01", "SELECT $2", "SELECT $99999999999999999999999999", "SELECT 'unterminated"] {
            assert!(render(sql, &[json!("one")]).is_err(), "{sql}");
        }
        assert!(render("SELECT $1", &[json!(null)]).is_err());
    }
    #[test]
    fn unicode_multiline_and_crlf_locations_preserve_every_nonparameter_byte() {
        let sql = "SELECT '🙂',\t$1\r\n, 'é\n雪', $2\r, $1";
        let (actual, order) = render(sql, &[json!("a"),json!("b")]).unwrap();
        assert_eq!(actual, "SELECT '🙂',\t%s\r\n, 'é\n雪', %s\r, %s");
        assert_eq!(order, vec![json!("a"),json!("b"),json!("a")]);
    }
}
