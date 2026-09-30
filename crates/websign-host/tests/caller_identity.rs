//! SPEC §3: `Caller`, its consent key and whether it can be remembered.

mod common;

use common::harness::desktop_caller;
use websign_core::present::caller::{CodeSigner, DesktopCaller};
use websign_core::present::origin::OriginError;
use websign_host::Caller;
use websign_protocol::types::{BrowserInfo, BrowserName, HelloReason, WebContext};

fn browser() -> BrowserInfo {
    BrowserInfo {
        name: BrowserName::Firefox,
        version: "131.0".to_owned(),
        reason: HelloReason::Popup,
    }
}

fn context(origin: &str, top: &str) -> WebContext {
    WebContext {
        origin: origin.to_owned(),
        top_origin: top.to_owned(),
    }
}

#[test]
fn a_page_in_its_own_tab_has_no_top_origin() {
    let same = "https://app.example.com";
    let caller = Caller::web(&context(same, same), &browser()).expect("secure");
    match &caller {
        Caller::Web {
            origin,
            top,
            browser: b,
        } => {
            assert_eq!(origin.canonical, same);
            assert_eq!(*top, None);
            assert_eq!(b.name, BrowserName::Firefox);
        }
        other => panic!("expected web, got {other:?}"),
    }
    assert_eq!(caller.consent_key(), same);
}

#[test]
fn a_frame_shows_the_tab_origin_as_top() {
    let caller = Caller::web(
        &context("https://widget.example.net", "https://app.example.com"),
        &browser(),
    )
    .expect("secure");
    match caller {
        Caller::Web { origin, top, .. } => {
            assert_eq!(origin.canonical, "https://widget.example.net");
            assert_eq!(top.expect("top").canonical, "https://app.example.com");
        }
        other => panic!("expected web, got {other:?}"),
    }
}

#[test]
fn the_consent_key_is_the_frame_origin_not_the_tab() {
    let caller = Caller::web(
        &context("https://widget.example.net", "https://app.example.com"),
        &browser(),
    )
    .expect("secure");
    assert_eq!(caller.consent_key(), "https://widget.example.net");
}

#[test]
fn insecure_and_malformed_origins_are_refused_for_either_field() {
    let ok = "https://app.example.com";
    let insecure = "http://evil.example";
    assert_eq!(
        Caller::web(&context(insecure, insecure), &browser()).expect_err("no"),
        OriginError::Insecure
    );
    assert_eq!(
        Caller::web(&context(ok, insecure), &browser()).expect_err("no"),
        OriginError::Insecure
    );
    assert_eq!(
        Caller::web(&context("nonsense", "nonsense"), &browser()).expect_err("no"),
        OriginError::Malformed
    );
}

#[test]
fn ip_and_idn_origins_cannot_be_remembered() {
    for origin in ["https://192.0.2.10", "https://xn--dignos-4nf.health"] {
        let caller = Caller::web(&context(origin, origin), &browser()).expect("secure");
        assert!(!caller.can_remember(), "{origin}");
    }
    let normal = Caller::web(
        &context("https://app.example.com", "https://app.example.com"),
        &browser(),
    )
    .expect("secure");
    assert!(normal.can_remember());
}

#[test]
fn desktop_programs_can_always_be_remembered_and_use_app_or_path_keys() {
    let unsigned = Caller::Desktop(desktop_caller());
    assert!(unsigned.can_remember());
    assert_eq!(unsigned.consent_key(), "path:/opt/tools/invoicer");

    let signed = Caller::Desktop(DesktopCaller {
        signer: Some(CodeSigner::Apple {
            team_id: "ABCDE12345".to_owned(),
            identifier: "com.example.invoicer".to_owned(),
        }),
        ..desktop_caller()
    });
    assert!(signed.can_remember());
    assert!(signed.consent_key().starts_with("app:"));
}
