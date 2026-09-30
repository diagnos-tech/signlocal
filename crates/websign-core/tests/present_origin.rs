//! SPEC §9 (`docs/ux.md` §4.3, vectors §16.2): `present::origin::format_origin`.

use websign_core::present::origin::{FormattedOrigin, OriginError, OriginWarning, format_origin};

use OriginWarning::{Idn, LocalIp, Localhost, PublicIp};

/// Cyrillic "а" (U+0430) in place of the Latin one: the classic look-alike.
const LOOKALIKE_UNICODE: &str = "di\u{430}gnos.health";

struct Row {
    input: &'static str,
    canonical: &'static str,
    prefix: &'static str,
    registrable: &'static str,
    port: Option<u16>,
    warning: Option<OriginWarning>,
    can_remember: bool,
}

const fn row(
    input: &'static str,
    canonical: &'static str,
    prefix: &'static str,
    registrable: &'static str,
    port: Option<u16>,
    warning: Option<OriginWarning>,
    can_remember: bool,
) -> Row {
    Row {
        input,
        canonical,
        prefix,
        registrable,
        port,
        warning,
        can_remember,
    }
}

fn ok(input: &str) -> FormattedOrigin {
    format_origin(input).unwrap_or_else(|e| panic!("{input:?} must be accepted, got {e:?}"))
}

fn check(rows: &[Row]) {
    for r in rows {
        let got = ok(r.input);
        assert_eq!(got.canonical, r.canonical, "{} canonical", r.input);
        assert_eq!(got.prefix, r.prefix, "{} prefix", r.input);
        assert_eq!(got.registrable, r.registrable, "{} registrable", r.input);
        assert_eq!(got.port, r.port, "{} port", r.input);
        assert_eq!(got.warning, r.warning, "{} warning", r.input);
        assert_eq!(got.can_remember, r.can_remember, "{} can_remember", r.input);
    }
}

// --- ux.md §16.2 vectors -----------------------------------------------------------------------

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

// --- secure context ----------------------------------------------------------------------------

#[test]
fn refuses_plain_http_outside_localhost() {
    for origin in [
        "http://laudos.exemplo.com",
        "http://example.com:8080",
        "http://192.168.0.20",
        "http://203.0.113.7",
        "http://10.0.0.1:3000",
        "http://[2001:db8::1]",
        "http://[fe80::1]",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Insecure),
            "{origin}"
        );
    }
}

#[test]
fn does_not_mistake_a_look_alike_for_localhost() {
    for origin in [
        "http://localhost.evil.example",
        "http://127.0.0.1.evil.example",
        "http://notlocalhost",
        "http://xlocalhost:3000",
        "http://localhost.example.com",
        "http://128.0.0.1",
        "http://126.255.255.255",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Insecure),
            "{origin}"
        );
    }
}

#[test]
fn accepts_http_for_every_localhost_form() {
    for origin in [
        "http://localhost",
        "http://LOCALHOST:8080",
        "http://app.localhost",
        "http://a.b.localhost:3000",
        "http://127.0.0.1",
        "http://127.0.0.2:9000",
        "http://127.255.255.254",
        "http://[::1]",
    ] {
        let got = ok(origin);
        assert_eq!(got.warning, Some(Localhost), "{origin}");
        assert!(got.can_remember, "{origin}");
    }
}

#[test]
fn a_localhost_subdomain_is_shown_whole() {
    let got = ok("http://app.localhost:3000");
    assert_eq!(got.canonical, "http://app.localhost:3000");
    assert_eq!(got.prefix, "http://");
    assert_eq!(got.registrable, "app.localhost");
    assert_eq!(got.port, Some(3000));
}

#[test]
fn refuses_other_schemes() {
    for origin in [
        "ftp://example.com",
        "ws://example.com",
        "wss://example.com",
        "chrome-extension://abcdefghijklmnop",
        "moz-extension://abc",
        "about://blank",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Insecure),
            "{origin}"
        );
    }
}

#[test]
fn refuses_hostless_origins_as_insecure_or_malformed() {
    // SPEC §9 table: `Err(Insecure)` or `Err(Malformed)` (no host).
    for origin in [
        "file://",
        "file:///etc/passwd",
        "data:",
        "data:text/html,hello",
        "chrome-extension://abc",
        "javascript:alert(1)",
        "about:blank",
    ] {
        assert!(
            matches!(
                format_origin(origin),
                Err(OriginError::Insecure | OriginError::Malformed)
            ),
            "{origin}"
        );
    }
}

// --- malformed ---------------------------------------------------------------------------------

#[test]
fn refuses_paths_queries_fragments_and_credentials() {
    for origin in [
        "https://a.example/path",
        "https://a.example/index.html",
        "https://a.example//",
        "https://a.example?x=1",
        "https://a.example/?x=1",
        "https://a.example#frag",
        "https://user@a.example",
        "https://user:secret@a.example",
        "https://@a.example",
        "http://localhost:3000/app",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Malformed),
            "{origin}"
        );
    }
}

