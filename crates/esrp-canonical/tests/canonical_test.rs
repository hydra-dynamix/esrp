//! Comprehensive tests for canonical JSON serialization

use esrp_canonical::{to_canonical_json, to_canonical_json_string, CanonicalError};
use serde_json::json;

mod key_sorting {
    use super::*;

    #[test]
    fn test_simple_key_sorting() {
        let value = json!({"c": 3, "a": 1, "b": 2});
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, r#"{"a":1,"b":2,"c":3}"#);
    }

    #[test]
    fn test_nested_object_sorting() {
        let value = json!({
            "outer": {"z": 1, "a": 2},
            "inner": {"y": 3, "b": 4}
        });
        let result = to_canonical_json_string(&value).unwrap();
        // Both outer keys and inner keys should be sorted
        assert!(result.contains(r#""inner":{"b":4,"y":3}"#));
        assert!(result.contains(r#""outer":{"a":2,"z":1}"#));
    }

    #[test]
    fn test_deeply_nested_sorting() {
        let value = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "z": 1, "a": 2
                    },
                    "b": 3, "c": 4
                },
                "y": 5, "x": 6
            },
            "m": 7, "n": 8
        });
        let result = to_canonical_json_string(&value).unwrap();

        // Verify all levels are sorted
        assert!(result.find("\"a\":").unwrap() < result.find("\"z\":").unwrap());
        assert!(result.find("\"b\":").unwrap() < result.find("\"c\":").unwrap());
        assert!(result.find("\"x\":").unwrap() < result.find("\"y\":").unwrap());
        assert!(result.find("\"m\":").unwrap() < result.find("\"n\":").unwrap());
    }

    #[test]
    fn test_unicode_key_sorting() {
        // UTF-8 byte order comparison
        let value = json!({"é": 1, "a": 2, "z": 3});
        let result = to_canonical_json_string(&value).unwrap();

        // 'a' (0x61) < 'z' (0x7A) < 'é' (0xC3 0xA9 in UTF-8)
        let a_pos = result.find("\"a\":").unwrap();
        let z_pos = result.find("\"z\":").unwrap();
        let e_pos = result.find("\"é\":").unwrap();

        assert!(a_pos < z_pos);
        assert!(z_pos < e_pos);
    }

    #[test]
    fn test_numeric_string_key_sorting() {
        // String "10" < "2" lexicographically
        let value = json!({"10": 1, "2": 2, "1": 3});
        let result = to_canonical_json_string(&value).unwrap();

        // Lexicographic: "1" < "10" < "2"
        let pos1 = result.find("\"1\":").unwrap();
        let pos10 = result.find("\"10\":").unwrap();
        let pos2 = result.find("\"2\":").unwrap();

        assert!(pos1 < pos10);
        assert!(pos10 < pos2);
    }
}

mod float_rejection {
    use super::*;

    #[test]
    fn test_simple_float_rejected() {
        let value = json!({"temp": 0.7});
        let result = to_canonical_json(&value);
        assert!(matches!(result, Err(CanonicalError::FloatNotAllowed)));
    }

    #[test]
    fn test_nested_float_rejected() {
        let value = json!({"outer": {"inner": 0.5}});
        let result = to_canonical_json(&value);
        assert!(matches!(result, Err(CanonicalError::FloatNotAllowed)));
    }

    #[test]
    fn test_array_float_rejected() {
        let value = json!([1, 2.5, 3]);
        let result = to_canonical_json(&value);
        assert!(matches!(result, Err(CanonicalError::FloatNotAllowed)));
    }

