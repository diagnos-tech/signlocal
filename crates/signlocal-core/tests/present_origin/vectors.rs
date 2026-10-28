//! The `docs/ux.md` §16.2 vectors, IDN hosts and IP literals shown whole.

use super::*;

#[test]
fn splits_ordinary_hosts_at_the_registrable_domain() {
    check(&[
        row(
            "https://app.diagnos.health",
            "https://app.diagnos.health",
            "https://app.",
            "diagnos.health",
            None,
            None,
            true,
        ),
        row(
            "https://laudos.clinicasaolucas.med.br",
            "https://laudos.clinicasaolucas.med.br",
            "https://laudos.",
            "clinicasaolucas.med.br",
            None,
            None,
            true,
        ),
        row(
            "https://diagnos.health.cadastro-medico.com",
            "https://diagnos.health.cadastro-medico.com",
            "https://diagnos.health.",
            "cadastro-medico.com",
            None,
            None,
            true,
        ),
    ]);
}

#[test]
fn hides_the_default_port_and_shows_any_other() {
    check(&[
        row(
            "https://app.diagnos.health:443",
            "https://app.diagnos.health",
            "https://app.",
            "diagnos.health",
            None,
            None,
            true,
        ),
        row(
            "https://app.diagnos.health:8443",
            "https://app.diagnos.health:8443",
            "https://app.",
            "diagnos.health",
            Some(8443),
            None,
            true,
        ),
        // 80 is only the default for http.
        row(
            "https://app.diagnos.health:80",
            "https://app.diagnos.health:80",
            "https://app.",
            "diagnos.health",
            Some(80),
            None,
            true,
        ),
        row(
            "http://localhost:80",
            "http://localhost",
            "http://",
            "localhost",
            None,
            Some(Localhost),
            true,
        ),
        row(
            "http://localhost:443",
            "http://localhost:443",
            "http://",
            "localhost",
            Some(443),
            Some(Localhost),
            true,
        ),
    ]);
}

#[test]
fn lowercases_scheme_and_host() {
    check(&[row(
        "HTTPS://App.Example.COM",
        "https://app.example.com",
        "https://app.",
        "example.com",
        None,
        None,
        true,
    )]);
    let scheme_only = ok("Https://app.example.com");
    assert_eq!(scheme_only.canonical, "https://app.example.com");
}

#[test]
fn flags_a_punycode_host_and_shows_its_unicode_form() {
    let got = ok("https://xn--dignos-4nf.health");
    assert_eq!(got.canonical, "https://xn--dignos-4nf.health");
    assert_eq!(got.prefix, "https://");
    assert_eq!(got.registrable, "xn--dignos-4nf.health");
    assert_eq!(got.port, None);
    assert_eq!(got.warning, Some(Idn));
    assert_eq!(got.unicode.as_deref(), Some(LOOKALIKE_UNICODE));
    assert!(!got.can_remember);
}

#[test]
fn converts_a_unicode_host_to_punycode() {
    let got = ok(&format!("https://{LOOKALIKE_UNICODE}"));
    assert_eq!(got.canonical, "https://xn--dignos-4nf.health");
    assert_eq!(got.registrable, "xn--dignos-4nf.health");
    assert_eq!(got.warning, Some(Idn));
    assert_eq!(got.unicode.as_deref(), Some(LOOKALIKE_UNICODE));
    assert!(!got.can_remember);
}

#[test]
fn unicode_hosts_are_lowercased_before_conversion() {
    let got = ok("https://B\u{fc}CHER.example");
    assert_eq!(got.canonical, "https://xn--bcher-kva.example");
    assert_eq!(got.unicode.as_deref(), Some("b\u{fc}cher.example"));
    assert_eq!(got.warning, Some(Idn));
}

#[test]
fn splits_idn_subdomains_from_an_ascii_registrable_domain() {
    let got = ok("https://b\u{fc}cher.example.com");
    assert_eq!(got.canonical, "https://xn--bcher-kva.example.com");
    assert_eq!(got.prefix, "https://xn--bcher-kva.");
    assert_eq!(got.registrable, "example.com");
    assert_eq!(got.warning, Some(Idn));
    assert_eq!(got.unicode.as_deref(), Some("b\u{fc}cher.example.com"));
    assert!(!got.can_remember);
}

#[test]
fn plain_ascii_hosts_have_no_unicode_form() {
    assert_eq!(ok("https://app.diagnos.health").unicode, None);
    assert_eq!(ok("http://localhost:3000").unicode, None);
    assert_eq!(ok("https://203.0.113.7").unicode, None);
}

#[test]
fn ip_literals_are_shown_whole() {
    check(&[
        row(
            "https://192.168.0.20:8443",
            "https://192.168.0.20:8443",
            "https://",
            "192.168.0.20",
            Some(8443),
            Some(LocalIp),
            false,
        ),
        row(
            "https://203.0.113.7",
            "https://203.0.113.7",
            "https://",
            "203.0.113.7",
            None,
            Some(PublicIp),
            false,
        ),
    ]);
}

#[test]
fn localhost_and_loopback_may_be_remembered() {
    check(&[
        row(
            "http://localhost:5173",
            "http://localhost:5173",
            "http://",
            "localhost",
            Some(5173),
            Some(Localhost),
            true,
        ),
        row(
            "http://127.0.0.1:8000",
            "http://127.0.0.1:8000",
            "http://",
            "127.0.0.1",
            Some(8000),
            Some(Localhost),
            true,
        ),
        row(
            "http://[::1]:3000",
            "http://[::1]:3000",
            "http://",
            "[::1]",
            Some(3000),
            Some(Localhost),
            true,
        ),
        row(
            "https://localhost",
            "https://localhost",
            "https://",
            "localhost",
            None,
            Some(Localhost),
            true,
        ),
        row(
            "https://127.0.0.1",
            "https://127.0.0.1",
            "https://",
            "127.0.0.1",
            None,
            Some(Localhost),
            true,
        ),
        row(
            "https://[::1]",
            "https://[::1]",
            "https://",
            "[::1]",
            None,
            Some(Localhost),
            true,
        ),
    ]);
}
