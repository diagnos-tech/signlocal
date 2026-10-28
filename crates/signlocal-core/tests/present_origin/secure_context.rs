//! Plain `http` only for localhost; other schemes.

use super::*;

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
fn another_scheme_with_slashes_is_insecure_and_without_them_malformed() {
    for origin in ["file://", "file:///etc/passwd", "chrome-extension://abc"] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Insecure),
            "{origin}"
        );
    }
    for origin in [
        "data:",
        "data:text/html,hello",
        "javascript:alert(1)",
        "about:blank",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Malformed),
            "{origin}"
        );
    }
}