#[test]
fn refuses_things_that_are_not_origins() {
    for origin in [
        "",
        "null",
        "example.com",
        "//example.com",
        "https:",
        "https:/",
        "https:/example.com",
        "https://",
        "https:///",
        "https://:443",
        "https:// a.example",
        "https://a .example",
        "https://a.example ",
        " https://a.example",
        "https://a.example\n",
        "https://a.example\t",
    ] {
        assert!(format_origin(origin).is_err(), "{origin:?}");
    }
}

#[test]
fn an_empty_host_is_malformed() {
    for origin in ["https://", "https://:443", "http://:80"] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Malformed),
            "{origin}"
        );
    }
    assert_eq!(format_origin("null"), Err(OriginError::Malformed));
    assert_eq!(format_origin(""), Err(OriginError::Malformed));
}

#[test]
fn refuses_origins_longer_than_512_characters() {
    let long = format!("https://{}.example.com", "a".repeat(600));
    assert!(long.len() > 512);
    assert_eq!(format_origin(&long), Err(OriginError::Malformed));

    let many_labels = format!("https://{}example.com", "ab.".repeat(200));
    assert!(many_labels.len() > 512);
    assert_eq!(format_origin(&many_labels), Err(OriginError::Malformed));

    let long_port_free_insecure = format!("http://{}.example.com", "a".repeat(600));
    assert!(matches!(
        format_origin(&long_port_free_insecure),
        Err(OriginError::Malformed | OriginError::Insecure)
    ));
}

#[test]
fn accepts_a_trailing_slash_alone_and_ignores_it() {
    assert_eq!(ok("https://a.example/"), ok("https://a.example"));
    assert_eq!(ok("https://a.example/").canonical, "https://a.example");
    assert_eq!(ok("http://localhost:3000/"), ok("http://localhost:3000"));
}

#[test]
fn checks_the_port_range() {
    assert_eq!(ok("https://a.example:1").port, Some(1));
    assert_eq!(ok("https://a.example:65535").port, Some(65535));
    for origin in [
        "https://a.example:0",
        "https://a.example:65536",
        "https://a.example:99999",
        "https://a.example:100000000000",
        "https://a.example:-1",
        "https://a.example:abc",
        "https://a.example:44 3",
        "https://a.example:4x",
        // SPEC: `+443` is not a port; Rust's integer parser would accept it.
        "https://a.example:+443",
        "https://a.example:443:443",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Malformed),
            "{origin}"
        );
    }
}

// --- private, link-local and public IP literals --------------------------------------------------

#[test]
fn classifies_ipv4_ranges() {
    let cases = [
        ("10.0.0.1", LocalIp),
        ("10.255.255.255", LocalIp),
        ("172.16.0.1", LocalIp),
        ("172.31.255.255", LocalIp),
        ("192.168.0.1", LocalIp),
        ("192.168.255.255", LocalIp),
        ("169.254.0.1", LocalIp),
        ("169.254.169.254", LocalIp),
        ("172.15.255.255", PublicIp),
        ("172.32.0.1", PublicIp),
        ("192.167.0.1", PublicIp),
        ("192.169.0.1", PublicIp),
        ("169.253.0.1", PublicIp),
        ("11.0.0.1", PublicIp),
        ("8.8.8.8", PublicIp),
        ("203.0.113.7", PublicIp),
    ];
    for (host, warning) in cases {
        let got = ok(&format!("https://{host}"));
        assert_eq!(got.warning, Some(warning), "{host}");
        assert!(!got.can_remember, "{host}");
        assert_eq!(got.registrable, host, "{host}");
        assert_eq!(got.prefix, "https://", "{host}");
        assert_eq!(got.canonical, format!("https://{host}"), "{host}");
    }
}

#[test]
fn classifies_ipv6_literals_and_keeps_the_brackets() {
    let cases = [
        ("[fe80::1]", LocalIp),
        ("[febf::1]", LocalIp),
        ("[fc00::1]", LocalIp),
        ("[fd12:3456:789a::1]", LocalIp),
        ("[2001:db8::1]", PublicIp),
        ("[2606:4700::1111]", PublicIp),
        ("[fec0::1]", PublicIp),
        ("[fbff::1]", PublicIp),
        ("[::1]", Localhost),
    ];
    for (host, warning) in cases {
        let got = ok(&format!("https://{host}:8443"));
        assert_eq!(got.warning, Some(warning), "{host}");
        assert_eq!(got.registrable, host, "{host}");
        assert_eq!(got.prefix, "https://", "{host}");
        assert_eq!(got.canonical, format!("https://{host}:8443"), "{host}");
        assert_eq!(got.port, Some(8443), "{host}");
        assert_eq!(got.can_remember, warning == Localhost, "{host}");
    }
}

