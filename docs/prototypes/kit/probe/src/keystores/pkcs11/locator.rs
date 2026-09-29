//! The opaque handle a listed key carries so it can be found again when
//! signing: `slot=3;id=0a1b`.
//!
//! Slot numbers are only stable while the same readers stay plugged in, so
//! the locator is a first guess, not an identity: the signing side verifies it
//! against the certificate and searches the other slots when it is stale.

use std::fmt::Write as _;

/// Where a certificate was found on its module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locator {
    pub slot: u64,
    /// `CKA_ID`, which ties a certificate to its private key.
    pub id: Vec<u8>,
}

impl Locator {
    pub fn format(&self) -> String {
        let mut text = format!("slot={};id=", self.slot);
        for byte in &self.id {
            let _ = write!(text, "{byte:02x}");
        }
        text
    }

    pub fn parse(text: &str) -> Option<Locator> {
        let (slot, id) = text.split_once(';')?;
        let slot = slot.strip_prefix("slot=")?.parse().ok()?;
        let id = decode_hex(id.strip_prefix("id=")?)?;
        Some(Locator { slot, id })
    }
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|start| u8::from_str_radix(&text[start..start + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_slot_and_lowercase_hex_id() {
        let locator = Locator {
            slot: 3,
            id: vec![0x0a, 0x1b, 0x00],
        };
        assert_eq!(locator.format(), "slot=3;id=0a1b00");
    }

    #[test]
    fn round_trips_including_an_empty_id() {
        for id in [vec![], vec![0], vec![0xff, 0x01, 0x80]] {
            let locator = Locator {
                slot: 1_234_567,
                id,
            };
            assert_eq!(Locator::parse(&locator.format()), Some(locator));
        }
    }

    #[test]
    fn rejects_malformed_text() {
        for text in [
            "",
            "slot=1",
            "slot=x;id=00",
            "slot=1;id=0",
            "slot=1;id=zz",
            "id=00;slot=1",
            "slot=1;id=é0",
        ] {
            assert_eq!(Locator::parse(text), None, "{text:?}");
        }
    }
}
