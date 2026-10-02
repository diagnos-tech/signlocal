//! The SHA-1 thumbprint Windows indexes certificates by.

/// SHA-1 of the certificate, as Windows indexes it (`CERT_HASH_PROP_ID`).
/// Used only to find the certificate again, never as its identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thumbprint(pub(super) [u8; 20]);

impl Thumbprint {
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 40 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let mut bytes = [0u8; 20];
        for (byte, pair) in bytes.iter_mut().zip(hex.as_bytes().chunks(2)) {
            *byte = u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?;
        }
        Some(Self(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::Thumbprint;

    #[test]
    fn thumbprint_hex_round_trips() {
        let hex = "00ff10a0b1c2d3e4f5061728394a5b6c7d8e9fab";
        assert_eq!(Thumbprint::from_hex(hex).unwrap().to_hex(), hex);
        assert_eq!(
            Thumbprint::from_hex(&hex.to_uppercase()).unwrap().to_hex(),
            hex
        );
    }

    #[test]
    fn thumbprint_rejects_bad_hex() {
        assert!(Thumbprint::from_hex("").is_none());
        assert!(Thumbprint::from_hex(&"0".repeat(39)).is_none());
        assert!(Thumbprint::from_hex(&"g".repeat(40)).is_none());
        assert!(Thumbprint::from_hex(&"é".repeat(20)).is_none());
        assert!(Thumbprint::from_hex(&"+f".repeat(20)).is_none());
    }
}
