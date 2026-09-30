//! SPEC §5: `Fingerprint`, `ParseFingerprintError`.

mod common;

use std::collections::{BTreeSet, HashSet};

use common::{cert, cert_names, counting_bytes, fixture_fingerprint, hex_decode};
use websign_core::{Fingerprint, ParseFingerprintError};

const ABC_HEX: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const EMPTY_HEX: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn bytes_32(start: u8) -> [u8; 32] {
    counting_bytes(start, 32).try_into().unwrap()
}

fn from_hex(hex: &str) -> Fingerprint {
    Fingerprint::from_bytes(hex_decode(hex).try_into().expect("32 bytes"))
}

/// `00:01:02:...:1F` for the bytes 0..32.
fn counting_display() -> String {
    (0..32)
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

// --- construction and accessors -------------------------------------------------

#[test]
fn computes_sha256_of_the_input() {
    assert_eq!(Fingerprint::of(b"abc").to_hex(), ABC_HEX);
    assert_eq!(Fingerprint::of(b"").to_hex(), EMPTY_HEX);
}

#[test]
fn hashes_every_certificate_fixture_like_openssl() {
    let names = cert_names();
    assert!(names.len() > 100, "fixture set went missing");
    for name in names {
        assert_eq!(
            Fingerprint::of(&cert(&name)),
            fixture_fingerprint(&name),
            "{name}"
        );
    }
}

#[test]
fn different_inputs_have_different_fingerprints() {
    assert_ne!(Fingerprint::of(b"a"), Fingerprint::of(b"b"));
    assert_ne!(Fingerprint::of(b""), Fingerprint::of(&[0]));
    assert_eq!(Fingerprint::of(b"a"), Fingerprint::of(b"a"));
}

#[test]
fn from_bytes_and_as_bytes_are_inverse() {
    for start in [0u8, 1, 0x80, 0xFF] {
        let bytes = bytes_32(start);
        assert_eq!(Fingerprint::from_bytes(bytes).as_bytes(), &bytes);
    }
}

#[test]
fn of_stores_the_raw_digest_bytes() {
    assert_eq!(
        Fingerprint::of(b"abc").as_bytes()[..],
        hex_decode(ABC_HEX)[..]
    );
}

// --- to_hex ---------------------------------------------------------------------

#[test]
fn hex_is_64_lowercase_digits_without_separators() {
    let hex = Fingerprint::from_bytes([0xAB; 32]).to_hex();
    assert_eq!(hex, "ab".repeat(32));
    assert_eq!(hex.len(), 64);
    let hex = Fingerprint::from_bytes(bytes_32(0xF0)).to_hex();
    assert!(
        hex.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
}

#[test]
fn hex_keeps_leading_zeros() {
    assert_eq!(Fingerprint::from_bytes([0; 32]).to_hex(), "0".repeat(64));
    let mut bytes = [0; 32];
    bytes[31] = 1;
    assert_eq!(
        Fingerprint::from_bytes(bytes).to_hex(),
        format!("{}01", "00".repeat(31))
    );
}

// --- Display and Debug -----------------------------------------------------------------

#[test]
fn displays_uppercase_pairs_separated_by_colons() {
    let fp = Fingerprint::from_bytes(bytes_32(0));
    assert_eq!(fp.to_string(), counting_display());
    assert_eq!(
        Fingerprint::of(b"abc").to_string(),
        "BA:78:16:BF:8F:01:CF:EA:41:41:40:DE:5D:AE:22:23:B0:03:61:A3:96:17:7A:9C:B4:10:FF:61:F2:00:15:AD"
    );
}

#[test]
fn display_is_95_characters_with_31_separators() {
    for start in [0u8, 0x7F, 0xFF] {
        let text = Fingerprint::from_bytes(bytes_32(start)).to_string();
        assert_eq!(text.len(), 95);
        assert_eq!(text.matches(':').count(), 31);
        assert!(!text.chars().any(|c| c.is_ascii_lowercase()));
        assert!(text.split(':').all(|pair| pair.len() == 2));
    }
}

#[test]
fn display_zero_pads_small_bytes() {
    let text = Fingerprint::from_bytes([0x0A; 32]).to_string();
    assert!(text.starts_with("0A:0A:"));
    assert!(text.ends_with(":0A"));
}

#[test]
fn debug_wraps_the_lowercase_hex() {
    let fp = Fingerprint::of(b"abc");
    assert_eq!(format!("{fp:?}"), format!("Fingerprint({ABC_HEX})"));
    assert_eq!(
        format!("{:?}", Fingerprint::from_bytes([0; 32])),
        format!("Fingerprint({})", "0".repeat(64))
    );
}

// --- FromStr --------------------------------------------------------------------------------

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

// --- derived traits ------------------------------------------------------------------------------

#[test]
fn is_copy_and_compares_by_bytes() {
    let a = Fingerprint::from_bytes([1; 32]);
    let b = a;
    assert_eq!(a, b);
    assert_ne!(a, Fingerprint::from_bytes([2; 32]));
    let mut last_differs = [1; 32];
    last_differs[31] = 2;
    assert_ne!(a, Fingerprint::from_bytes(last_differs));
}

#[test]
fn hashes_consistently_with_equality() {
    let set: HashSet<Fingerprint> = [[1u8; 32], [2; 32], [1; 32]]
        .into_iter()
        .map(Fingerprint::from_bytes)
        .collect();
    assert_eq!(set.len(), 2);
    assert!(set.contains(&Fingerprint::from_bytes([2; 32])));
}

#[test]
fn orders_lexicographically_by_bytes() {
    let mut low = [0u8; 32];
    low[31] = 1;
    let mut mid = [0u8; 32];
    mid[0] = 1;
    let high = [0xFF; 32];
    let sorted = [
        Fingerprint::from_bytes([0; 32]),
        Fingerprint::from_bytes(low),
        Fingerprint::from_bytes(mid),
        Fingerprint::from_bytes(high),
    ];
    assert!(sorted.windows(2).all(|w| w[0] < w[1]));
    let shuffled: BTreeSet<Fingerprint> = [sorted[3], sorted[1], sorted[0], sorted[2]]
        .into_iter()
        .collect();
    assert_eq!(shuffled.into_iter().collect::<Vec<_>>(), sorted);
    assert_eq!(sorted[1].cmp(&sorted[1]), std::cmp::Ordering::Equal);
}
