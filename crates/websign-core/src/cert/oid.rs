//! OBJECT IDENTIFIERs: dotted text, and the encoded form of every OID the
//! summary recognizes.
//!
//! Known OIDs are compared on their encoded bytes, so an identifier this
//! crate never needs to print (an unknown extension, say) is never decoded
//! and cannot make a certificate unreadable.

use std::fmt::Write;

/// Dotted text of an OBJECT IDENTIFIER's content octets.
///
/// `None` if the bytes are not a valid encoding (empty, a truncated last arc,
/// or an arc padded with a leading `80`) or an arc exceeds `u128`. The
/// largest arcs in use are the 128-bit UUIDs under `2.25`, so the limit only
/// turns away hostile input, which would otherwise cost quadratic time.
pub(super) fn to_dotted(content: &[u8]) -> Option<String> {
    let mut text = String::new();
    let mut arc: u128 = 0;
    let mut arc_start = true;
    for &byte in content {
        if arc_start && byte == 0x80 {
            return None;
        }
        arc = arc.checked_mul(128)? | u128::from(byte & 0x7f);
        arc_start = byte & 0x80 == 0;
        if arc_start {
            push_arc(&mut text, arc);
            arc = 0;
        }
    }
    (arc_start && !text.is_empty()).then_some(text)
}

/// Appends one decoded subidentifier. The first one packs the first two arcs
/// as `40 * x + y` (X.690 §8.19.4).
fn push_arc(text: &mut String, arc: u128) {
    // Writing to a `String` cannot fail.
    let _ = if text.is_empty() {
        match arc {
            0..40 => write!(text, "0.{arc}"),
            40..80 => write!(text, "1.{}", arc - 40),
            _ => write!(text, "2.{}", arc - 80),
        }
    } else {
        write!(text, ".{arc}")
    };
}

// Distinguished-name attributes (RFC 4519).
/// 2.5.4.3
pub(super) const COMMON_NAME: &[u8] = &[0x55, 0x04, 0x03];
/// 2.5.4.4
pub(super) const SURNAME: &[u8] = &[0x55, 0x04, 0x04];
/// 2.5.4.5
pub(super) const SERIAL_NUMBER: &[u8] = &[0x55, 0x04, 0x05];
/// 2.5.4.6
pub(super) const COUNTRY_NAME: &[u8] = &[0x55, 0x04, 0x06];
/// 2.5.4.10
pub(super) const ORGANIZATION_NAME: &[u8] = &[0x55, 0x04, 0x0a];
/// 2.5.4.11
pub(super) const ORGANIZATIONAL_UNIT_NAME: &[u8] = &[0x55, 0x04, 0x0b];

/// 2.5.4.42
pub(super) const GIVEN_NAME: &[u8] = &[0x55, 0x04, 0x2a];

// Public key algorithms.
/// 1.2.840.113549.1.1.1
pub(super) const RSA_ENCRYPTION: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];
/// 1.2.840.113549.1.1.10
pub(super) const RSASSA_PSS: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0a];
/// 1.2.840.10045.2.1
pub(super) const EC_PUBLIC_KEY: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];

// Certificate extensions (RFC 5280, RFC 3739).
/// 2.5.29.15
pub(super) const KEY_USAGE: &[u8] = &[0x55, 0x1d, 0x0f];
/// 2.5.29.17
pub(super) const SUBJECT_ALT_NAME: &[u8] = &[0x55, 0x1d, 0x11];
/// 2.5.29.19
pub(super) const BASIC_CONSTRAINTS: &[u8] = &[0x55, 0x1d, 0x13];
/// 2.5.29.32
pub(super) const CERTIFICATE_POLICIES: &[u8] = &[0x55, 0x1d, 0x20];
/// 2.5.29.37
pub(super) const EXTENDED_KEY_USAGE: &[u8] = &[0x55, 0x1d, 0x25];
/// 1.3.6.1.5.5.7.1.3
pub(super) const QC_STATEMENTS: &[u8] = &[0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x01, 0x03];

