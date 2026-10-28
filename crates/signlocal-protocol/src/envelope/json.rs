//! Reads a frame into a JSON value, refusing repeated object keys.
//!
//! `serde_json::Value` silently keeps the last of two equal keys, while other
//! parsers (and the extension's validator) may keep the first: the same bytes
//! would be two different messages. A frame with a repeated key, at any depth
//! and after unescaping (`"id"` and `"id"` are equal), is not a message.
//! Depth is bounded by `serde_json`'s recursion limit (128).

use std::fmt;

use serde::de::{Deserialize, Deserializer, Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

/// The frame as a JSON value, or `None` when it is not UTF-8 JSON, has
/// trailing data or repeats a key.
pub(super) fn read(frame: &[u8]) -> Option<Value> {
    let mut deserializer = serde_json::Deserializer::from_slice(frame);
    let UniqueKeys(value) = UniqueKeys::deserialize(&mut deserializer).ok()?;
    deserializer.end().ok()?;
    Some(value)
}

struct UniqueKeys(Value);

impl<'de> Deserialize<'de> for UniqueKeys {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_any(UniqueKeysVisitor)
            .map(UniqueKeys)
    }
}

struct UniqueKeysVisitor;

impl<'de> Visitor<'de> for UniqueKeysVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON without repeated object keys")
    }

    fn visit_unit<E: Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<Value, E> {
        // JSON cannot spell NaN or infinity, so this never falls back.
        Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(UniqueKeys(item)) = seq.next_element()? {
            items.push(item);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut object = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(A::Error::custom("repeated object key"));
            }
            let UniqueKeys(value) = map.next_value()?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_json_like_serde_json() {
        let frame = br#"{"a":[1,-2,3.5,"x",true,null,{"b":{}}]}"#;
        let expected: Value = serde_json::from_slice(frame).expect("valid JSON");
        assert_eq!(read(frame), Some(expected));
    }

    #[test]
    fn refuses_repeated_keys_at_any_depth_and_spelling() {
        for frame in [
            r#"{"id":"a","id":"b"}"#,
            r#"{"id":"a","id":"a"}"#,
            r#"{"web":{"origin":"a","origin":"a"}}"#,
            r#"[{"x":1,"x":1}]"#,
        ] {
            assert_eq!(read(frame.as_bytes()), None, "{frame}");
        }
    }

    #[test]
    fn refuses_trailing_data_and_excessive_depth() {
        assert_eq!(read(b"{} {}"), None);
        assert_eq!(read("[".repeat(200).as_bytes()), None);
        let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
        assert_eq!(read(deep.as_bytes()), None);
    }
}
