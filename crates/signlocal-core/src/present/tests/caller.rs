use std::path::PathBuf;

use crate::present::caller::{
    CallerLabel, CodeSigner, DesktopCaller, caller_label, consent_key, runs_scripts,
};

fn caller(path: &str, product: Option<&str>, signer: Option<CodeSigner>) -> DesktopCaller {
    DesktopCaller {
        executable: PathBuf::from(path),
        product_name: product.map(str::to_owned),
        signer,
    }
}

fn authenticode() -> CodeSigner {
    CodeSigner::Authenticode {
        subject: "Contoso Ltd".into(),
    }
}

fn apple() -> CodeSigner {
    CodeSigner::Apple {
        team_id: "ABCDE12345".into(),
        identifier: "com.contoso.app".into(),
    }
}

#[test]
fn signed_callers_show_their_signer() {
    let label = caller_label(&caller(
        "/apps/Tool.EXE",
        Some(" Tool "),
        Some(authenticode()),
    ));
    assert_eq!(
        label,
        CallerLabel {
            name: "Tool".into(),
            detail: "Contoso Ltd".into(),
            verified: true,
            runs_scripts: false,
        }
    );
    let label = caller_label(&caller("/apps/tool", Some("Tool"), Some(apple())));
    assert_eq!(label.detail, "com.contoso.app (ABCDE12345)");
    assert!(label.verified);
}

#[test]
fn unsigned_callers_show_their_path_and_are_unverified() {
    let label = caller_label(&caller("/home/a/bin/tool", Some("  "), None));
    assert_eq!(
        label,
        CallerLabel {
            name: "tool".into(),
            detail: "/home/a/bin/tool".into(),
            verified: false,
            runs_scripts: false,
        }
    );
}

#[test]
fn long_product_names_are_shortened_to_64_characters() {
    let long = "é".repeat(100);
    let label = caller_label(&caller("/a/b", Some(&long), None));
    assert_eq!(label.name.chars().count(), 64);
    assert!(label.name.ends_with('…'));
    let exact = "x".repeat(64);
    assert_eq!(
        caller_label(&caller("/a/b", Some(&exact), None)).name,
        exact
    );
}

#[test]
fn consent_keys_follow_the_signer() {
    assert_eq!(
        consent_key(&caller("C:/Apps/Tool.EXE", None, Some(authenticode()))),
        "app:authenticode:Contoso Ltd:tool.exe"
    );
    assert_eq!(
        consent_key(&caller("/apps/tool", None, Some(apple()))),
        "app:apple:ABCDE12345:com.contoso.app"
    );
    assert_eq!(
        consent_key(&caller("/home/a/tool", None, None)),
        "path:/home/a/tool"
    );
}

#[test]
fn interpreters_and_shells_run_scripts_and_say_so() {
    let node = caller("/usr/bin/node", None, None);
    assert!(runs_scripts(&node));
    assert!(caller_label(&node).runs_scripts);
    let pwsh = caller(
        "C:/Program Files/PowerShell/7/pwsh.exe",
        Some("PowerShell 7"),
        Some(authenticode()),
    );
    assert!(caller_label(&pwsh).runs_scripts);
    assert!(!caller_label(&caller("/opt/acme/invoicer", None, None)).runs_scripts);
}
