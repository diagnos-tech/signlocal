//! The few DER primitives needed for `ECDSA-Sig-Value`.
//!
//! Written by hand instead of using a DER crate because the accepted grammar
//! is deliberately not strict DER: tokens differ in how minimal their INTEGERs
//! are, and the length forms allowed are a small, specific subset.

/// Reads a length in short form, or long form with a single length byte.
///
/// Returns `None` for the indefinite form (`80`), for long forms with more
/// than one length byte (`82 ..`) and for truncated input. A `P-521`
/// signature is the only one that needs `81 xx`.
///
/// SPEC: `81 xx` with `xx < 0x80` (a non-minimal length) is accepted like
/// any other `81 xx`; the SPEC lists only the indefinite and multi-byte forms
/// as malformed, and tokens are known to be sloppy about minimality.
fn read_length(input: &[u8]) -> Option<(usize, &[u8])> {
    let (&first, rest) = input.split_first()?;
    match first {
        0x00..=0x7f => Some((usize::from(first), rest)),
        0x81 => {
            let (&len, rest) = rest.split_first()?;
            Some((usize::from(len), rest))
        }
        _ => None,
    }
}

/// Reads one TLV whose tag must be `tag`.
///
/// Returns the content and whatever follows it, or `None` if the tag differs,
/// the length form is not accepted or the content is truncated.
pub(super) fn read_tlv(input: &[u8], tag: u8) -> Option<(&[u8], &[u8])> {
    let (&found, rest) = input.split_first()?;
    if found != tag {
        return None;
    }
    let (len, rest) = read_length(rest)?;
    rest.split_at_checked(len)
}

/// Appends a definite length: short form up to 127, minimal long form above.
pub(super) fn push_length(out: &mut Vec<u8>, len: usize) {
    if let Ok(short) = u8::try_from(len)
        && short < 0x80
    {
        out.push(short);
        return;
    }
    let bytes = len.to_be_bytes();
    let leading_zeros = bytes.iter().take_while(|&&b| b == 0).count();
    let significant = bytes.get(leading_zeros..).unwrap_or_default();
    // A `usize` has at most 8 bytes, so the count always fits in the low bits.
    out.push(0x80 | significant.len() as u8);
    out.extend_from_slice(significant);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_and_one_byte_long_lengths_are_read() {
        assert_eq!(
            read_tlv(&[0x02, 0x01, 0x05, 0xaa], 0x02),
            Some((&[5][..], &[0xaa][..]))
        );
        let mut long = vec![0x30, 0x81, 0x80];
        long.extend([7u8; 0x80]);
        assert_eq!(
            read_tlv(&long, 0x30).map(|(c, r)| (c.len(), r.len())),
            Some((0x80, 0))
        );
    }

    #[test]
    fn unsupported_length_forms_are_refused() {
        assert_eq!(read_tlv(&[0x30, 0x80, 0x00, 0x00], 0x30), None);
        assert_eq!(read_tlv(&[0x30, 0x82, 0x00, 0x01, 0x00], 0x30), None);
        assert_eq!(read_tlv(&[0x30, 0xff], 0x30), None);
        assert_eq!(read_tlv(&[0x30, 0x81], 0x30), None);
    }

    #[test]
    fn wrong_tag_and_truncated_content_are_refused() {
        assert_eq!(read_tlv(&[0x04, 0x01, 0x00], 0x02), None);
        assert_eq!(read_tlv(&[0x02, 0x05, 0x00], 0x02), None);
        assert_eq!(read_tlv(&[], 0x02), None);
    }

    #[test]
    fn push_length_picks_the_minimal_form() {
        for (len, expected) in [
            (0usize, vec![0x00]),
            (127, vec![0x7f]),
            (128, vec![0x81, 0x80]),
            (255, vec![0x81, 0xff]),
            (256, vec![0x82, 0x01, 0x00]),
        ] {
            let mut out = Vec::new();
            push_length(&mut out, len);
            assert_eq!(out, expected, "length {len}");
        }
    }
}
