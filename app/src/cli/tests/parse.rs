//! Every command and flag of `desktop-api.md` §2 parses; mistakes are usage
//! errors (exit 2).

use clap::Parser;

use crate::cli::options::{AlgorithmArg, BrowserArg, HashArg};
use crate::cli::register::ScopeArg;
use crate::cli::{Cli, Command};

fn parse(args: &[&str]) -> Result<Option<Command>, clap::Error> {
    Cli::try_parse_from(std::iter::once("websign").chain(args.iter().copied()))
        .map(|cli| cli.command)
}

fn usage_error(args: &[&str]) {
    let error = parse(args).expect_err(&format!("{args:?} must be refused"));
    assert_eq!(error.exit_code(), 2, "{args:?}: {error}");
}

#[test]
fn no_arguments_opens_diagnostics() {
    assert!(parse(&[]).unwrap().is_none());
}

#[test]
fn install_and_uninstall_flags() {
    let Some(Command::Install(args)) = parse(&[
        "install",
        "--browser",
        "chrome",
        "--browser",
        "firefox",
        "--system",
        "--dry-run",
        "--json",
    ])
    .unwrap() else {
        panic!("install");
    };
    assert_eq!(args.browsers, [BrowserArg::Chrome, BrowserArg::Firefox]);
    assert!(args.system && args.dry_run && args.json);
    let Some(Command::Uninstall(args)) =
        parse(&["uninstall", "--purge", "--system", "--dry-run", "--json"]).unwrap()
    else {
        panic!("uninstall");
    };
    assert!(args.purge && args.system && args.dry_run && args.json);
    usage_error(&["install", "--browser", "netscape"]);
    usage_error(&["uninstall", "--browser", "chrome"]);
}

#[test]
fn register_flags() {
    let Some(Command::Register(args)) = parse(&[
        "register",
        "--browser",
        "all",
        "--uninstall",
        "--scope",
        "system",
        "--user-data-dir",
        "/tmp/p",
        "--extension-id",
        "abcdefghijklmnopabcdefghijklmnop",
        "--manifest-dir",
        "/tmp/m",
        "--dry-run",
        "--json",
    ])
    .unwrap() else {
        panic!("register");
    };
    assert_eq!(args.browsers, [BrowserArg::All]);
    assert_eq!(args.scope, ScopeArg::System);
    assert!(args.uninstall && args.dry_run && args.json);
    assert_eq!(args.extension_ids.len(), 1);
    let Some(Command::Register(args)) = parse(&["register"]).unwrap() else {
        panic!("register");
    };
    assert_eq!(args.scope, ScopeArg::User);
}

#[test]
fn sign_takes_a_hash_and_exactly_one_digest_source() {
    let Some(Command::Sign(args)) = parse(&[
        "sign",
        "--hash",
        "SHA-384",
        "--digest",
        "00",
        "--algorithm",
        "RSASSA-PSS",
        "--algorithm",
        "ecdsa",
        "--certificate",
        "ab",
    ])
    .unwrap() else {
        panic!("sign");
    };
    assert_eq!(args.hash, HashArg::Sha384);
    assert_eq!(args.algorithms, [AlgorithmArg::RsaPss, AlgorithmArg::Ecdsa]);
    assert!(matches!(
        parse(&["sign", "--hash", "sha256", "--digest-file", "-"]),
        Ok(Some(Command::Sign(_)))
    ));
    usage_error(&["sign", "--hash", "SHA-256"]);
    usage_error(&["sign", "--digest", "00"]);
    usage_error(&["sign", "--hash", "MD5", "--digest", "00"]);
    usage_error(&[
        "sign",
        "--hash",
        "SHA-256",
        "--digest",
        "00",
        "--digest-file",
        "x",
    ]);
    usage_error(&[
        "sign",
        "--hash",
        "SHA-256",
        "--digest",
        "00",
        "--algorithm",
        "EdDSA",
    ]);
}

#[test]
fn choose_connect_doctor_diagnostics_and_version() {
    assert!(matches!(
        parse(&["choose", "--algorithm", "pkcs1"]),
        Ok(Some(Command::Choose(_)))
    ));
    assert!(matches!(parse(&["connect"]), Ok(Some(Command::Connect(_)))));
    assert!(matches!(parse(&["doctor", "--json"]), Ok(Some(Command::Doctor(a))) if a.json));
    assert!(matches!(
        parse(&["diagnostics", "--tab", "devices"]),
        Ok(Some(Command::Diagnostics(_)))
    ));
    assert!(matches!(parse(&["version", "--json"]), Ok(Some(Command::Version(a))) if a.json));
    usage_error(&["connect", "--port", "1"]);
    usage_error(&["diagnostics", "--tab", "sites"]);
    usage_error(&["daemon"]);
}

#[test]
fn help_and_version_flags_are_not_errors() {
    for flag in ["--help", "--version"] {
        let error = parse(&[flag]).unwrap_err();
        assert_eq!(error.exit_code(), 0, "{flag}");
    }
}

#[test]
fn help_of_the_machine_commands_shows_examples_and_output() {
    for (command, needles) in [
        (
            "sign",
            &["Examples:", "--digest-file -", "sign.result", "exit code 3"][..],
        ),
        ("choose", &["Examples:", "choose.result"]),
        ("connect", &["little-endian", "hello", "stdin"]),
    ] {
        let help = parse(&[command, "--help"]).unwrap_err().to_string();
        for needle in needles {
            assert!(
                help.contains(needle),
                "{command}: {needle:?} missing from\n{help}"
            );
        }
    }
    let main = parse(&["--help"]).unwrap_err().to_string();
    assert!(main.contains("Exit codes"), "{main}");
}
