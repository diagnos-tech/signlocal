//! Text that is not a serialized origin.

use super::*;

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
        "https://a.example\u{0}",
        "https://a.example:",
        "https://example.com.",
        "https://a..example",
        "https://.example",
        "https://a<b.example",
        "https://a^b.example",
        "https://a|b.example",
        "https://1.2.3",
        "https://127.1",
        "https://0x7f.0.0.1",
        "https://999.999.999.999",
        "https://a.example.123",
        "https://[fe80::1%25eth0]",
    ] {
        assert_eq!(
            format_origin(origin),
            Err(OriginError::Malformed),
            "{origin:?}"
        );
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

    // The length is checked before anything else.
    let long_insecure = format!("http://{}.example.com", "a".repeat(600));
    assert_eq!(format_origin(&long_insecure), Err(OriginError::Malformed));
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
        // Not a port in the URL grammar, though Rust's integer parser takes it.
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
