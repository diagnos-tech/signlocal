//! The desktop entry in a throwaway data folder, with `xdg-mime` faked.

use std::cell::RefCell;

use super::*;

#[test]
fn register_writes_a_handler_entry_and_asks_xdg_mime_to_make_it_default() {
    let data = tempfile::tempdir().unwrap();
    let applications = data.path().join("applications");
    let calls = RefCell::new(Vec::new());
    let fake = |file: &str, mime: &str| {
        calls.borrow_mut().push((file.to_owned(), mime.to_owned()));
        Tool::Done
    };

    let outcome = register(&applications, Path::new("/usr/bin/websign"), false, &fake);

    assert_eq!(outcome, Outcome::Written);
    let text = fs::read_to_string(applications.join(file_name())).unwrap();
    assert!(text.contains("Exec=/usr/bin/websign %u\n"), "{text}");
    assert!(text.contains(&format!("MimeType=x-scheme-handler/{URL_SCHEME};\n")));
    assert_eq!(*calls.borrow(), [(file_name(), mime_type())]);
}

#[test]
fn a_missing_xdg_mime_is_a_skip_and_a_failing_one_a_failure() {
    let data = tempfile::tempdir().unwrap();
    let exe = Path::new("/usr/bin/websign");
    let missing = register(data.path(), exe, false, &|_, _| Tool::Missing);
    assert!(matches!(missing, Outcome::Skipped(_)), "{missing:?}");
    assert!(data.path().join(file_name()).exists());
    let failed = register(data.path(), exe, false, &|_, _| {
        Tool::Failed("exit 1".into())
    });
    assert!(matches!(failed, Outcome::Failed(_)), "{failed:?}");
}

#[test]
fn unregister_removes_the_entry_once() {
    let data = tempfile::tempdir().unwrap();
    register(
        data.path(),
        Path::new("/usr/bin/websign"),
        false,
        &|_, _| Tool::Done,
    );
    assert_eq!(unregister(data.path(), true), Outcome::DryRun);
    assert_eq!(unregister(data.path(), false), Outcome::Removed);
    assert_eq!(unregister(data.path(), false), Outcome::NotPresent);
}

#[test]
fn dry_run_touches_nothing() {
    let data = tempfile::tempdir().unwrap();
    let applications = data.path().join("applications");
    let outcome = register(
        &applications,
        Path::new("/usr/bin/websign"),
        true,
        &|_, _| panic!("xdg-mime must not run on a dry run"),
    );
    assert_eq!(outcome, Outcome::DryRun);
    assert!(!applications.exists());
}

#[test]
fn programs_with_reserved_characters_are_quoted_and_escaped() {
    assert_eq!(
        exec_argument("/opt/web sign/websign").unwrap(),
        r#""/opt/web sign/websign""#
    );
    assert_eq!(
        exec_argument("/home/u/100%$x").unwrap(),
        r#""/home/u/100%%\\$x""#
    );
    assert_eq!(exec_argument("/tmp/a\nb"), None);
}
