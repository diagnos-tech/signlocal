//! Warning precedence and inputs that must never panic.

use super::*;

#[test]
fn an_idn_warning_wins_over_the_localhost_one() {
    // `warning` is the first that applies: IDN, then localhost.
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

#[test]
fn random_origin_like_text_is_an_error_or_a_value_never_a_panic() {
    const PIECES: [&str; 24] = [
        "https://",
        "http://",
        "HTTP://",
        "a",
        "Z",
        "0",
        "9",
        ".",
        ":",
        "/",
        "[",
        "]",
        "::1",
        "xn--",
        "-",
        "@",
        "%",
        "\u{fc}",
        "\u{430}",
        "\u{202e}",
        "\u{0}",
        " ",
        "localhost",
        "255",
    ];
    let mut rng = Rng::new(0x0816_1A55);
    for _ in 0..20_000 {
        let count = rng.below(12) as usize;
        let text: String = (0..count)
            .map(|_| PIECES[rng.below(PIECES.len() as u64) as usize])
            .collect();
        if let Ok(got) = format_origin(&text) {
            // Whatever is accepted is already canonical.
            assert_eq!(ok(&got.canonical), got, "{text:?}");
        }
    }
}
