//! Standard Base64 (RFC 4648 §4, padded), the encoding the SDK uses for
//! digests and signatures.
//!
//! Small enough to own, so the host needs no extra dependency. Decoding is
//! strict: no whitespace, no URL-safe alphabet, padding required, and unused
//! trailing bits must be zero. A digest that decodes two different ways would
//! be an ambiguity worth refusing.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Input that is not canonical padded Base64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid base64")]
pub struct DecodeError;

pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, &b)| acc | u32::from(b) << (16 - 8 * i));
        let sextets = chunk.len() + 1;
        for i in 0..4 {
            if i < sextets {
                out.push(char::from(
                    ALPHABET[(group >> (18 - 6 * i) & 0x3f) as usize],
                ));
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub fn decode(text: &str) -> Result<Vec<u8>, DecodeError> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(DecodeError);
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let last_chunk = bytes.len().saturating_sub(4);
    for (index, chunk) in bytes.chunks(4).enumerate() {
        let padding = chunk.iter().rev().take_while(|&&b| b == b'=').count();
        // Padding is only legal at the very end, and at most two characters.
        if padding > 2 || (padding > 0 && index * 4 != last_chunk) {
            return Err(DecodeError);
        }
        let mut group = 0u32;
        for &symbol in &chunk[..4 - padding] {
            group = group << 6 | u32::from(sextet(symbol)?);
        }
        group <<= 6 * padding;
        let produced = 3 - padding;
        // Bits that do not belong to any output byte must be zero.
        if group & ((1 << (8 * (3 - produced))) - 1) != 0 {
            return Err(DecodeError);
        }
        out.extend_from_slice(&group.to_be_bytes()[1..1 + produced]);
    }
    Ok(out)
}

fn sextet(symbol: u8) -> Result<u8, DecodeError> {
    match symbol {
        b'A'..=b'Z' => Ok(symbol - b'A'),
        b'a'..=b'z' => Ok(symbol - b'a' + 26),
        b'0'..=b'9' => Ok(symbol - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(DecodeError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4648 §10 test vectors.
    const VECTORS: [(&str, &str); 7] = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];

    #[test]
    fn encodes_rfc_vectors() {
        for (plain, encoded) in VECTORS {
            assert_eq!(encode(plain.as_bytes()), encoded);
        }
    }

    #[test]
    fn decodes_rfc_vectors() {
        for (plain, encoded) in VECTORS {
            assert_eq!(decode(encoded).unwrap(), plain.as_bytes());
        }
    }

    #[test]
    fn round_trips_every_byte_value_and_length() {
        let all: Vec<u8> = (0..=255).collect();
        for len in 0..all.len() {
            assert_eq!(decode(&encode(&all[..len])).unwrap(), &all[..len]);
        }
    }

    #[test]
    fn uses_the_standard_alphabet() {
        assert_eq!(encode(&[0xfb, 0xff, 0xbf]), "+/+/");
        assert_eq!(decode("+/+/").unwrap(), [0xfb, 0xff, 0xbf]);
        assert_eq!(decode("-_-_"), Err(DecodeError));
    }

    #[test]
    fn rejects_malformed_input() {
        for bad in [
            "Zg",       // missing padding
            "Zg=",      // wrong padding length
            "Z===",     // too much padding
            "Zg==Zg==", // padding in the middle
            "Zm9v ",    // whitespace
            "Zm9v\n", "Zm 9", "Zh==", // non-zero trailing bits
            "Zm9=", // non-zero trailing bits
            "Zm9*", "é===",
        ] {
            assert_eq!(decode(bad), Err(DecodeError), "{bad:?} must be rejected");
        }
    }
}
