//! The issuer and subject names of a certificate as raw DER, for chaining.
//!
//! `websign-core` summarizes names for people (CN, O, …) and drops the rest,
//! so two different CAs can summarize alike. Chaining compares the encoded
//! names byte for byte instead, which is what issuers and subjects are
//! required to match by (RFC 5280 §7.1 allows more, but CAs encode
//! consistently).

/// `(issuer, subject)`, each a complete `Name` TLV, or `None` when the bytes
/// are not a certificate this reader can walk.
pub fn issuer_and_subject(der: &[u8]) -> Option<(&[u8], &[u8])> {
    let (certificate, _) = element(der, SEQUENCE)?;
    let (tbs, _) = element(content(certificate)?, SEQUENCE)?;
    let mut rest = content(tbs)?;
    if rest.first() == Some(&EXPLICIT_VERSION) {
        rest = skip(rest)?;
    }
    rest = skip(rest)?; // serialNumber
    rest = skip(rest)?; // signature algorithm
    let (issuer, after_issuer) = element(rest, SEQUENCE)?;
    let after_validity = skip(after_issuer)?;
    let (subject, _) = element(after_validity, SEQUENCE)?;
    Some((issuer, subject))
}

const SEQUENCE: u8 = 0x30;
const EXPLICIT_VERSION: u8 = 0xa0;

/// The TLV at the start of `bytes` with tag `tag`, and what follows it.
fn element(bytes: &[u8], tag: u8) -> Option<(&[u8], &[u8])> {
    if *bytes.first()? != tag {
        return None;
    }
    let (header, length) = header(bytes)?;
    let end = header.checked_add(length)?;
    (end <= bytes.len()).then(|| bytes.split_at(end))
}

/// What follows the TLV at the start of `bytes`.
fn skip(bytes: &[u8]) -> Option<&[u8]> {
    let tag = *bytes.first()?;
    element(bytes, tag).map(|(_, rest)| rest)
}

/// The value of a complete TLV.
fn content(tlv: &[u8]) -> Option<&[u8]> {
    let (header, length) = header(tlv)?;
    tlv.get(header..header.checked_add(length)?)
}

/// Header length and value length of the TLV at the start of `bytes`
/// (single-byte tags only, which is all X.509 uses up to the subject).
fn header(bytes: &[u8]) -> Option<(usize, usize)> {
    let first = *bytes.get(1)?;
    if first < 0x80 {
        return Some((2, usize::from(first)));
    }
    let count = usize::from(first & 0x7f);
    if count == 0 || count > 4 {
        return None;
    }
    let length = bytes
        .get(2..2 + count)?
        .iter()
        .fold(0usize, |length, &byte| (length << 8) | usize::from(byte));
    Some((2 + count, length))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal certificate shape: version, serial, algorithm, issuer,
    /// validity, subject (contents are placeholders; only the layout matters).
    fn certificate(issuer: &[u8], subject: &[u8], version: bool) -> Vec<u8> {
        let mut tbs = Vec::new();
        if version {
            tbs.extend([0xa0, 0x03, 0x02, 0x01, 0x02]);
        }
        tbs.extend([0x02, 0x01, 0x07]);
        tbs.extend([0x30, 0x00]);
        tbs.extend(issuer);
        tbs.extend([0x30, 0x00]);
        tbs.extend(subject);
        let tbs = tlv(0x30, &tbs);
        tlv(
            0x30,
            &[tbs, vec![0x30, 0x00], vec![0x03, 0x01, 0x00]].concat(),
        )
    }

    fn tlv(tag: u8, value: &[u8]) -> Vec<u8> {
        let mut out = vec![tag];
        if value.len() < 0x80 {
            out.push(value.len() as u8);
        } else {
            out.extend([0x82, (value.len() >> 8) as u8, value.len() as u8]);
        }
        out.extend(value);
        out
    }

    #[test]
    fn finds_issuer_and_subject_with_and_without_a_version() {
        let issuer = tlv(0x30, b"issuer-name");
        let subject = tlv(0x30, &[b'x'; 200]);
        for version in [true, false] {
            let der = certificate(&issuer, &subject, version);
            assert_eq!(
                issuer_and_subject(&der),
                Some((issuer.as_slice(), subject.as_slice()))
            );
        }
    }

    #[test]
    fn truncated_or_foreign_bytes_give_none() {
        let der = certificate(&tlv(0x30, b"i"), &tlv(0x30, b"s"), true);
        for cut in 0..der.len() {
            assert_eq!(issuer_and_subject(&der[..cut]), None, "cut at {cut}");
        }
        assert_eq!(issuer_and_subject(b"not a certificate"), None);
        assert_eq!(issuer_and_subject(&[0x30, 0x85, 1, 1, 1, 1, 1]), None);
    }
}
