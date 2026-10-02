//! `devices.json` must satisfy `devices.schema.json`. No JSON Schema crate is
//! a dependency of the workspace, so this validates the keyword subset the
//! schema uses and fails loudly on any keyword or pattern it does not know:
//! extending the schema means extending this test.

use serde_json::Value;

fn load(name: &str) -> Value {
    let path = format!("{}/../../{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn pattern_matches(pattern: &str, text: &str) -> bool {
    match pattern {
        "^[a-z0-9]+(-[a-z0-9]+)*$" => {
            !text.is_empty()
                && text.split('-').all(|part| {
                    !part.is_empty()
                        && part
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                })
        }
        "^[0-9a-f]{4}:[0-9a-f]{4}$" => {
            let hex =
                |s: &str| s.len() == 4 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
            text.split_once(':').is_some_and(|(v, p)| hex(v) && hex(p))
        }
        "^([0-9A-F]{2}|\\.\\.)+$" => {
            !text.is_empty()
                && text.len().is_multiple_of(2)
                && text.as_bytes().chunks(2).all(|pair| {
                    pair == b".." || pair.iter().all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'F'))
                })
        }
        "^https://" => text.starts_with("https://"),
        other => panic!("test does not know the pattern {other}"),
    }
}

/// Keywords `check` implements, plus annotations that constrain nothing.
const KNOWN_KEYWORDS: &[&str] = &[
    "$ref",
    "const",
    "enum",
    "type",
    "minLength",
    "maxLength",
    "pattern",
    "items",
    "properties",
    "required",
    "additionalProperties",
    "$schema",
    "$id",
    "$defs",
    "title",
    "description",
];

fn resolve<'a>(root: &'a Value, reference: &str) -> &'a Value {
    let path = reference.strip_prefix("#/").expect("local $ref only");
    path.split('/').fold(root, |node, key| &node[key])
}

fn report(errors: &mut Vec<String>, at: &str, message: String) {
    errors.push(format!("{at}: {message}"));
}

fn check(root: &Value, schema: &Value, value: &Value, at: &str, errors: &mut Vec<String>) {
    for keyword in schema.as_object().into_iter().flat_map(|map| map.keys()) {
        assert!(
            KNOWN_KEYWORDS.contains(&keyword.as_str()),
            "test does not know the keyword {keyword}"
        );
    }
    if let Some(reference) = schema.get("$ref") {
        let target = resolve(root, reference.as_str().expect("$ref is a string"));
        return check(root, target, value, at, errors);
    }
    if let Some(expected) = schema.get("const")
        && expected != value
    {
        return report(errors, at, format!("expected {expected}, found {value}"));
    }
    if let Some(Value::Array(allowed)) = schema.get("enum")
        && !allowed.contains(value)
    {
        return report(errors, at, format!("{value} is not one of {allowed:?}"));
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("object") if !value.is_object() => {
            return report(errors, at, "expected an object".into());
        }
        Some("array") if !value.is_array() => {
            return report(errors, at, "expected an array".into());
        }
        Some("string") if !value.is_string() => {
            return report(errors, at, "expected a string".into());
        }
        Some("boolean") if !value.is_boolean() => {
            return report(errors, at, "expected a boolean".into());
        }
        Some("object" | "array" | "string" | "boolean") | None => {}
        Some(other) => panic!("test does not know the type {other}"),
    }
    if let Some(text) = value.as_str() {
        let length = text.chars().count() as u64;
        if schema
            .get("minLength")
            .and_then(Value::as_u64)
            .is_some_and(|min| length < min)
        {
            report(errors, at, "too short".into());
        }
        if schema
            .get("maxLength")
            .and_then(Value::as_u64)
            .is_some_and(|max| length > max)
        {
            report(errors, at, "too long".into());
        }
        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
            && !pattern_matches(pattern, text)
        {
            report(errors, at, format!("{text:?} does not match {pattern}"));
        }
    }
    if let Some(items) = value.as_array()
        && let Some(item_schema) = schema.get("items")
    {
        for (index, item) in items.iter().enumerate() {
            check(root, item_schema, item, &format!("{at}[{index}]"), errors);
        }
    }
    if let Some(object) = value.as_object() {
        let properties = schema.get("properties").and_then(Value::as_object);
        for key in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let key = key.as_str().expect("required names are strings");
            if !object.contains_key(key) {
                report(errors, at, format!("missing {key}"));
            }
        }
        for (key, child) in object {
            match properties.and_then(|p| p.get(key)) {
                Some(child_schema) => {
                    check(root, child_schema, child, &format!("{at}.{key}"), errors)
                }
                None if schema.get("additionalProperties") == Some(&Value::Bool(false)) => {
                    report(errors, at, format!("unexpected property {key}"));
                }
                None => {}
            }
        }
    }
}

fn validate(document: &Value) -> Vec<String> {
    let schema = load("devices.schema.json");
    let mut errors = Vec::new();
    check(&schema, &schema, document, "$", &mut errors);
    errors
}

#[test]
fn devices_json_is_valid_against_its_schema() {
    let errors = validate(&load("devices.json"));
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn the_validator_rejects_bad_documents() {
    let bad = serde_json::json!({
        "version": 2,
        "devices": [{
            "id": "Bad Id", "name": "", "kind": "gadget",
            "match": { "usb": ["0529:620"], "atr": ["3b"], "extra": [] },
            "driver": { "name": "x", "download": { "linux": "http://insecure" } }
        }]
    });
    let errors = validate(&bad).join("\n");
    for expected in [
        "expected 1",
        "Bad Id",
        "too short",
        "gadget",
        "0529:620",
        "\"3b\"",
        "unexpected property extra",
        "http://insecure",
    ] {
        assert!(
            errors.contains(expected),
            "missing {expected:?} in\n{errors}"
        );
    }
}
