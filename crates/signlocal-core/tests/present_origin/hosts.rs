//! IP ranges and Public Suffix List splits.

use super::*;

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
        ("[::ffff:192.168.0.1]", LocalIp),
        ("[::ffff:203.0.113.7]", PublicIp),
        ("[::ffff:127.0.0.1]", Localhost),
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
    // The Public Suffix List's private section is used too, so a tenant of a
    // hosting platform is shown as its own registrable domain.
    let got = ok("https://app.tenant.github.io");
    assert_eq!(got.registrable, "tenant.github.io");
    assert_eq!(got.prefix, "https://app.");
    let platform = ok("https://github.io");
    assert_eq!(platform.registrable, "github.io");
    assert_eq!(platform.prefix, "https://");
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
