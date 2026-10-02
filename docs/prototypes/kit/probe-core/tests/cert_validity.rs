//! SPEC §6: `serial_hex`, `not_before`, `not_after` and `is_valid_at` (§6.5).

mod common;

use common::{blank_info, info, validity_oracles};
use probe_core::CertInfo;

#[test]
fn serial_and_validity_match_openssl_for_every_fixture() {
    let oracles = validity_oracles();
    assert!(oracles.len() > 100, "oracle manifest went missing");
    for oracle in oracles {
        if oracle.name.starts_with("bad-") {
            continue;
        }
        let info = info(&oracle.name);
        assert_eq!(info.serial_hex, oracle.serial_hex, "{} serial", oracle.name);
        assert_eq!(
            info.not_before, oracle.not_before,
            "{} notBefore",
            oracle.name
        );
        assert_eq!(info.not_after, oracle.not_after, "{} notAfter", oracle.name);
    }
}

#[test]
fn drops_the_sign_byte_from_the_serial() {
    // 0x80 is encoded 00 80; twenty 0xff bytes are encoded with a leading 00.
    assert_eq!(info("serial-80").serial_hex, "80");
    assert_eq!(info("serial-ff20").serial_hex, "ff".repeat(20));
}

#[test]
fn keeps_serials_without_a_sign_byte_as_they_are() {
    assert_eq!(info("serial-01").serial_hex, "01");
    assert_eq!(info("serial-7f").serial_hex, "7f");
    assert_eq!(info("serial-1234").serial_hex, "1234");
}

#[test]
fn keeps_a_leading_zero_nibble_of_the_serial() {
    // Content octets 01 02 are "0102", not "102".
    assert_eq!(info("serial-0102").serial_hex, "0102");
}

#[test]
fn serial_is_lowercase_hex() {
    for name in ["serial-ff20", "serial-80", "p256"] {
        let serial = info(name).serial_hex;
        assert!(
            serial
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{serial}"
        );
        assert_eq!(serial.len() % 2, 0, "whole octets: {serial}");
    }
}

#[test]
fn reads_validity_as_unix_seconds() {
    let cases = [
        ("rsa2048", 1_577_836_800, 2_208_988_800), // 2020-01-01 .. 2040-01-01
        ("time-window", 1_710_498_030, 1_726_425_650), // 2024-03-15T10:20:30Z .. 2024-09-15T18:40:50Z
        ("expired", 1_262_304_000, 1_293_840_000),     // 2010 .. 2011
        ("time-y2k", 946_684_799, 946_684_800), // 1999-12-31T23:59:59Z .. 2000-01-01T00:00:00Z
    ];
    for (name, not_before, not_after) in cases {
        let info = info(name);
        assert_eq!(
            (info.not_before, info.not_after),
            (not_before, not_after),
            "{name}"
        );
    }
}

#[test]
fn reads_utc_time_before_1970_as_a_negative_number() {
    // UTCTime "600101000000Z" is 1960 (years 50..99 mean 19xx).
    let info = info("time-pre-1970");
    assert_eq!(info.not_before, -315_619_200);
    assert_eq!(
        info.not_after, 2_524_607_999,
        "2049-12-31T23:59:59Z is still UTCTime"
    );
}

#[test]
fn reads_generalized_time_from_2050_on() {
    let info = info("time-generalized");
    assert_eq!(info.not_before, 2_524_608_000);
    assert_eq!(info.not_after, 2_556_144_000);
}

#[test]
fn reads_the_no_expiry_date_of_year_9999() {
    assert_eq!(info("time-no-expiry").not_after, 253_402_300_799);
}

#[test]
fn validity_window_is_inclusive_at_both_ends() {
    let info = info("time-window");
    let (from, to) = (info.not_before, info.not_after);
    assert!(!info.is_valid_at(from - 1));
    assert!(info.is_valid_at(from));
    assert!(info.is_valid_at(from + 1));
    assert!(info.is_valid_at(to - 1));
    assert!(info.is_valid_at(to));
    assert!(!info.is_valid_at(to + 1));
}

#[test]
fn expired_fixture_is_not_valid_today_but_was_valid_then() {
    let info = info("expired");
    assert!(!info.is_valid_at(1_800_000_000));
    assert!(info.is_valid_at(1_270_000_000));
    assert!(!info.is_valid_at(0));
}

#[test]
fn validity_check_on_a_hand_built_summary() {
    let info = CertInfo {
        not_before: 1_000,
        not_after: 2_000,
        ..blank_info()
    };
    let cases = [
        (i64::MIN, false),
        (-1, false),
        (0, false),
        (999, false),
        (1_000, true),
        (1_500, true),
        (2_000, true),
        (2_001, false),
        (i64::MAX, false),
    ];
    for (t, expected) in cases {
        assert_eq!(info.is_valid_at(t), expected, "t = {t}");
    }
}

#[test]
fn validity_check_survives_the_extremes_of_i64() {
    let always = CertInfo {
        not_before: i64::MIN,
        not_after: i64::MAX,
        ..blank_info()
    };
    assert!(always.is_valid_at(i64::MIN));
    assert!(always.is_valid_at(0));
    assert!(always.is_valid_at(i64::MAX));

    let instant = CertInfo {
        not_before: 42,
        not_after: 42,
        ..blank_info()
    };
    assert!(instant.is_valid_at(42));
    assert!(!instant.is_valid_at(41));
    assert!(!instant.is_valid_at(43));

    let inverted = CertInfo {
        not_before: 2_000,
        not_after: 1_000,
        ..blank_info()
    };
    for t in [999, 1_000, 1_500, 2_000, 2_001] {
        assert!(!inverted.is_valid_at(t), "t = {t}");
    }
}
