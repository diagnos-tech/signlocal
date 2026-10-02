//! Reading `--digest` / `--digest-file` into bytes.

use std::io::Read;
use std::path::Path;

use websign_protocol::types::HashName;

/// Read at most this much from a digest file: one byte more than the
/// longest digest, so an oversized file is reported instead of truncated.
const MAX_FILE_BYTES: u64 = 65;

/// `text` as hex (any case, `:` between bytes allowed) or padded standard
/// Base64. Text made only of hex digits is always hex, which is how every
/// tool prints a digest (the Base64 of a digest essentially never is), so a
/// hex digest of the wrong length is refused instead of being re-read as
/// Base64 of another length.
pub fn parse_text(text: &str, hash: HashName) -> Result<Vec<u8>, String> {
    let text = text.trim();
    let compact: String = text.chars().filter(|&c| c != ':').collect();
    let is_hex = !compact.is_empty() && compact.chars().all(|c| c.is_ascii_hexdigit());
    let bytes = if is_hex {
        hex(&compact)?
    } else {
        websign_protocol::base64::decode(text)
            .map_err(|_| "the digest is neither hex nor standard Base64".to_owned())?
    };
    check_len(bytes, hash)
}

/// The raw bytes of `path`, or of stdin for `-`.
pub fn read_file(path: &Path, hash: HashName) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let result = if path == Path::new("-") {
        std::io::stdin()
            .lock()
            .take(MAX_FILE_BYTES)
            .read_to_end(&mut bytes)
    } else {
        std::fs::File::open(path).and_then(|f| f.take(MAX_FILE_BYTES).read_to_end(&mut bytes))
    };
    result.map_err(|e| format!("the digest file could not be read: {:?}", e.kind()))?;
    check_len(bytes, hash)
}

fn check_len(bytes: Vec<u8>, hash: HashName) -> Result<Vec<u8>, String> {
    let want = hash.digest_len();
    if bytes.len() == want {
        Ok(bytes)
    } else {
        Err(format!(
            "a {hash:?} digest has {want} bytes, got {}",
            bytes.len()
        ))
    }
}

fn hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) {
        return Err("the hex digest has an odd number of digits".to_owned());
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(|_| "invalid hex".to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA256_HEX: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const SHA256_B64: &str = "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";

    #[test]
    fn hex_in_any_case_with_or_without_colons() {
        let lower = parse_text(SHA256_HEX, HashName::Sha256).unwrap();
        let upper = parse_text(&SHA256_HEX.to_uppercase(), HashName::Sha256).unwrap();
        let colons: Vec<String> = (0..32)
            .map(|i| SHA256_HEX[2 * i..2 * i + 2].to_owned())
            .collect();
        let colons = parse_text(&colons.join(":"), HashName::Sha256).unwrap();
        assert_eq!(lower.len(), 32);
        assert_eq!(lower, upper);
        assert_eq!(lower, colons);
    }

    #[test]
    fn padded_base64_is_the_same_digest() {
        assert_eq!(
            parse_text(SHA256_B64, HashName::Sha256).unwrap(),
            parse_text(SHA256_HEX, HashName::Sha256).unwrap()
        );
    }

    #[test]
    fn a_digest_of_another_length_or_shape_is_refused() {
        assert!(parse_text(SHA256_HEX, HashName::Sha384).is_err());
        assert!(parse_text("zz", HashName::Sha256).is_err());
        assert!(parse_text("", HashName::Sha256).is_err());
        assert!(parse_text(&SHA256_HEX[..62], HashName::Sha256).is_err());
    }

    #[test]
    fn a_digest_file_holds_raw_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d");
        std::fs::write(&path, [7u8; 48]).unwrap();
        assert_eq!(read_file(&path, HashName::Sha384).unwrap(), vec![7u8; 48]);
        std::fs::write(&path, [7u8; 200]).unwrap();
        assert!(read_file(&path, HashName::Sha384).is_err());
    }
}
