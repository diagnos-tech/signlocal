//! `negotiate` (SPEC §2) and the protocol range type.

mod common;

use serde_json::json;
use websign_protocol::version::{OLDEST_SUPPORTED_VERSION, ProtocolRange};
use websign_protocol::{ErrorCode, PROTOCOL_VERSION, negotiate};

fn range(min: u32, max: u32) -> ProtocolRange {
    ProtocolRange { min, max }
}

#[test]
fn spec_table() {
    assert_eq!(negotiate(range(1, 1), range(1, 1)), Ok(1));
    assert_eq!(negotiate(range(1, 3), range(2, 5)), Ok(3));
    assert_eq!(
        negotiate(range(2, 3), range(1, 1)),
        Err(ErrorCode::ClientOutdated)
    );
    assert_eq!(
        negotiate(range(1, 1), range(2, 2)),
        Err(ErrorCode::AppOutdated)
    );
    assert_eq!(
        negotiate(range(1, 1), range(0, 1)),
        Err(ErrorCode::InvalidRequest)
    );
    assert_eq!(
        negotiate(range(1, 1), range(2, 1)),
        Err(ErrorCode::InvalidRequest)
    );
}

#[test]
fn picks_the_highest_common_version() {
    assert_eq!(negotiate(range(1, 5), range(3, 4)), Ok(4));
    assert_eq!(negotiate(range(3, 4), range(1, 5)), Ok(4));
    assert_eq!(negotiate(range(1, 3), range(3, 9)), Ok(3));
    assert_eq!(negotiate(range(4, 6), range(1, 4)), Ok(4));
    assert_eq!(negotiate(range(1, 3), range(1, 3)), Ok(3));
}

#[test]
fn outdated_direction_follows_who_is_behind() {
    assert_eq!(
        negotiate(range(3, 5), range(1, 2)),
        Err(ErrorCode::ClientOutdated)
    );
    assert_eq!(
        negotiate(range(1, 2), range(3, 5)),
        Err(ErrorCode::AppOutdated)
    );
}

#[test]
fn constants_describe_the_current_catalog() {
    assert_eq!(PROTOCOL_VERSION, 1);
    assert_eq!(OLDEST_SUPPORTED_VERSION, 1);
    assert_eq!(ProtocolRange::CURRENT, range(1, 1));
    assert_eq!(
        negotiate(ProtocolRange::CURRENT, ProtocolRange::CURRENT),
        Ok(PROTOCOL_VERSION)
    );
}

#[test]
fn range_json_shape() {
    assert_eq!(
        serde_json::to_value(range(1, 3)).unwrap(),
        json!({ "min": 1, "max": 3 })
    );
    assert_eq!(
        serde_json::from_value::<ProtocolRange>(json!({ "min": 2, "max": 4 })).unwrap(),
        range(2, 4)
    );
}

#[test]
fn range_parsing_is_strict() {
    for bad in [
        json!({ "min": 1 }),
        json!({ "max": 1 }),
        json!({ "min": 1, "max": 1, "extra": 1 }),
        json!({ "min": "1", "max": 1 }),
        json!({ "min": -1, "max": 1 }),
        json!({ "min": 1.5, "max": 2 }),
        json!([1, 1]),
    ] {
        assert!(
            serde_json::from_value::<ProtocolRange>(bad.clone()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn hello_carries_the_client_range_end_to_end() {
    let mut frame = common::hello();
    frame["protocols"] = json!({ "min": 1, "max": 7 });
    let parsed = common::parse_client(&frame, None).unwrap();
    match parsed.message {
        websign_protocol::ClientMessage::Hello(hello) => assert_eq!(hello.protocols, range(1, 7)),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn a_second_hello_parses_and_is_left_to_the_session() {
    // SPEC §5: after negotiation `hello` is an ordinary message (v must match);
    // the session refuses it as a second hello.
    let frame = common::hello();
    assert!(common::parse_client(&frame, Some(1)).is_ok());
    let error = common::parse_client(&frame, Some(2)).expect_err("v must match");
    assert_eq!(error.code, ErrorCode::InvalidRequest);
}

#[test]
fn the_app_hello_speaks_the_version_it_picked() {
    // SPEC §5: before negotiation the app's `hello` must have `v == protocol`.
    let reply = common::hello_reply();
    assert!(common::parse_app(&reply, None).is_ok());
    let mut picked_two = reply.clone();
    picked_two["protocol"] = json!(2);
    let error = common::parse_app(&picked_two, None).expect_err("v 1 but protocol 2");
    assert_eq!(error.code, ErrorCode::InvalidRequest);
    picked_two["v"] = json!(2);
    assert!(common::parse_app(&picked_two, None).is_ok());
}
