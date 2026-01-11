//! Canonical JSON serialization

use crate::error::CanonicalError;
use serde::Serialize;
use serde_json::{Map, Value};
use std::fmt::Write as FmtWrite;

/// Serialize a value to canonical JSON bytes
///
/// # Rules
///
/// - Object keys sorted lexicographically
/// - Arrays preserve order
/// - No whitespace
/// - Floats are rejected (use strings)
///
/// # Errors
///
/// Returns `CanonicalError::FloatNotAllowed` if any float is detected.
///
/// # Example
///
/// ```rust
/// use esrp_canonical::to_canonical_json;
///
/// let value = serde_json::json!({"z": 1, "a": 2});
/// let canonical = to_canonical_json(&value).unwrap();
/// assert_eq!(canonical, b"{\"a\":2,\"z\":1}");
/// ```
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    let json_value = serde_json::to_value(value)?;
    canonical_json_value(&json_value)
}

/// Serialize a serde_json::Value to canonical JSON bytes
pub fn to_canonical_json_value(value: &Value) -> Result<Vec<u8>, CanonicalError> {
    canonical_json_value(value)
}

/// Serialize a value to canonical JSON string
pub fn to_canonical_json_string<T: Serialize>(value: &T) -> Result<String, CanonicalError> {
    let bytes = to_canonical_json(value)?;
    // Safe because we only produce valid UTF-8
    Ok(String::from_utf8(bytes).expect("canonical JSON is always valid UTF-8"))
}

/// Internal implementation for canonical JSON serialization
fn canonical_json_value(value: &Value) -> Result<Vec<u8>, CanonicalError> {
    let mut output = Vec::new();
    write_canonical_value(&mut output, value)?;
    Ok(output)
}

/// Write a JSON value in canonical form
fn write_canonical_value(output: &mut Vec<u8>, value: &Value) -> Result<(), CanonicalError> {
    match value {
        Value::Null => {
            output.extend_from_slice(b"null");
        }
        Value::Bool(b) => {
            if *b {
                output.extend_from_slice(b"true");
            } else {
                output.extend_from_slice(b"false");
            }
        }
        Value::Number(n) => {
            // CRITICAL: Reject floats
            if n.is_f64() && !n.is_i64() && !n.is_u64() {
                return Err(CanonicalError::FloatNotAllowed);
            }
            // For integers, use the standard representation
            output.extend_from_slice(n.to_string().as_bytes());
        }
        Value::String(s) => {
            write_canonical_string(output, s);
        }
        Value::Array(arr) => {
            output.push(b'[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    output.push(b',');
                }
                write_canonical_value(output, item)?;
            }
            output.push(b']');
        }
        Value::Object(obj) => {
            write_canonical_object(output, obj)?;
        }
    }
    Ok(())
}

/// Write a JSON object with sorted keys
fn write_canonical_object(
    output: &mut Vec<u8>,
    obj: &Map<String, Value>,
) -> Result<(), CanonicalError> {
    output.push(b'{');

    // Sort keys lexicographically by UTF-8 bytes
    let mut keys: Vec<&String> = obj.keys().collect();
    keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            output.push(b',');
        }

        // Write key
        write_canonical_string(output, key);
        output.push(b':');

        // Write value
        if let Some(value) = obj.get(*key) {
            write_canonical_value(output, value)?;
        }
    }

    output.push(b'}');
    Ok(())
}

