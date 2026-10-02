use crate::present::origin::{FormattedOrigin, OriginError, OriginWarning, format_origin};

fn ok(origin: &str) -> FormattedOrigin {
    format_origin(origin).unwrap_or_else(|e| panic!("{origin}: {e}"))
}

#[test]
fn splits_the_registrable_domain() {
    let cases = [
        (
            "https://app.diagnos.health",
            "https://app.",
            "diagnos.health",
        ),
        (
            "https://app.diagnos.health:443",
            "https://app.",
            "diagnos.health",
        ),
        (
            "https://laudos.clinicasaolucas.med.br",
            "https://laudos.",
            "clinicasaolucas.med.br",
        ),
        (
            "https://diagnos.health.cadastro-medico.com",
            "https://diagnos.health.",
            "cadastro-medico.com",
        ),
        ("HTTPS://App.Example.COM", "https://app.", "example.com"),
        ("https://user.github.io", "https://", "user.github.io"),
        ("https://com", "https://", "com"),
    ];
    for (input, prefix, registrable) in cases {
        let origin = ok(input);
        assert_eq!(origin.prefix, prefix, "{input}");
        assert_eq!(origin.registrable, registrable, "{input}");
        assert_eq!(origin.warning, None, "{input}");
        assert!(origin.can_remember, "{input}");
        assert_eq!(origin.port, None, "{input}");
    }
    assert_eq!(
        ok("https://app.diagnos.health:443").canonical,
        "https://app.diagnos.health"
    );
    assert_eq!(
        ok("HTTPS://App.Example.COM").canonical,
        "https://app.example.com"
    );
    assert_eq!(ok("https://a.example/").canonical, "https://a.example");
}

#[test]
fn idn_hosts_show_punycode_and_unicode_and_cannot_be_remembered() {
    let origin = ok("https://xn--dignos-4nf.health");
    assert_eq!(origin.registrable, "xn--dignos-4nf.health");
    assert_eq!(origin.warning, Some(OriginWarning::Idn));
    assert!(!origin.can_remember);
    assert_eq!(origin.unicode.as_deref(), Some("di\u{430}gnos.health"));
    let typed = ok("https://di\u{430}gnos.health");
    assert_eq!(typed.canonical, "https://xn--dignos-4nf.health");
}

#[test]
fn ip_literals_are_classified() {
    let local = ok("https://192.168.0.20:8443");
    assert_eq!(local.registrable, "192.168.0.20");
    assert_eq!(local.port, Some(8443));
    assert_eq!(local.warning, Some(OriginWarning::LocalIp));
    assert!(!local.can_remember);
    let cases = [
        ("https://203.0.113.7", OriginWarning::PublicIp),
        ("https://10.1.2.3", OriginWarning::LocalIp),
        ("https://172.16.0.1", OriginWarning::LocalIp),
        ("https://172.32.0.1", OriginWarning::PublicIp),
        ("https://169.254.1.1", OriginWarning::LocalIp),
        ("https://[fe80::1]", OriginWarning::LocalIp),
        ("https://[fd00::1]", OriginWarning::LocalIp),
        ("https://[2001:db8::1]", OriginWarning::PublicIp),
    ];
    for (input, warning) in cases {
        assert_eq!(ok(input).warning, Some(warning), "{input}");
    }
}

#[test]
fn loopback_may_use_http_and_be_remembered() {
    let cases = [
        ("http://localhost:5173", "localhost", Some(5173)),
        ("http://127.0.0.1:8000", "127.0.0.1", Some(8000)),
        ("http://[::1]:3000", "[::1]", Some(3000)),
        ("http://app.localhost", "app.localhost", None),
    ];
    for (input, registrable, port) in cases {
        let origin = ok(input);
        assert_eq!(origin.prefix, "http://", "{input}");
        assert_eq!(origin.registrable, registrable, "{input}");
        assert_eq!(origin.port, port, "{input}");
        assert_eq!(origin.warning, Some(OriginWarning::Localhost), "{input}");
        assert!(origin.can_remember, "{input}");
    }
    assert_eq!(ok("http://localhost:80").port, None);
}

#[test]
fn refuses_insecure_origins() {
    let insecure = [
        "http://laudos.exemplo.com",
        "ftp://a.example",
        "file:///tmp",
        "chrome-extension://abc",
        "http://203.0.113.7",
    ];
    for input in insecure {
        assert_eq!(format_origin(input), Err(OriginError::Insecure), "{input}");
    }
}

#[test]
fn refuses_malformed_origins() {
    let malformed = [
        "",
        "null",
        "data:text/html,x",
        "https://",
        "https://a.example/path",
        "https://a.example//",
        "https://a.example?x=1",
        "https://a.example#x",
        "https://user@a.example",
        "https://a.example:0",
        "https://a.example:65536",
        "https://a.example:",
        "https://a.example:80:80",
        "https://[::1",
        "https://[zz]",
        "https://a..example",
        "https://.example",
        "https://a b.example",
        "https://999.1.1.1",
        "https://1.2.3",
    ];
    for input in malformed {
        assert_eq!(
            format_origin(input),
            Err(OriginError::Malformed),
            "{input:?}"
        );
    }
    let long = format!("https://{}.example", "a".repeat(600));
    assert_eq!(format_origin(&long), Err(OriginError::Malformed));
}