#[test]
fn ipv6_hosts_are_lowercased() {
    let got = ok("https://[2001:DB8::A]");
    assert_eq!(got.canonical, "https://[2001:db8::a]");
    assert_eq!(got.registrable, "[2001:db8::a]");
}

// --- public suffixes ---------------------------------------------------------------------------

#[test]
fn a_host_that_is_a_public_suffix_is_shown_whole() {
    for host in ["co.uk", "com.br", "github.io"] {
        let got = ok(&format!("https://{host}"));
        assert_eq!(got.registrable, host, "{host}");
        assert_eq!(got.prefix, "https://", "{host}");
        assert_eq!(got.warning, None, "{host}");
    }
}

#[test]
fn multi_label_suffixes_keep_one_more_label() {
    let got = ok("https://a.b.c.example.co.uk");
    assert_eq!(got.registrable, "example.co.uk");
    assert_eq!(got.prefix, "https://a.b.c.");
    let got = ok("https://www.exemplo.com.br:8443");
    assert_eq!(got.registrable, "exemplo.com.br");
    assert_eq!(got.prefix, "https://www.");
    assert_eq!(got.port, Some(8443));
}

#[test]
fn private_suffixes_count_as_public_suffixes() {
    // SPEC: the Public Suffix List's private section is used too, so a tenant
    // of a hosting platform is shown as its own registrable domain.
    let got = ok("https://app.tenant.github.io");
    assert_eq!(got.registrable, "tenant.github.io");
    assert_eq!(got.prefix, "https://app.");
}

#[test]
fn a_bare_two_label_domain_has_no_prefix_but_the_scheme() {
    let got = ok("https://diagnos.health");
    assert_eq!(got.prefix, "https://");
    assert_eq!(got.registrable, "diagnos.health");
}

#[test]
fn prefix_registrable_and_port_reassemble_the_canonical_origin() {
    for origin in [
        "https://app.diagnos.health",
        "https://app.diagnos.health:9443",
        "https://laudos.clinicasaolucas.med.br",
        "http://localhost:5173",
        "https://192.168.0.20:8443",
        "https://[2001:db8::1]:444",
        "https://b\u{fc}cher.example.com",
    ] {
        let got = ok(origin);
        let port = got.port.map(|p| format!(":{p}")).unwrap_or_default();
        assert_eq!(
            format!("{}{}{}", got.prefix, got.registrable, port),
            got.canonical,
            "{origin}"
        );
    }
}

#[test]
fn canonical_form_is_a_fixed_point() {
    for origin in [
        "HTTPS://App.Example.COM:443/",
        "https://b\u{fc}cher.example",
        "http://LOCALHOST:80",
        "https://[2001:DB8::1]",
    ] {
        let once = ok(origin);
        let twice = ok(&once.canonical);
        assert_eq!(once.canonical, twice.canonical, "{origin}");
        assert_eq!(once.prefix, twice.prefix, "{origin}");
        assert_eq!(once.registrable, twice.registrable, "{origin}");
        assert_eq!(once.port, twice.port, "{origin}");
    }
}

// --- warning precedence and robustness -----------------------------------------------------------

#[test]
fn an_idn_warning_wins_over_the_localhost_one() {
    // SPEC: `warning` is the first that applies: IDN, then localhost.
    let got = ok("http://b\u{fc}cher.localhost:3000");
    assert_eq!(got.warning, Some(Idn));
    assert!(!got.can_remember);
    assert_eq!(got.registrable, "xn--bcher-kva.localhost");
}

#[test]
fn never_panics_on_odd_input() {
    let mut inputs: Vec<String> = vec![
        "https://[".into(),
        "https://[]".into(),
        "https://[::1".into(),
        "https://[zz]".into(),
        "https://[1:2:3:4:5:6:7:8:9]".into(),
        "https://999.999.999.999".into(),
        "https://1.2.3".into(),
        "https://a..example".into(),
        "https://.example".into(),
        "https://example.".into(),
        "https://-a.example".into(),
        "https://xn--.example".into(),
        "https://xn--zzzzzzzzzzzzzzzzzzzz.example".into(),
        "https://\u{0}".into(),
        "https://a\u{0}b.example".into(),
        "https://\u{202e}moc.elpmaxe".into(),
        "https://\u{fe0f}.example".into(),
        "\u{1f4a9}".into(),
        "https://%41.example".into(),
        "https://a.example:".into(),
        ":".into(),
        "/".into(),
        "//".into(),
    ];
    inputs.push("https://".to_owned() + &"\u{fc}".repeat(100));
    inputs.push("h".repeat(1000));
    for input in inputs {
        let _ = format_origin(&input);
    }
}