    #[test]
    fn test_integer_accepted() {
        let value = json!({"count": 42});
        let result = to_canonical_json(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_negative_integer_accepted() {
        let value = json!({"offset": -10});
        let result = to_canonical_json(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_accepted() {
        let value = json!({"count": 0});
        let result = to_canonical_json(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_integer_accepted() {
        let value = json!({"big": 9007199254740991_i64});
        let result = to_canonical_json(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_as_string_accepted() {
        let value = json!({"temp": "0.7"});
        let result = to_canonical_json(&value);
        assert!(result.is_ok());
    }
}

mod array_order {
    use super::*;

    #[test]
    fn test_array_order_preserved() {
        let value = json!([3, 1, 4, 1, 5, 9, 2, 6]);
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, "[3,1,4,1,5,9,2,6]");
    }

    #[test]
    fn test_object_array_order_preserved() {
        let value = json!([
            {"id": 3},
            {"id": 1},
            {"id": 2}
        ]);
        let result = to_canonical_json_string(&value).unwrap();
        // Order should be 3, 1, 2
        let pos3 = result.find("\"id\":3").unwrap();
        let pos1 = result.find("\"id\":1").unwrap();
        let pos2 = result.find("\"id\":2").unwrap();
        assert!(pos3 < pos1);
        assert!(pos1 < pos2);
    }

    #[test]
    fn test_nested_array_order_preserved() {
        let value = json!([[3, 2, 1], [6, 5, 4]]);
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, "[[3,2,1],[6,5,4]]");
    }
}

mod whitespace {
    use super::*;

    #[test]
    fn test_no_whitespace() {
        let value = json!({
            "key": "value",
            "array": [1, 2, 3],
            "nested": {"a": 1}
        });
        let result = to_canonical_json_string(&value).unwrap();

        assert!(!result.contains(' '));
        assert!(!result.contains('\n'));
        assert!(!result.contains('\t'));
        assert!(!result.contains('\r'));
    }
}

mod string_escaping {
    use super::*;

    #[test]
    fn test_quote_escaping() {
        let value = json!({"text": "say \"hello\""});
        let result = to_canonical_json_string(&value).unwrap();
        assert!(result.contains(r#"\"hello\""#));
    }

    #[test]
    fn test_backslash_escaping() {
        let value = json!({"path": "C:\\Users\\test"});
        let result = to_canonical_json_string(&value).unwrap();
        assert!(result.contains(r"C:\\Users\\test"));
    }

    #[test]
    fn test_newline_escaping() {
        let value = json!({"text": "line1\nline2"});
        let result = to_canonical_json_string(&value).unwrap();
        assert!(result.contains(r"\n"));
        assert!(!result.contains('\n')); // Actual newline should not appear
    }

    #[test]
    fn test_tab_escaping() {
        let value = json!({"text": "col1\tcol2"});
        let result = to_canonical_json_string(&value).unwrap();
        assert!(result.contains(r"\t"));
    }

    #[test]
    fn test_carriage_return_escaping() {
        let value = json!({"text": "line1\rline2"});
        let result = to_canonical_json_string(&value).unwrap();
        assert!(result.contains(r"\r"));
    }

    #[test]
    fn test_control_character_escaping() {
        // ASCII control character (bell)
        let value = json!({"text": "\x07"});
        let result = to_canonical_json_string(&value).unwrap();
        assert!(result.contains(r"\u0007"));
    }
}

mod special_values {
    use super::*;

    #[test]
    fn test_null_value() {
        let value = json!({"empty": null});
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, r#"{"empty":null}"#);
    }

    #[test]
    fn test_true_value() {
        let value = json!({"flag": true});
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, r#"{"flag":true}"#);
    }

    #[test]
    fn test_false_value() {
        let value = json!({"flag": false});
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, r#"{"flag":false}"#);
    }

    #[test]
    fn test_empty_string() {
        let value = json!({"text": ""});
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, r#"{"text":""}"#);
    }

    #[test]
    fn test_empty_object() {
        let value = json!({});
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_empty_array() {
        let value = json!([]);
        let result = to_canonical_json_string(&value).unwrap();
        assert_eq!(result, "[]");
    }
}

mod determinism {
    use super::*;

    #[test]
    fn test_repeated_calls_identical() {
        let value = json!({"key": "value", "nested": {"a": 1}});

        let results: Vec<_> = (0..100)
            .map(|_| to_canonical_json(&value).unwrap())
            .collect();

        let first = &results[0];
        for result in &results[1..] {
            assert_eq!(first, result);
        }
    }

    #[test]
    fn test_different_construction_same_result() {
        // Build same object different ways
        let v1 = json!({"a": 1, "b": 2});
        let v2 = json!({"b": 2, "a": 1});

        let mut map = serde_json::Map::new();
        map.insert("b".to_string(), json!(2));
        map.insert("a".to_string(), json!(1));
        let v3 = serde_json::Value::Object(map);

        let r1 = to_canonical_json(&v1).unwrap();
        let r2 = to_canonical_json(&v2).unwrap();
        let r3 = to_canonical_json(&v3).unwrap();

        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }
}

mod unicode {
    use super::*;

    #[test]
    fn test_unicode_preserved() {
        let value = json!({"greeting": "Hello 世界 🌍"});
        let result = to_canonical_json_string(&value).unwrap();

        assert!(result.contains("世界"));
        assert!(result.contains("🌍"));
    }

    #[test]
    fn test_emoji_preserved() {
        let value = json!({"emoji": "😀🎉🚀"});
        let result = to_canonical_json_string(&value).unwrap();

        assert!(result.contains("😀"));
        assert!(result.contains("🎉"));
        assert!(result.contains("🚀"));
    }

    #[test]
    fn test_mixed_scripts() {
        let value = json!({
            "english": "Hello",
            "chinese": "你好",
            "arabic": "مرحبا",
            "russian": "Привет"
        });
        let result = to_canonical_json_string(&value).unwrap();

        // All scripts should be preserved
        assert!(result.contains("Hello"));
        assert!(result.contains("你好"));
        assert!(result.contains("مرحبا"));
        assert!(result.contains("Привет"));
    }
}
