//! Certificate validity times as Unix seconds.
//!
//! Hand-rolled because `x509-cert`/`der` cannot represent instants before
//! 1970, while UTCTime covers 1950–2049 and a notBefore in the 1960s is valid.

use super::der::{DerError, GENERALIZED_TIME, Tlv, UTC_TIME};

const INVALID: DerError = DerError::Invalid("time");

/// Seconds since 1970-01-01T00:00:00Z (negative before it) of a `Time`.
///
/// Accepts the two forms RFC 5280 §4.1.2.5 allows: UTCTime `YYMMDDHHMMSSZ`
/// (years 50–99 are 19xx, 00–49 are 20xx) and GeneralizedTime
/// `YYYYMMDDHHMMSSZ`, in any year. Fractional seconds, time-zone offsets,
/// missing seconds and impossible dates (February 30, 24:00) are rejected.
pub(super) fn unix_seconds(time: Tlv<'_>) -> Result<i64, DerError> {
    let (year, rest) = match (time.tag, time.content.len()) {
        (UTC_TIME, 13) => {
            let (yy, rest) = digits(time.content, 2)?;
            (if yy >= 50 { 1900 + yy } else { 2000 + yy }, rest)
        }
        (GENERALIZED_TIME, 15) => digits(time.content, 4)?,
        _ => return Err(INVALID),
    };
    let (month, rest) = digits(rest, 2)?;
    let (day, rest) = digits(rest, 2)?;
    let (hour, rest) = digits(rest, 2)?;
    let (minute, rest) = digits(rest, 2)?;
    let (second, rest) = digits(rest, 2)?;
    let valid = rest == b"Z"
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day)
        && hour < 24
        && minute < 60
        && second < 60;
    if !valid {
        return Err(INVALID);
    }
    // Every term is bounded by the 4-digit year, far from overflowing i64.
    Ok(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// The first `count` bytes of `text` read as a decimal number, and the rest.
fn digits(text: &[u8], count: usize) -> Result<(i64, &[u8]), DerError> {
    let (number, rest) = text.split_at_checked(count).ok_or(INVALID)?;
    if !number.iter().all(u8::is_ascii_digit) {
        return Err(INVALID);
    }
    let value = number
        .iter()
        .fold(0, |value, digit| value * 10 + i64::from(digit - b'0'));
    Ok((value, rest))
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Days from 1970-01-01 to the given proleptic Gregorian date, after
/// Howard Hinnant's `days_from_civil` (years counted from March so the leap
/// day is the last day of the year).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(text: &str) -> Result<i64, DerError> {
        unix_seconds(Tlv {
            tag: UTC_TIME,
            content: text.as_bytes(),
        })
    }

    fn generalized(text: &str) -> Result<i64, DerError> {
        unix_seconds(Tlv {
            tag: GENERALIZED_TIME,
            content: text.as_bytes(),
        })
    }

    #[test]
    fn utc_time_spans_1950_to_2049() {
        assert_eq!(utc("700101000000Z"), Ok(0));
        assert_eq!(utc("500101000000Z"), Ok(-631_152_000));
        assert_eq!(utc("600101000000Z"), Ok(-315_619_200));
        assert_eq!(utc("691231235959Z"), Ok(-1));
        assert_eq!(utc("000101000000Z"), Ok(946_684_800));
        assert_eq!(utc("491231235959Z"), Ok(2_524_607_999));
    }

    #[test]
    fn generalized_time_covers_any_four_digit_year() {
        assert_eq!(generalized("20500101000000Z"), Ok(2_524_608_000));
        assert_eq!(generalized("99991231235959Z"), Ok(253_402_300_799));
        assert_eq!(generalized("19691231235959Z"), Ok(-1));
        assert_eq!(generalized("00000101000000Z"), Ok(-62_167_219_200));
    }

    #[test]
    fn leap_days_follow_the_gregorian_rules() {
        assert_eq!(utc("000229000000Z"), Ok(951_782_400));
        assert_eq!(utc("240229120000Z"), Ok(1_709_208_000));
        assert!(generalized("21000229000000Z").is_err());
        assert!(utc("230229000000Z").is_err());
    }

    #[test]
    fn rejects_forms_rfc_5280_does_not_allow() {
        for bad in [
            "7001010000Z",
            "700101000000",
            "700101000000+0000",
            "700101000000.5Z",
            "7001010000000Z",
            "700132000000Z",
            "701301000000Z",
            "700001000000Z",
            "700100000000Z",
            "700431000000Z",
            "700101240000Z",
            "700101006000Z",
            "700101000060Z",
            "70010100000aZ",
            "7001010000-0Z",
            "700101000000z",
        ] {
            assert!(utc(bad).is_err(), "{bad}");
        }
        assert!(generalized("700101000000Z").is_err());
        assert!(generalized("20500101000000.5Z").is_err());
        assert!(utc("20500101000000Z").is_err());
        let other_tag = Tlv {
            tag: 0x04,
            content: b"700101000000Z",
        };
        assert!(unix_seconds(other_tag).is_err());
    }
}
