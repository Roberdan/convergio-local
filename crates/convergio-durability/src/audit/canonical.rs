//! Canonical JSON: stable byte-string for any `serde::Serialize` value.
//!
//! Two equal JSON values produce equal byte strings, regardless of how
//! they were constructed (HashMap iteration order, whitespace, key
//! reordering by clients). This is a hard requirement of the audit
//! chain — see [ADR-0002].
//!
//! Rules:
//! - object keys sorted lexicographically (UTF-8 byte order)
//! - no whitespace
//! - numbers in `serde_json`'s shortest representation
//! - strings escaped via `serde_json::to_string` (RFC 8259)
//!
//! [ADR-0002]: ../../../../docs/adr/0002-audit-hash-chain.md

use crate::error::Result;
use serde::Serialize;

/// Serialize `value` to canonical JSON.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String> {
    let v: serde_json::Value = serde_json::to_value(value)?;
    let mut out = String::new();
    write_canonical(&v, &mut out);
    Ok(out)
}

fn write_canonical(v: &serde_json::Value, out: &mut String) {
    match v {
        serde_json::Value::Null => out.push_str("null"),
        serde_json::Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        serde_json::Value::Number(n) => out.push_str(&n.to_string()),
        serde_json::Value::String(s) => out.push_str(&serde_json::to_string(s).unwrap_or_default()),
        serde_json::Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(*k).unwrap_or_default());
                out.push(':');
                if let Some(val) = map.get(*k) {
                    write_canonical(val, out);
                }
            }
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_keys() {
        let s1 = canonical_json(&json!({"b": 2, "a": 1})).unwrap();
        let s2 = canonical_json(&json!({"a": 1, "b": 2})).unwrap();
        assert_eq!(s1, s2);
        assert_eq!(s1, r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn handles_nested() {
        let s = canonical_json(&json!({
            "x": {"z": 1, "a": [3,2,1]},
            "a": null
        }))
        .unwrap();
        assert_eq!(s, r#"{"a":null,"x":{"a":[3,2,1],"z":1}}"#);
    }

    #[test]
    fn escapes_strings_predictably() {
        let s = canonical_json(&json!({"k": "a\"b\nc"})).unwrap();
        assert_eq!(s, r#"{"k":"a\"b\nc"}"#);
    }
}
