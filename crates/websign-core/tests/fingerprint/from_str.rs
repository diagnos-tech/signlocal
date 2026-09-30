//! FromStr.

use super::*;

#[test]
fn parses_plain_lowercase_and_uppercase_hex() {
    let expected = from_hex(ABC_HEX);
    assert_eq!(ABC_HEX.parse(), Ok(expected));
    assert_eq!(ABC_HEX.to_uppercase().parse(), Ok(expected));
    let mixed: String = ABC_HEX
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i % 2 == 0 {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect();
    assert_eq!(mixed.parse(), Ok(expected));
}

#[test]
fn parses_its_own_display_output() {
    for start in [0u8, 1, 0x80, 0xFF] {
        let fp = Fingerprint::from_bytes(bytes_32(start));
        assert_eq!(fp.to_string().parse(), Ok(fp));
        assert_eq!(fp.to_hex().parse(), Ok(fp));
    }
}

#[test]
fn ignores_colons_and_spaces_in_any_position() {
    let expected = from_hex(ABC_HEX);
    let spaced: String = ABC_HEX
        .as_bytes()
        .chunks(2)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ");
    let colon_per_digit: String = ABC_HEX.chars().map(|c| format!("{c}:")).collect();
    let cases = [
        spaced,
        colon_per_digit,
        format!(":{ABC_HEX}"),
        format!("{ABC_HEX}:"),
        format!(" {ABC_HEX} "),
        format!("  :  {ABC_HEX}  :  "),
        format!("{}:{}", &ABC_HEX[..1], &ABC_HEX[1..]),
        format!("{}  {}", &ABC_HEX[..33], &ABC_HEX[33..]),
        format!("::{}::{}::", &ABC_HEX[..10], &ABC_HEX[10..]),
    ];
    for text in cases {
        assert_eq!(text.parse(), Ok(expected), "{text:?}");
    }
}

#[test]
fn rejects_wrong_digit_counts() {
    for count in [0usize, 1, 2, 31, 32, 62, 63, 65, 66, 96, 128] {
        let text = "a".repeat(count);
        assert_eq!(
            text.parse::<Fingerprint>(),
            Err(ParseFingerprintError),
            "{count} digits"
        );
    }
    let odd = format!("{ABC_HEX}a");
    assert!(odd.parse::<Fingerprint>().is_err());
}

#[test]
fn rejects_digit_counts_that_only_look_right_with_separators() {
    // 32 pairs + one stray digit; 31 pairs; SHA-1 sized (20 bytes).
    let pairs = |n: usize| vec!["ab"; n].join(":");
    assert!(format!("{}:c", pairs(32)).parse::<Fingerprint>().is_err());
    assert!(pairs(31).parse::<Fingerprint>().is_err());
    assert!(pairs(20).parse::<Fingerprint>().is_err());
    assert!(pairs(33).parse::<Fingerprint>().is_err());
}

#[test]
fn rejects_non_hex_characters() {
    let replace_at = |index: usize, with: &str| {
        let mut text = ABC_HEX.to_owned();
        text.replace_range(index..index + 1, with);
        text
    };
    for bad in ["g", "G", "x", "-", "_", ".", "é", "٣", "０"] {
        for index in [0, 31, 63] {
            let text = if bad.len() == 1 {
                replace_at(index, bad)
            } else {
                format!("{}{bad}", &ABC_HEX[..63])
            };
            assert!(text.parse::<Fingerprint>().is_err(), "{text:?}");
        }
    }
}

#[test]
fn rejects_prefixes_and_other_separators() {
    for text in [
        format!("0x{ABC_HEX}"),
        format!("sha256:{ABC_HEX}"),
        format!("SHA256 Fingerprint={ABC_HEX}"),
        ABC_HEX
            .as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join("-"),
        ABC_HEX
            .as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join("."),
        ABC_HEX
            .as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join(","),
    ] {
        assert_eq!(
            text.parse::<Fingerprint>(),
            Err(ParseFingerprintError),
            "{text:?}"
        );
    }
}

#[test]
fn rejects_whitespace_other_than_the_ascii_space() {
    for separator in ["\t", "\n", "\r\n", "\u{a0}", "\u{2009}"] {
        let text = format!("{}{separator}{}", &ABC_HEX[..32], &ABC_HEX[32..]);
        assert_eq!(
            text.parse::<Fingerprint>(),
            Err(ParseFingerprintError),
            "{separator:?}"
        );
    }
}

#[test]
fn rejects_empty_and_separator_only_input() {
    for text in [
        "",
        " ",
        ":",
        "::::",
        " : : ",
        "                                ",
    ] {
        assert_eq!(
            text.parse::<Fingerprint>(),
            Err(ParseFingerprintError),
            "{text:?}"
        );
    }
}

#[test]
fn parse_error_has_a_fixed_message() {
    let err = "zz".parse::<Fingerprint>().unwrap_err();
    assert_eq!(err.to_string(), "not a SHA-256 fingerprint");
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<ParseFingerprintError>();
}
