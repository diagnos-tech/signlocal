use websign_protocol::types::BrowserName;

use super::*;

fn error(at: i64) -> ErrorRecord {
    ErrorRecord {
        at,
        operation: "sign".into(),
        code: "Internal".into(),
        source: "host".into(),
        native: None,
    }
}

#[test]
fn remember_creates_then_refreshes_and_orders_by_recency() {
    let mut consent = ConsentDocument::default();
    consent.remember("https://a.example", "aa", 10);
    consent.remember("https://a.example", "bb", 20);
    consent.remember("https://a.example", "aa", 30);
    let record = consent.get("https://a.example").unwrap();
    assert_eq!(record.remembered_at, 10);
    assert_eq!(record.last_used_at, 30);
    assert_eq!(record.certificates, ["aa", "bb"]);
    assert_eq!(consent.records.len(), 1);
}

#[test]
fn record_use_never_creates_consent() {
    let mut consent = ConsentDocument::default();
    consent.record_use("https://a.example", "aa", 10);
    assert!(consent.records.is_empty());
    consent.remember("https://a.example", "aa", 10);
    consent.record_use("https://a.example", "bb", 20);
    assert_eq!(consent.records[0].certificates, ["bb", "aa"]);
}

#[test]
fn revoke_removes_the_caller() {
    let mut consent = ConsentDocument::default();
    consent.remember("https://a.example", "aa", 1);
    consent.revoke("https://a.example");
    assert!(consent.get("https://a.example").is_none());
}

#[test]
fn certificates_per_caller_are_capped_at_twenty() {
    let mut consent = ConsentDocument::default();
    for n in 0..30 {
        consent.remember("k", &format!("{n:02}"), n);
    }
    let record = consent.get("k").unwrap();
    assert_eq!(record.certificates.len(), 20);
    assert_eq!(record.certificates[0], "29");
}

#[test]
fn usage_is_newest_first_deduplicated_and_capped() {
    let mut usage = UsageDocument::default();
    for n in 0..120 {
        usage.record(&n.to_string());
    }
    usage.record("50");
    assert_eq!(usage.fingerprints.len(), 100);
    assert_eq!(usage.fingerprints[0], "50");
    assert_eq!(usage.fingerprints.iter().filter(|f| *f == "50").count(), 1);
}

#[test]
fn errors_keep_the_newest_twenty_last() {
    let mut errors = ErrorsDocument::default();
    for at in 0..25 {
        errors.record(error(at));
    }
    assert_eq!(errors.records.len(), MAX_RECENT_ERRORS);
    assert_eq!(errors.records[0].at, 5);
    assert_eq!(errors.records[19].at, 24);
}

#[test]
fn connections_keep_one_record_per_browser() {
    let record = |browser, seen| ConnectionRecord {
        browser,
        browser_version: "1".into(),
        extension_version: "1".into(),
        last_seen: seen,
    };
    let mut doc = ConnectionsDocument::default();
    doc.record(record(BrowserName::Chrome, 1));
    doc.record(record(BrowserName::Firefox, 2));
    doc.record(record(BrowserName::Chrome, 3));
    assert_eq!(doc.records.len(), 2);
    assert_eq!(doc.records.last().unwrap().last_seen, 3);
}

#[test]
fn settings_flatten_next_to_the_version() {
    let mut doc = SettingsDocument::default();
    doc.settings.onboarding_dismissed = true;
    let json = serde_json::to_value(&doc).unwrap();
    assert_eq!(json["version"], 1);
    assert_eq!(json["onboardingDismissed"], true);
    let back: SettingsDocument = serde_json::from_value(json).unwrap();
    assert_eq!(back, doc);
}
