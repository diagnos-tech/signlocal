//! The least DER needed to build test inputs by hand, plus a strict checker
//! for what `raw_to_der` must produce. Deliberately dumb: it never fixes up
//! its input, so a test can spell out a malformed or non-minimal encoding.

/// Definite-length header: short form below 128, `81 xx` up to 255, else `82 xx xx`.
pub fn length_octets(len: usize) -> Vec<u8> {
    match len {
        0..=127 => vec![len as u8],
        128..=255 => vec![0x81, len as u8],
        _ => vec![0x82, (len >> 8) as u8, len as u8],
    }
}

/// One TLV with exactly the content given.
pub fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend(length_octets(content.len()));
    out.extend_from_slice(content);
    out
}

/// `SEQUENCE { INTEGER r, INTEGER s }` where `r` and `s` are the INTEGER
/// content octets verbatim (sign byte, padding and all).
pub fn der_ecdsa(r: &[u8], s: &[u8]) -> Vec<u8> {
    let mut body = tlv(0x02, r);
    body.extend(tlv(0x02, s));
    tlv(0x30, &body)
}

/// The INTEGER content octets a minimal encoder produces for a big-endian
/// magnitude: no redundant leading zeros, a `00` when the top bit is set.
pub fn minimal_integer(magnitude: &[u8]) -> Vec<u8> {
    let first = magnitude
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(magnitude.len() - 1);
    let significant = &magnitude[first..];
    let mut out = Vec::new();
    if significant[0] >= 0x80 {
        out.push(0);
    }
    out.extend_from_slice(significant);
    out
}

/// Splits a strict, minimal DER `ECDSA-Sig-Value` into the content octets of
/// `r` and `s`. `None` if it is not exactly that: wrong tags, non-minimal
/// lengths or integers, negative values, or bytes left over.
pub fn parse_strict_ecdsa_der(der: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let (tag, body, rest) = split_tlv(der)?;
    if tag != 0x30 || !rest.is_empty() {
        return None;
    }
    let (r_tag, r, after_r) = split_tlv(body)?;
    let (s_tag, s, after_s) = split_tlv(after_r)?;
    if r_tag != 0x02 || s_tag != 0x02 || !after_s.is_empty() {
        return None;
    }
    [r, s]
        .iter()
        .all(|int| is_minimal_positive(int))
        .then(|| (r.to_vec(), s.to_vec()))
}

fn is_minimal_positive(int: &[u8]) -> bool {
    match int {
        [] => false,
        [first, ..] if *first >= 0x80 => false,
        [0, second, ..] => *second >= 0x80,
        _ => true,
    }
}

/// Splits `bytes` into (tag, content, remainder), accepting only minimal
/// definite lengths.
fn split_tlv(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, rest) = bytes.split_first()?;
    let (&first, rest) = rest.split_first()?;
    let (len, rest) = match first {
        0..=127 => (first as usize, rest),
        0x81 => {
            let (&len, rest) = rest.split_first()?;
            (len as usize >= 128).then_some((len as usize, rest))?
        }
        _ => return None,
    };
    (rest.len() >= len).then(|| (tag, &rest[..len], &rest[len..]))
}
