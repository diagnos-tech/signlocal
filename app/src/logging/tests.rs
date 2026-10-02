//! The log privacy audit: personal-looking data goes through the real `log`
//! macros and the real line formatter, and must not come out.

use std::sync::{Mutex, Once};

use super::*;

const HOME: &str = "/home/ana.souza";
const LOGIN: &str = "ana.souza";

static LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Captures formatted lines in memory instead of a file.
struct Capture(Redactor);

impl Log for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn log(&self, record: &Record<'_>) {
        LINES.lock().unwrap().push(format_line(&self.0, record));
    }
    fn flush(&self) {}
}

/// Logs through the macro and returns the line carrying `marker`.
fn logged(marker: &str, emit: impl FnOnce()) -> String {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        log::set_boxed_logger(Box::new(Capture(Redactor::new(Some(HOME), Some(LOGIN))))).unwrap();
        log::set_max_level(LevelFilter::Trace);
    });
    emit();
    let lines = LINES.lock().unwrap();
    lines
        .iter()
        .rev()
        .find(|l| l.contains(marker))
        .cloned()
        .unwrap()
}

fn assert_absent(line: &str, secrets: &[&str]) {
    for secret in secrets {
        assert!(!line.contains(secret), "{secret:?} leaked into {line:?}");
    }
}

#[test]
fn names_and_document_numbers_are_redacted() {
    let cn = "ANA BEATRIZ SOUZA:12345678909";
    let line = logged("m1", || {
        log::info!("m1 subject CN={cn},OU=AC SOLUTI,O=ICP-Brasil holder: Ana Beatriz Souza")
    });
    assert_absent(
        &line,
        &["ANA BEATRIZ", "12345678909", "SOLUTI", "Ana Beatriz"],
    );
    let line = logged("m2", || {
        log::warn!("m2 cpf 123.456.789-09 cnpj 12.345.678/0001-95")
    });
    assert_absent(&line, &["123.456.789-09", "12.345.678/0001-95", "0001"]);
}

#[test]
fn digests_serials_fingerprints_and_certificates_are_redacted() {
    let digest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let fp = "AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89";
    let b64 = "MIIDdzCCAl+gAwIBAgIEAgAAuTANBgkqhkiG9w0BAQUFADBaMQswCQYDVQQGEwJJ";
    let line = logged("m3", || {
        log::debug!("m3 digest {digest} fp {fp} serial=0A1B2C3D4E5F6071 der {b64}")
    });
    assert_absent(&line, &[digest, fp, "0A1B2C3D4E5F6071", b64]);
    let pem = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----";
    let line = logged("m4", || log::error!("m4 got {pem} end"));
    assert_absent(&line, &["MIIB", "BEGIN"]);
    assert!(line.contains("[certificate] end"), "{line}");
}

#[test]
fn pins_emails_and_sites_are_redacted() {
    let line = logged("m5", || {
        log::info!(
            "m5 pin=246810 PIN: 1234 password = \"hunter 2\" from ana@example.com.br at https://bank.example/pay?x=1"
        )
    });
    assert_absent(&line, &["246810", "hunter", "ana@", "bank.example"]);
    assert!(
        line.contains("PIN: [redacted]") && line.contains("pin=[redacted]"),
        "{line}"
    );
}

#[test]
fn home_folder_and_login_are_redacted_from_paths() {
    let line = logged("m6", || {
        log::info!(
            "m6 module {HOME}/lib/libaetpkss.so and /mnt/{LOGIN}/x and C:\\Users\\Ana.Souza\\x"
        )
    });
    assert!(line.contains("~/lib/libaetpkss.so"), "{line}");
    assert_absent(&line, &["ana.souza", "Ana.Souza"]);
}

#[test]
fn ip_addresses_are_redacted() {
    let line = logged("m8", || {
        log::info!(
            "m8 peer 10.0.0.1 proxy 192.168.100.200:8080 v6 fe80::1ff:fe23:4567:890a%en0 [2001:db8::7]:443 lo ::1"
        )
    });
    assert_absent(
        &line,
        &["10.0.0.1", "192.168.100.200", "fe80", "2001:db8", "::1"],
    );
    assert!(
        line.contains("[ip]:8080") && line.contains("[[ip]]:443"),
        "{line}"
    );
}

#[test]
fn e_mails_with_tags_and_subdomains_are_redacted() {
    let line = logged("m9", || {
        log::info!("m9 contact joao.silva+certs@mail.example.gov.br, JOAO_S@X.ORG.")
    });
    assert_absent(
        &line,
        &["joao.silva", "certs@", "example.gov", "JOAO_S", "X.ORG"],
    );
}

#[test]
fn other_accounts_profiles_and_share_servers_are_redacted() {
    let line = logged("m10", || {
        log::info!(
            "m10 C:\\Users\\Bruno Lima\\AppData\\x.dll D:/Users/carla/x /Users/davi/Library/x /home/elisa/.pki \\\\PC-FERNANDA\\certs\\a.pfx C:\\Users\\gabriel failed"
        )
    });
    assert_absent(
        &line,
        &[
            "Bruno", "Lima", "carla", "davi", "elisa", "FERNANDA", "gabriel",
        ],
    );
    assert!(
        line.contains("\\AppData\\x.dll") && line.contains("] failed"),
        "{line}"
    );
    let escaped = logged("m11", || {
        log::info!("m11 {:?}", std::path::Path::new("C:\\Users\\helena\\x"))
    });
    assert_absent(&escaped, &["helena"]);
}

#[test]
fn a_pin_is_never_in_a_line_whatever_its_label() {
    for (marker, text) in [
        ("p1", "p1 PIN=1234"),
        ("p2", "p2 pin: 12345678"),
        ("p3", "p3 new pin = \"4321 9\""),
        ("p4", "p4 puk=87654321"),
    ] {
        let line = logged(marker, || log::info!("{text}"));
        assert_absent(&line, &["1234", "12345678", "4321", "87654321"]);
    }
}

#[test]
fn what_logs_are_for_stays_readable() {
    let line = logged("m7", || {
        log::warn!(
            "m7 C_Sign failed: CKR_PIN_INCORRECT (0x000000A0) after 1500 ms, 3 keys, frame of 1048576 bytes, v1.4.0 on 2026-09-30 at 12:30:45 in websign_host::engine from /usr/lib/x86_64-linux-gnu/opensc-pkcs11.so and C:\\Users\\Public\\x.dll"
        )
    });
    assert!(
        line.contains("C_Sign failed: CKR_PIN_INCORRECT (0x000000A0) after 1500 ms, 3 keys, frame of 1048576 bytes, v1.4.0 on 2026-09-30 at 12:30:45 in websign_host::engine from /usr/lib/x86_64-linux-gnu/opensc-pkcs11.so and C:\\Users\\Public\\x.dll"),
        "{line}"
    );
}

#[test]
fn the_level_comes_from_the_environment_value() {
    assert_eq!(level_from(None), LevelFilter::Info);
    assert_eq!(level_from(Some("trace")), LevelFilter::Trace);
    assert_eq!(level_from(Some(" off ")), LevelFilter::Off);
    assert_eq!(level_from(Some("loud")), LevelFilter::Info);
}