// qcStatements (ETSI EN 319 412-5).
/// 0.4.0.1862.1.1
pub(super) const QC_COMPLIANCE: &[u8] = &[0x04, 0x00, 0x8e, 0x46, 0x01, 0x01];
/// 0.4.0.1862.1.4
pub(super) const QC_SSCD: &[u8] = &[0x04, 0x00, 0x8e, 0x46, 0x01, 0x04];
/// 0.4.0.1862.1.6
pub(super) const QC_TYPE: &[u8] = &[0x04, 0x00, 0x8e, 0x46, 0x01, 0x06];
/// 0.4.0.1862.1.6.1
pub(super) const QC_TYPE_ESIGN: &[u8] = &[0x04, 0x00, 0x8e, 0x46, 0x01, 0x06, 0x01];
/// 0.4.0.1862.1.6.2
pub(super) const QC_TYPE_ESEAL: &[u8] = &[0x04, 0x00, 0x8e, 0x46, 0x01, 0x06, 0x02];
/// 0.4.0.1862.1.6.3
pub(super) const QC_TYPE_WEB: &[u8] = &[0x04, 0x00, 0x8e, 0x46, 0x01, 0x06, 0x03];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_oid_encodes_its_documented_text() {
        let known = [
            (COMMON_NAME, "2.5.4.3"),
            (COUNTRY_NAME, "2.5.4.6"),
            (ORGANIZATION_NAME, "2.5.4.10"),
            (ORGANIZATIONAL_UNIT_NAME, "2.5.4.11"),
            (RSA_ENCRYPTION, "1.2.840.113549.1.1.1"),
            (RSASSA_PSS, "1.2.840.113549.1.1.10"),
            (EC_PUBLIC_KEY, "1.2.840.10045.2.1"),
            (KEY_USAGE, "2.5.29.15"),
            (SUBJECT_ALT_NAME, "2.5.29.17"),
            (BASIC_CONSTRAINTS, "2.5.29.19"),
            (CERTIFICATE_POLICIES, "2.5.29.32"),
            (EXTENDED_KEY_USAGE, "2.5.29.37"),
            (QC_STATEMENTS, "1.3.6.1.5.5.7.1.3"),
            (QC_COMPLIANCE, "0.4.0.1862.1.1"),
            (QC_SSCD, "0.4.0.1862.1.4"),
            (QC_TYPE, "0.4.0.1862.1.6"),
            (QC_TYPE_ESIGN, "0.4.0.1862.1.6.1"),
            (QC_TYPE_ESEAL, "0.4.0.1862.1.6.2"),
            (QC_TYPE_WEB, "0.4.0.1862.1.6.3"),
        ];
        for (bytes, text) in known {
            assert_eq!(to_dotted(bytes).as_deref(), Some(text));
        }
    }

    #[test]
    fn splits_the_first_subidentifier_into_two_arcs() {
        assert_eq!(to_dotted(&[0x00]).as_deref(), Some("0.0"));
        assert_eq!(to_dotted(&[0x27]).as_deref(), Some("0.39"));
        assert_eq!(to_dotted(&[0x28]).as_deref(), Some("1.0"));
        assert_eq!(to_dotted(&[0x4f]).as_deref(), Some("1.39"));
        assert_eq!(to_dotted(&[0x50]).as_deref(), Some("2.0"));
        // 2.999 is 1079 = 0x437 as the first subidentifier.
        assert_eq!(to_dotted(&[0x88, 0x37]).as_deref(), Some("2.999"));
    }

    #[test]
    fn reads_arcs_beyond_u32_up_to_u128() {
        // 2.16.76.1.2.4294967296.1
        let beyond_u32 = [0x60, 0x4c, 0x01, 0x02, 0x90, 0x80, 0x80, 0x80, 0x00, 0x01];
        assert_eq!(
            to_dotted(&beyond_u32).as_deref(),
            Some("2.16.76.1.2.4294967296.1")
        );
        // 2.25.(2^128 - 1): nineteen 7-bit groups, the first holding 2 bits.
        let mut uuid = vec![0x69, 0x83];
        uuid.extend([0xff; 17]);
        uuid.push(0x7f);
        assert_eq!(to_dotted(&uuid), Some(format!("2.25.{}", u128::MAX)));
    }

    #[test]
    fn rejects_invalid_encodings_and_oversized_arcs() {
        let mut too_big = vec![0x69, 0x87];
        too_big.extend([0xff; 17]);
        too_big.push(0x7f);
        for bad in [
            &[][..],
            &[0x55, 0x1d, 0x8f],
            &[0x55, 0x80, 0x01],
            &[0x80, 0x01],
            &too_big,
        ] {
            assert_eq!(to_dotted(bad), None, "{bad:02x?}");
        }
    }
}
