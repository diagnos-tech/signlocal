//! Rules the parser enforces after the body parses (SPEC §5 step 7).

mod common;

use common::*;
use serde_json::{Value, json};
use websign_protocol::limits::{MAX_CHAIN_LEN, MAX_ORIGIN_LEN, MAX_SHORT_TEXT_LEN};

fn origin_of_len(len: usize) -> String {
    let prefix = "https://";
    let suffix = ".example";
    format!(
        "{prefix}{}{suffix}",
        "a".repeat(len - prefix.len() - suffix.len())
    )
}

#[test]
fn origins_are_bounded() {
    for field in ["origin", "topOrigin"] {
        let mut frame = sign_begin();
        frame["web"][field] = json!(origin_of_len(MAX_ORIGIN_LEN));
        assert!(parse_client(&frame, Some(1)).is_ok(), "{field}");
        frame["web"][field] = json!(origin_of_len(MAX_ORIGIN_LEN + 1));
        assert_names_field_not_value(frame, &format!("web.{field}"), "aaaa");
    }
}

#[test]
fn client_texts_are_bounded() {
    let longest = "x".repeat(MAX_SHORT_TEXT_LEN);
    let too_long = "x".repeat(MAX_SHORT_TEXT_LEN + 1);
    for (object, field) in [
        ("client", "name"),
        ("client", "version"),
        ("browser", "version"),
    ] {
        let mut frame = hello();
        frame[object][field] = json!(longest);
        assert!(parse_client(&frame, None).is_ok(), "{object}.{field}");
        frame[object][field] = json!(too_long);
        let error = assert_invalid(parse_client(&frame, None), id("h"));
        assert!(
            error.message.contains(&format!("{object}.{field}")),
            "{error:?}"
        );
        assert!(!error.message.contains("xxxx"), "{error:?}");
    }
}

#[test]
fn algorithm_lists_are_absent_or_not_empty() {
    assert_names_field_not_value(
        with(sign_begin(), "algorithms", json!([])),
        "algorithms",
        "[]",
    );
    let choose = json!({ "v": 1, "id": "2", "type": "choose", "filter": { "algorithms": [] } });
    assert_names_field_not_value(choose, "filter.algorithms", "[]");
    let choose = json!({ "v": 1, "id": "2", "type": "choose", "filter": {} });
    assert!(parse_client(&choose, Some(1)).is_ok());
}

#[test]
fn choose_result_holds_at_least_one_certificate() {
    let empty = json!({ "v": 1, "id": "2", "type": "choose.result", "certificates": [] });
    let error = assert_invalid(parse_app(&empty, Some(1)), id("2"));
    assert!(error.message.contains("certificates"), "{error:?}");
}

#[test]
fn chains_are_bounded() {
    let chain = |len: usize| Value::Array(vec![json!("MIIG"); len]);
    let mut certificate = certificate();
    certificate["chain"] = chain(MAX_CHAIN_LEN);
    let frame = with(need_digest(), "certificate", certificate.clone());
    assert!(parse_app(&frame, Some(1)).is_ok());
    certificate["chain"] = chain(MAX_CHAIN_LEN + 1);
    for frame in [
        with(need_digest(), "certificate", certificate.clone()),
        with(sign_result(), "certificate", certificate.clone()),
        json!({ "v": 1, "id": "3", "type": "choose.result", "certificates": [certificate] }),
    ] {
        let error = assert_invalid(parse_app(&frame, Some(1)), id("3"));
        assert!(error.message.contains("chain"), "{error:?}");
    }
}

#[test]
fn digest_length_is_left_to_the_app() {
    // The length depends on the request's `hash`, which the frame does not carry.
    for digest in ["", "AA==", DIGEST_B64] {
        assert!(parse_client(&with(sign_digest(), "digest", json!(digest)), Some(1)).is_ok());
    }
}
