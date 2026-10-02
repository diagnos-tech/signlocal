//! `RequestId` (SPEC §3) and `FingerprintHex` (SPEC §4).

use serde_json::json;
use websign_protocol::RequestId;
use websign_protocol::types::FingerprintHex;

#[test]
fn valid_request_ids() {
    for text in [
        "1",
        "h",
        "p1.3.abc",
        "a:b-c_d",
        &"a".repeat(64),
        "A",
        "Z9",
        "...",
        "---",
        "::",
        "___",
    ] {
        let id = RequestId::new(text).unwrap_or_else(|_| panic!("{text:?} should be valid"));
        assert_eq!(id.as_str(), text);
        assert_eq!(id.to_string(), text);
    }
}

#[test]
fn every_allowed_character_is_accepted() {
    let allowed = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._:-";
    for c in allowed.chars() {
        assert!(RequestId::new(c.to_string()).is_ok(), "{c:?}");
    }
}

#[test]
fn invalid_request_ids() {
    let long = "a".repeat(65);
    for text in [
        "",
        long.as_str(),
        "a b",
        "é",
        "a\n",
        "<x>",
        "a/b",
        "a\\b",
        "a\0",
        "a\"b",
        "a,b",
        "a;b",
        "a@b",
        "a#b",
        "a+b",
        "a=b",
        " a",
        "a ",
        "\t",
        "ａ",
        "日本",
    ] {
        assert!(RequestId::new(text).is_err(), "{text:?} should be invalid");
    }
}

#[test]
fn length_limit_counts_bytes_not_characters() {
    assert!(RequestId::new("é".repeat(32)).is_err());
    assert!(RequestId::new("a".repeat(64)).is_ok());
    assert!(RequestId::new("a".repeat(65)).is_err());
}

#[test]
fn request_id_serde_uses_the_same_check() {
    let id: RequestId = serde_json::from_value(json!("p1.3.abc")).unwrap();
    assert_eq!(serde_json::to_value(&id).unwrap(), json!("p1.3.abc"));
    for bad in [
        json!(""),
        json!("a b"),
        json!("a".repeat(65)),
        json!(1),
        json!(null),
        json!(["a"]),
    ] {
        assert!(
            serde_json::from_value::<RequestId>(bad.clone()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn request_ids_are_ordered_hashable_and_comparable() {
    use std::collections::HashSet;
    let a = RequestId::new("a").unwrap();
    let b = RequestId::new("b").unwrap();
    assert!(a < b);
    let set: HashSet<_> = [a.clone(), a.clone(), b].into_iter().collect();
    assert_eq!(set.len(), 2);
    assert!(set.contains(&a));
}

#[test]
fn valid_fingerprints() {
    for text in [
        "0".repeat(64),
        "f".repeat(64),
        "256adb9a".repeat(8),
        "0123456789abcdef".repeat(4),
    ] {
        assert_eq!(FingerprintHex::new(text.clone()).unwrap().as_str(), text);
    }
}

#[test]
fn invalid_fingerprints() {
    let valid = "256adb9a".repeat(8);
    let cases = [
        valid.to_uppercase(),
        format!("{}A{}", &valid[..10], &valid[11..]),
        valid[..63].to_string(),
        format!("{valid}0"),
        valid
            .as_bytes()
            .chunks(2)
            .map(|pair| std::str::from_utf8(pair).unwrap())
            .collect::<Vec<_>>()
            .join(":"),
        format!(" {}", &valid[1..]),
        format!("{}g", &valid[..63]),
        format!("{}é", &valid[..62]),
        String::new(),
    ];
    for text in cases {
        assert!(FingerprintHex::new(text.clone()).is_err(), "{text:?}");
    }
}

#[test]
fn fingerprint_serde_uses_the_same_check() {
    let text = "256adb9a".repeat(8);
    let parsed: FingerprintHex = serde_json::from_value(json!(text)).unwrap();
    assert_eq!(serde_json::to_value(&parsed).unwrap(), json!(text));
    assert!(serde_json::from_value::<FingerprintHex>(json!(text.to_uppercase())).is_err());
    assert!(serde_json::from_value::<FingerprintHex>(json!(1)).is_err());
}
