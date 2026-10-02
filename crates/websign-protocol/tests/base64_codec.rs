//! Strict RFC 4648 §4 Base64 (SPEC §7) and the `Base64Bytes` wrapper.

use serde_json::json;
use websign_protocol::base64::{decode, encode};
use websign_protocol::types::Base64Bytes;

#[test]
fn rfc_4648_vectors() {
    let vectors = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];
    for (plain, encoded) in vectors {
        assert_eq!(encode(plain.as_bytes()), encoded);
        assert_eq!(decode(encoded).unwrap(), plain.as_bytes());
    }
}

#[test]
fn standard_alphabet_uses_plus_and_slash() {
    assert_eq!(encode(&[0xfb, 0xff, 0xbf]), "+/+/");
    assert_eq!(encode(&[0xff, 0xff]), "//8=");
    assert_eq!(decode("+/+/").unwrap(), [0xfb, 0xff, 0xbf]);
}

#[test]
fn protocol_md_digest_example_is_32_bytes() {
    let bytes = decode("n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg=").unwrap();
    assert_eq!(bytes.len(), 32);
    assert_eq!(
        encode(&bytes),
        "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="
    );
}

#[test]
fn every_byte_value_round_trips() {
    let all: Vec<u8> = (0..=255).collect();
    assert_eq!(decode(&encode(&all)).unwrap(), all);
    for len in 0..70usize {
        let bytes: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
        let text = encode(&bytes);
        assert_eq!(text.len(), len.div_ceil(3) * 4);
        assert_eq!(decode(&text).unwrap(), bytes);
    }
}

#[test]
fn output_is_canonical() {
    for len in 0..20 {
        let text = encode(&vec![0xAB; len]);
        assert!(
            text.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'='))
        );
    }
}

#[test]
fn rejects_whitespace_anywhere() {
    for text in [
        "Zm9v\n", " Zm9v", "Zm 9v", "Zm9v\r\n", "Zm\t9v", "Zm9v ", "\nZm9v",
    ] {
        assert!(decode(text).is_err(), "{text:?}");
    }
}

#[test]
fn rejects_the_url_safe_alphabet() {
    assert!(decode("-_-_").is_err());
    assert!(decode("+_+_").is_err());
    assert!(decode("-/-/").is_err());
}

#[test]
fn requires_padding() {
    for text in ["Zg", "Zm8", "Zm9vYg", "Zm9vYmE", "Z", "Zm9vY"] {
        assert!(decode(text).is_err(), "{text:?}");
    }
}

#[test]
fn rejects_misplaced_or_excess_padding() {
    for text in [
        "Z===", "====", "Zg=a", "Zg==Zg==", "Zm9v====", "=Zg=", "Zm=v", "Zg===", "Zm8==",
    ] {
        assert!(decode(text).is_err(), "{text:?}");
    }
}

#[test]
fn rejects_nonzero_unused_trailing_bits() {
    // "Zh==" and "Zm9=" decode to the same bytes as "Zg==" and "Zm8=" under a lenient decoder.
    for text in ["Zh==", "Zm9=", "Zg9=", "AB==", "AAB="] {
        assert!(decode(text).is_err(), "{text:?}");
    }
    assert!(decode("Zg==").is_ok());
    assert!(decode("Zm8=").is_ok());
}

#[test]
fn rejects_non_alphabet_and_non_ascii_bytes() {
    for text in ["Zm9!", "Zm9é", "Zm9\0", "Zm9.", "日本語!", "Zm9v\u{200b}"] {
        assert!(decode(text).is_err(), "{text:?}");
    }
}

#[test]
fn base64_bytes_wrapper() {
    let bytes = Base64Bytes::new(b"foobar".to_vec());
    assert_eq!(bytes.as_bytes(), b"foobar");
    assert_eq!(serde_json::to_value(&bytes).unwrap(), json!("Zm9vYmFy"));
    let parsed: Base64Bytes = serde_json::from_value(json!("Zm9vYmFy")).unwrap();
    assert_eq!(parsed, bytes);
    assert_eq!(parsed.into_bytes(), b"foobar");
}

#[test]
fn base64_bytes_parsing_is_strict() {
    for bad in [
        json!("Zm9v\n"),
        json!("Zg"),
        json!("Zh=="),
        json!("-_-_"),
        json!(5),
        json!(null),
        json!([1, 2]),
    ] {
        assert!(
            serde_json::from_value::<Base64Bytes>(bad.clone()).is_err(),
            "{bad}"
        );
    }
    let empty: Base64Bytes = serde_json::from_value(json!("")).unwrap();
    assert!(empty.as_bytes().is_empty());
}

#[test]
fn base64_bytes_debug_does_not_leak_content() {
    let text = format!("{:?}", Base64Bytes::new(b"secret-signature-bytes".to_vec()));
    assert!(!text.contains("secret"));
    assert!(!text.contains(&encode(b"secret-signature-bytes")));
}
