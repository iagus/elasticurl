//! Rendering helpers: padded tables, key-value listings and search hits.

use serde_json::{Map, Value};

/// Print a text body (e.g. `_cat` output) with exactly one trailing newline.
pub fn print_text(body: &str) {
    print!("{}", body.trim_end_matches('\n'));
    println!();
}

/// Print a JSON value, optionally pretty-printed.
pub fn print_json(value: &Value, pretty: bool) {
    let rendered = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .expect("serializing JSON");
    println!("{rendered}");
}

/// Render padded columns in the style of Elasticsearch `_cat` output.
pub fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let cols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(cols) {
            widths[i] = widths[i].max(cell.len());
        }
    }
    let header: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    let mut out = format_row(&header, &widths);
    for row in rows {
        out.push_str(&format_row(row, &widths));
    }
    out
}

fn format_row(cells: &[String], widths: &[usize]) -> String {
    let mut line = String::new();
    for (i, width) in widths.iter().enumerate() {
        let cell = cells.get(i).map(String::as_str).unwrap_or("");
        if i > 0 {
            line.push_str("  ");
        }
        if i + 1 < widths.len() {
            line.push_str(&format!("{cell:<width$}"));
        } else {
            line.push_str(cell); // no padding on the last column
        }
    }
    let mut out = line.trim_end().to_string();
    out.push('\n');
    out
}

/// Render a top-level JSON object as `key: value` lines (e.g. cluster health).
pub fn render_kv(value: &Value) -> String {
    match value.as_object() {
        Some(obj) => {
            let mut out = String::new();
            for (key, val) in obj {
                out.push_str(&format!("{key}: {}\n", scalar(val)));
            }
            out
        }
        None => format!("{}\n", scalar(value)),
    }
}

fn scalar(v: &Value) -> String {
    match v {
        Value::Null => "-".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => serde_json::to_string(other).expect("serializing JSON"),
    }
}

/// Flatten `GET /<index>/_mapping` into (field, type) rows, expanding nested
/// `properties` and multi-fields (`fields`) into dotted paths.
pub fn flatten_mappings(value: &Value) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    if let Some(indices) = value.as_object() {
        for mappings in indices.values() {
            if let Some(props) = mappings
                .pointer("/mappings/properties")
                .and_then(Value::as_object)
            {
                walk_props(props, "", &mut rows);
            }
        }
    }
    rows
}

fn walk_props(props: &Map<String, Value>, prefix: &str, rows: &mut Vec<Vec<String>>) {
    for (name, def) in props {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        let ty = def.get("type").and_then(Value::as_str).unwrap_or("-");
        rows.push(vec![path.clone(), ty.to_string()]);
        if let Some(inner) = def.get("properties").and_then(Value::as_object) {
            walk_props(inner, &path, rows);
        }
        if let Some(fields) = def.get("fields").and_then(Value::as_object) {
            walk_props(fields, &path, rows);
        }
    }
}

/// Render `_search` results as `total: N` followed by a table of hits.
pub fn render_hits(value: &Value) -> String {
    let mut out = String::new();
    // ES >= 7: {"total": {"value": N}}, ES <= 6: {"total": N}
    let total = value
        .pointer("/hits/total/value")
        .or_else(|| value.pointer("/hits/total"))
        .and_then(Value::as_i64);
    if let Some(total) = total {
        out.push_str(&format!("total: {total}\n"));
    }
    match value.pointer("/hits/hits").and_then(Value::as_array) {
        Some(hits) if !hits.is_empty() => {
            let rows: Vec<Vec<String>> = hits
                .iter()
                .map(|hit| {
                    ["_index", "_id", "_score", "_source"]
                        .iter()
                        .map(|k| hit.get(*k).map(|v| scalar(v)).unwrap_or_default())
                        .collect()
                })
                .collect();
            out.push_str(&render_table(&["_index", "_id", "_score", "_source"], &rows));
        }
        _ => out.push_str("no hits\n"),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn table_pads_columns() {
        let out = render_table(
            &["name", "type"],
            &[
                vec!["msg".into(), "text".into()],
                vec!["user.name".into(), "keyword".into()],
            ],
        );
        assert_eq!(out, "name       type\nmsg        text\nuser.name  keyword\n");
    }

    #[test]
    fn mappings_are_flattened() {
        let v = json!({"logs": {"mappings": {"properties": {
            "@timestamp": {"type": "date"},
            "msg": {"type": "text", "fields": {"keyword": {"type": "keyword"}}},
            "user": {"properties": {"name": {"type": "keyword"}}}
        }}}});
        let rows = flatten_mappings(&v);
        assert!(rows.contains(&vec!["msg".to_string(), "text".to_string()]));
        assert!(rows.contains(&vec!["msg.keyword".to_string(), "keyword".to_string()]));
        assert!(rows.contains(&vec!["user".to_string(), "-".to_string()]));
        assert!(rows.contains(&vec!["user.name".to_string(), "keyword".to_string()]));
    }

    #[test]
    fn hits_render_with_total() {
        let v = json!({"hits": {"total": {"value": 1}, "hits": [
            {"_index": "logs", "_id": "1", "_score": 1.0, "_source": {"msg": "hi"}}
        ]}});
        let out = render_hits(&v);
        assert!(out.contains("total: 1"));
        assert!(out.contains("_index"));
        assert!(out.contains(r#"{"msg":"hi"}"#));
    }

    #[test]
    fn empty_hits() {
        let v = json!({"hits": {"total": {"value": 0}, "hits": []}});
        assert_eq!(render_hits(&v), "total: 0\nno hits\n");
    }
}
