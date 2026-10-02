//! The built binary as callers see it: stdout, stderr and exit codes, in a
//! throwaway home so nothing touches the real profile.

use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn websign(home: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_websign"))
        .args(args)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("LOCALAPPDATA", home.join("local"))
        .env("APPDATA", home.join("roaming"))
        .env("WEBSIGN_LOCALE", "en")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn version_prints_text_or_app_info() {
    let home = tempfile::tempdir().unwrap();
    let out = websign(home.path(), &["version"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).starts_with("websign ") && stdout(&out).contains("(protocol 1)"));
    let out = websign(home.path(), &["version", "--json"]);
    let info: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(info["protocols"]["max"], 1);
    assert_eq!(info["channel"], "direct");
}

#[test]
fn usage_errors_exit_2_with_nothing_on_stdout() {
    let home = tempfile::tempdir().unwrap();
    for args in [
        &["daemon"][..],
        &["sign", "--hash", "SHA-256"],
        &["install", "--browser", "x"],
    ] {
        let out = websign(home.path(), args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn a_bad_digest_is_one_json_error_and_exit_14() {
    let home = tempfile::tempdir().unwrap();
    let out = websign(
        home.path(),
        &["sign", "--hash", "SHA-256", "--digest", "abcd"],
    );
    assert_eq!(out.status.code(), Some(14));
    let text = stdout(&out);
    assert_eq!(text.lines().count(), 1, "{text}");
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["type"], "error");
    assert_eq!(json["code"], "InvalidRequest");
    assert!(!out.stderr.is_empty(), "people get a line on stderr");
}

#[cfg(unix)]
#[test]
fn register_dry_run_reports_json_and_changes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let profile = home.path().join("profile");
    let args = [
        "register",
        "--browser",
        "chromium",
        "--dry-run",
        "--json",
        "--user-data-dir",
    ];
    let out = websign(
        home.path(),
        &[&args[..], &[profile.to_str().unwrap()]].concat(),
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(report["command"], "register");
    assert_eq!(report["steps"][0]["outcome"], "dryRun");
    assert!(!profile.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn install_then_uninstall_in_a_throwaway_home() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join("config/chromium")).unwrap();
    let out = websign(home.path(), &["install", "--json"]);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let written = report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["outcome"] == "written");
    assert!(written, "{report}");
    // Installing again changes nothing that matters and still succeeds.
    let again = websign(home.path(), &["install", "--json"]);
    assert_eq!(again.status.code(), Some(0), "{}", stdout(&again));
    let out = websign(home.path(), &["uninstall", "--purge", "--json"]);
    let report: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let removed = report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["outcome"] == "removed");
    assert!(removed, "{report}");
    assert!(!home.path().join("config/websign").exists(), "{report}");
    // Uninstalling what is already gone is not a failure either. (Only the
    // log folder comes back: this very run logs before it purges.)
    let again = websign(home.path(), &["uninstall", "--purge", "--json"]);
    let report: serde_json::Value = serde_json::from_str(&stdout(&again)).unwrap();
    assert_eq!(again.status.code(), Some(0), "{report}");
    let removed_again = report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["outcome"] == "removed" && s["target"] != "logs");
    assert!(!removed_again, "{report}");
}

#[test]
fn foreign_extension_arguments_never_start_a_host() {
    let home = tempfile::tempdir().unwrap();
    let out = websign(
        home.path(),
        &["chrome-extension://abcdefghijklmnopabcdefghijklmnop/"],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn connect_ends_when_stdin_closes() {
    let home = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_websign"))
        .arg("connect")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdin.take().map(|mut stdin| stdin.flush()));
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "connect kept running"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
