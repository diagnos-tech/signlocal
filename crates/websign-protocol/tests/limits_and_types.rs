//! Documented limits (protocol.md §8) and small value types.

use std::time::Duration;

use serde_json::json;
use websign_protocol::limits::*;
use websign_protocol::types::{HashName, SignatureAlgorithmName};

#[test]
fn limits_match_the_documentation() {
    assert_eq!(MAX_INCOMING_FRAME, 1024 * 1024);
    assert_eq!(MAX_OUTGOING_FRAME, 1024 * 1024);
    assert_eq!(MAX_REQUEST_ID_LEN, 64);
    assert_eq!(MAX_ORIGIN_LEN, 512);
    assert_eq!(MAX_IN_FLIGHT_PER_CONNECTION, 16);
    assert_eq!(MAX_QUEUED_REQUESTS, 10);
    assert_eq!(HELLO_TIMEOUT, Duration::from_secs(5));
    assert_eq!(DIGEST_TIMEOUT, Duration::from_secs(60));
    assert_eq!(DECISION_TIMEOUT, Duration::from_secs(300));
    assert_eq!(DESKTOP_IDLE_EXIT, Duration::from_secs(300));
    assert_eq!(EXTENSION_IDLE_CLOSE, Duration::from_secs(60));
    assert_eq!(APP_RESPONSE_TIMEOUT, Duration::from_secs(3));
    assert_eq!(MAX_CHAIN_LEN, 8);
}

#[test]
fn hash_digest_lengths() {
    assert_eq!(HashName::Sha256.digest_len(), 32);
    assert_eq!(HashName::Sha384.digest_len(), 48);
    assert_eq!(HashName::Sha512.digest_len(), 64);
}

#[test]
fn hash_names_serialize_as_webcrypto_names() {
    assert_eq!(
        serde_json::to_value(HashName::Sha256).unwrap(),
        json!("SHA-256")
    );
    assert_eq!(
        serde_json::to_value(HashName::Sha384).unwrap(),
        json!("SHA-384")
    );
    assert_eq!(
        serde_json::to_value(HashName::Sha512).unwrap(),
        json!("SHA-512")
    );
    for bad in ["SHA256", "sha-256", "SHA-1", "SHA3-256", ""] {
        assert!(
            serde_json::from_value::<HashName>(json!(bad)).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn default_algorithm_preference_order() {
    assert_eq!(
        SignatureAlgorithmName::DEFAULT_PREFERENCE,
        [
            SignatureAlgorithmName::Ecdsa,
            SignatureAlgorithmName::RsaPkcs1v15,
            SignatureAlgorithmName::RsaPss
        ]
    );
    let names: Vec<_> = SignatureAlgorithmName::DEFAULT_PREFERENCE
        .iter()
        .map(|a| serde_json::to_value(a).unwrap())
        .collect();
    assert_eq!(
        names,
        [
            json!("ECDSA"),
            json!("RSASSA-PKCS1-v1_5"),
            json!("RSASSA-PSS")
        ]
    );
}
