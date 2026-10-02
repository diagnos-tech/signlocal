use std::path::PathBuf;

use serde_json::json;
use websign_project::EXTENSION_DEV_ID;

use super::*;

const HOST: &str = "/usr/bin/websign";

fn chromium(path: &str, origins: &[String]) -> String {
    json!({
        "name": NATIVE_HOST,
        "path": path,
        "type": "stdio",
        "allowed_origins": origins,
    })
    .to_string()
}

fn ours() -> Vec<String> {
    vec![format!("chrome-extension://{EXTENSION_DEV_ID}/")]
}

fn judge(text: &str, family: Family) -> RegistrationState {
    evaluate(
        text,
        family,
        Path::new("/etc/opt/chrome/native-messaging-hosts/x.json"),
        Path::new(HOST),
    )
}

#[test]
fn what_register_writes_reads_back_as_registered() {
    for family in [Family::Chromium, Family::Firefox] {
        let text = manifest::render(family, Path::new(HOST), &ours()).unwrap();
        assert_eq!(judge(&text, family), RegistrationState::Registered);
    }
}

#[test]
fn another_program_is_reported_with_its_path() {
    assert_eq!(
        judge(&chromium("/opt/old/websign", &ours()), Family::Chromium),
        RegistrationState::PointsElsewhere {
            path: "/opt/old/websign".into()
        }
    );
}

#[test]
fn a_relative_path_is_resolved_against_the_manifest_folder() {
    let state = evaluate(
        &chromium("websign", &ours()),
        Family::Chromium,
        Path::new("/usr/bin/x.json"),
        Path::new(HOST),
    );
    assert_eq!(state, RegistrationState::Registered);
}

#[test]
fn broken_manifests_say_why() {
    let cases = [
        ("{ not json".to_owned(), "JSON"),
        (
            json!({"name": "other.host", "path": HOST, "allowed_origins": ours()}).to_string(),
            "another host",
        ),
        (
            chromium(
                HOST,
                &["chrome-extension://abcdefghijklmnopabcdefghijklmnop/".into()],
            ),
            "extension",
        ),
        (
            json!({"name": NATIVE_HOST, "allowed_origins": ours()}).to_string(),
            "program path",
        ),
        (json!([NATIVE_HOST]).to_string(), "another host"),
        (
            json!({"name": 7, "path": HOST, "allowed_origins": ours()}).to_string(),
            "another host",
        ),
        (
            json!({"name": NATIVE_HOST, "path": HOST, "allowed_origins": ours()[0]}).to_string(),
            "extension",
        ),
        (
            json!({"name": NATIVE_HOST, "path": 1, "allowed_origins": ours()}).to_string(),
            "program path",
        ),
    ];
    for (text, needle) in cases {
        match judge(&text, Family::Chromium) {
            RegistrationState::Broken { reason } => assert!(reason.contains(needle), "{reason}"),
            other => panic!("{text}: {other:?}"),
        }
    }
}

#[test]
fn firefox_needs_the_gecko_id_not_origins() {
    let text = chromium(HOST, &ours());
    assert!(matches!(
        judge(&text, Family::Firefox),
        RegistrationState::Broken { .. }
    ));
    let firefox = manifest::render(Family::Firefox, &PathBuf::from(HOST), &[]).unwrap();
    assert!(matches!(
        judge(&firefox, Family::Chromium),
        RegistrationState::Broken { .. }
    ));
}