/// Write a JSON string with proper escaping
fn write_canonical_string(output: &mut Vec<u8>, s: &str) {
    output.push(b'"');

    for c in s.chars() {
        match c {
            '"' => output.extend_from_slice(b"\\\""),
            '\\' => output.extend_from_slice(b"\\\\"),
            '\n' => output.extend_from_slice(b"\\n"),
            '\r' => output.extend_from_slice(b"\\r"),
            '\t' => output.extend_from_slice(b"\\t"),
            c if c.is_control() => {
                // Escape control characters as \uXXXX
                let mut hex_buf = String::new();
                write!(hex_buf, "\\u{:04x}", c as u32).unwrap();
                output.extend_from_slice(hex_buf.as_bytes());
            }
            c => {
                // Write UTF-8 bytes directly
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                output.extend_from_slice(encoded.as_bytes());
            }
        }
    }

    output.push(b'"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sorted_keys() {
        let value = json!({"z": 1, "a": 2, "m": 3});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_nested_objects_sorted() {
        let value = json!({
            "b": {"y": 1, "x": 2},
            "a": {"z": 3, "w": 4}
        });
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, r#"{"a":{"w":4,"z":3},"b":{"x":2,"y":1}}"#);
    }

    #[test]
    fn test_arrays_preserve_order() {
        let value = json!([3, 1, 2]);
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, "[3,1,2]");
    }

    #[test]
    fn test_no_whitespace() {
        let value = json!({"a": [1, 2], "b": {"c": 3}});
        let canonical = to_canonical_json_string(&value).unwrap();

        // No spaces, newlines, or tabs
        assert!(!canonical.contains(' '));
        assert!(!canonical.contains('\n'));
        assert!(!canonical.contains('\t'));
    }

    #[test]
    fn test_float_rejected() {
        let value = json!({"temperature": 0.7});
        let result = to_canonical_json(&value);
        assert!(matches!(result, Err(CanonicalError::FloatNotAllowed)));
    }

    #[test]
    fn test_integer_accepted() {
        let value = json!({"count": 42});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, r#"{"count":42}"#);
    }

    #[test]
    fn test_string_with_float_value_accepted() {
        // Floats as strings are OK
        let value = json!({"temperature": "0.7"});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, r#"{"temperature":"0.7"}"#);
    }

    #[test]
    fn test_string_escaping() {
        let value = json!({"text": "line1\nline2\ttab\"quote\\backslash"});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert!(canonical.contains("\\n"));
        assert!(canonical.contains("\\t"));
        assert!(canonical.contains("\\\""));
        assert!(canonical.contains("\\\\"));
    }

    #[test]
    fn test_null_value() {
        let value = json!({"empty": null});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, r#"{"empty":null}"#);
    }

    #[test]
    fn test_boolean_values() {
        let value = json!({"yes": true, "no": false});
        let canonical = to_canonical_json_string(&value).unwrap();
        // Keys sorted, booleans lowercase
        assert_eq!(canonical, r#"{"no":false,"yes":true}"#);
    }

    #[test]
    fn test_empty_object() {
        let value = json!({});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, "{}");
    }

    #[test]
    fn test_empty_array() {
        let value = json!([]);
        let canonical = to_canonical_json_string(&value).unwrap();
        assert_eq!(canonical, "[]");
    }

    #[test]
    fn test_unicode() {
        let value = json!({"greeting": "Hello 世界 🌍"});
        let canonical = to_canonical_json_string(&value).unwrap();
        // Unicode should be preserved as UTF-8
        assert!(canonical.contains("世界"));
        assert!(canonical.contains("🌍"));
    }

    #[test]
    fn test_determinism() {
        let value = json!({"c": 3, "a": 1, "b": 2});

        // Multiple calls should produce identical output
        let c1 = to_canonical_json(&value).unwrap();
        let c2 = to_canonical_json(&value).unwrap();
        let c3 = to_canonical_json(&value).unwrap();

        assert_eq!(c1, c2);
        assert_eq!(c2, c3);
    }

    #[test]
    fn test_negative_integers() {
        let value = json!({"negative": -42, "zero": 0, "positive": 42});
        let canonical = to_canonical_json_string(&value).unwrap();
        assert!(canonical.contains("-42"));
    }

    #[test]
    fn test_large_integers() {
        let value = json!({"large": 9007199254740991_i64});
        let result = to_canonical_json_string(&value);
        assert!(result.is_ok());
    }
}
