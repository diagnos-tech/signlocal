//! Which hardware a PKCS#11 key lives on (`SPEC.md` §3.3), for "Token
//! SafeNet eToken 5110" / "Card in reader" and for matching the key to a
//! device the app saw plugged in.
//!
//! Model and manufacturer are safe to show; the token label and serial number
//! are not (labels often carry the holder's name), so they are never used.

use websign_devices::anonymous_reader_name;

use crate::DeviceLink;

/// What the slot and token say about themselves.
#[derive(Debug, Clone, Copy)]
pub struct SlotFacts<'a> {
    pub model: &'a str,
    pub manufacturer: &'a str,
    pub slot_description: &'a str,
    /// `CKF_REMOVABLE_DEVICE`: the token can leave (a card in a reader).
    pub removable: bool,
}

/// The reader when the slot is a removable card in a reader PC/SC also sees,
/// else the token's model and manufacturer, else nothing.
///
/// `pcsc_readers` returns the anonymous names of the readers PC/SC sees; it
/// is only called for removable slots, so software tokens never pay for a
/// PC/SC scan.
pub fn link(
    facts: &SlotFacts<'_>,
    pcsc_readers: impl FnOnce() -> Vec<String>,
) -> Option<DeviceLink> {
    if facts.removable {
        let reader = anonymous_reader_name(facts.slot_description);
        if !reader.is_empty() && pcsc_readers().contains(&reader) {
            return Some(DeviceLink::Reader { name: reader });
        }
    }
    let model = facts.model.trim();
    (!model.is_empty()).then(|| DeviceLink::Pkcs11Token {
        model: model.to_owned(),
        manufacturer: facts.manufacturer.trim().to_owned(),
    })
}

/// Anonymous names of the readers PC/SC sees now; empty without the service.
pub fn pcsc_reader_names() -> Vec<String> {
    websign_devices::pcsc::scan()
        .readers
        .into_iter()
        .map(|reader| reader.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: SlotFacts<'static> = SlotFacts {
        model: "PKCS#15 emulated",
        manufacturer: "OpenSC Project ",
        slot_description: "Alcor Micro AU9540 (0A1B2C3D) 00 00",
        removable: true,
    };

    fn readers() -> Vec<String> {
        vec!["Alcor Micro AU9540 00 00".to_owned()]
    }

    #[test]
    fn a_removable_card_in_a_known_reader_links_to_the_reader_without_its_serial() {
        assert_eq!(
            link(&CARD, readers),
            Some(DeviceLink::Reader {
                name: "Alcor Micro AU9540 00 00".to_owned()
            })
        );
    }

    #[test]
    fn otherwise_the_token_model_and_manufacturer_are_used_trimmed() {
        let expected = Some(DeviceLink::Pkcs11Token {
            model: "PKCS#15 emulated".to_owned(),
            manufacturer: "OpenSC Project".to_owned(),
        });
        assert_eq!(link(&CARD, Vec::new), expected);
        let fixed = SlotFacts {
            removable: false,
            ..CARD
        };
        assert_eq!(
            link(&fixed, || panic!("no PC/SC scan for fixed slots")),
            expected
        );
    }

    #[test]
    fn a_blank_model_gives_no_link() {
        let blank = SlotFacts {
            model: "   ",
            removable: false,
            ..CARD
        };
        assert_eq!(link(&blank, Vec::new), None);
    }
}
