//! How a token is described to people, and whether it counts as hardware.
//!
//! The token label and serial number are deliberately never used: on
//! ICP-Brasil and eIDAS cards the label usually carries the holder's name,
//! and this text ends up in reports that are published. The slot description
//! is the reader name for most modules, so it loses the reader's USB serial.

use std::fmt::Write as _;

use crate::devices::anonymous_reader_name;

/// Whether the token shows its private keys before the PIN is entered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyVisibility {
    /// A private key object was listed without logging in.
    Visible,
    /// None was listed and the token requires a login: the key is probably
    /// there but hidden until the PIN is given.
    AfterLogin,
    /// None was listed and no login is required: there is no private key.
    Absent,
}

/// The token's own warning about the user PIN, from `CKF_USER_PIN_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PinState {
    pub count_low: bool,
    pub final_try: bool,
    pub locked: bool,
}

impl PinState {
    /// Short text for the interface's "few attempts left" hints.
    fn note(self) -> Option<&'static str> {
        if self.locked {
            Some("PIN locked")
        } else if self.final_try {
            Some("last PIN attempt")
        } else if self.count_low {
            Some("PIN attempts running low")
        } else {
            None
        }
    }
}

/// Everything the description is made of.
#[derive(Debug, Clone)]
pub struct TokenFacts<'a> {
    pub model: &'a str,
    pub manufacturer: &'a str,
    pub slot_description: &'a str,
    /// File name of the PKCS#11 module, e.g. `libeTPkcs11.so`.
    pub module_file: &'a str,
    pub keys: KeyVisibility,
    pub pin: PinState,
}

/// `"eToken (SafeNet, Inc.); slot "Alcor Micro AU9540 00 00"; libeTPkcs11.so; key hidden until login"`.
pub fn describe(facts: &TokenFacts<'_>) -> String {
    let model = or_unknown(facts.model, "unknown token model");
    let mut text = match facts.manufacturer.trim() {
        "" => model,
        manufacturer => format!("{model} ({manufacturer})"),
    };
    let slot = anonymous_reader_name(facts.slot_description);
    if !slot.is_empty() {
        let _ = write!(text, "; slot \"{slot}\"");
    }
    let _ = write!(text, "; {}", facts.module_file);
    text.push_str(match facts.keys {
        KeyVisibility::Visible => "; key visible",
        KeyVisibility::AfterLogin => "; key hidden until login",
        KeyVisibility::Absent => "; no private key listed",
    });
    if let Some(note) = facts.pin.note() {
        let _ = write!(text, "; {note}");
    }
    text
}

fn or_unknown(text: &str, fallback: &str) -> String {
    match text.trim() {
        "" => fallback.to_owned(),
        text => text.to_owned(),
    }
}

/// `false` for software tokens, `true` for everything else.
///
/// `CKF_HW_SLOT` cannot decide this: SoftHSM leaves it clear, but so do
/// plenty of middleware modules for real cards. What does give software
/// tokens away is their name, so that is what is matched. The answer is
/// therefore "hardware unless the token says it is software".
pub fn is_hardware(facts: &TokenFacts<'_>) -> bool {
    ![facts.model, facts.manufacturer, facts.slot_description]
        .iter()
        .any(|text| is_software_name(text))
}

fn is_software_name(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    ["softhsm", "software", "soft token", "softtoken"]
        .iter()
        .any(|marker| text.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> TokenFacts<'static> {
        TokenFacts {
            model: "eToken",
            manufacturer: "SafeNet, Inc.",
            slot_description: "Alcor Micro AU9540 00 00",
            module_file: "libeTPkcs11.so",
            keys: KeyVisibility::AfterLogin,
            pin: PinState::default(),
        }
    }

    #[test]
    fn describes_model_maker_reader_module_and_key_visibility() {
        assert_eq!(
            describe(&facts()),
            "eToken (SafeNet, Inc.); slot \"Alcor Micro AU9540 00 00\"; libeTPkcs11.so; key hidden until login"
        );
    }

    #[test]
    fn the_readers_usb_serial_never_reaches_the_description() {
        let text = describe(&TokenFacts {
            slot_description: "Gemalto USB Shell Token V2 (29C2E3B5) 00 00",
            ..facts()
        });
        assert!(!text.contains("29C2E3B5"), "{text}");
        assert!(
            text.contains("slot \"Gemalto USB Shell Token V2 00 00\""),
            "{text}"
        );
    }

    #[test]
    fn blank_fields_are_left_out_or_replaced() {
        let text = describe(&TokenFacts {
            model: "  ",
            manufacturer: "",
            slot_description: "",
            keys: KeyVisibility::Visible,
            ..facts()
        });
        assert_eq!(text, "unknown token model; libeTPkcs11.so; key visible");
    }

    #[test]
    fn pin_warnings_follow_severity() {
        let with = |pin| describe(&TokenFacts { pin, ..facts() });
        assert!(
            with(PinState {
                count_low: true,
                ..PinState::default()
            })
            .ends_with("PIN attempts running low")
        );
        assert!(
            with(PinState {
                count_low: true,
                final_try: true,
                locked: false
            })
            .ends_with("last PIN attempt")
        );
        assert!(
            with(PinState {
                count_low: true,
                final_try: true,
                locked: true
            })
            .ends_with("PIN locked")
        );
    }

    #[test]
    fn softhsm_is_software_and_real_tokens_are_hardware() {
        assert!(is_hardware(&facts()));
        let soft = TokenFacts {
            model: "SoftHSM v2",
            manufacturer: "SoftHSM project",
            slot_description: "SoftHSM slot ID 0x1",
            ..facts()
        };
        assert!(!is_hardware(&soft));
        let opensc = TokenFacts {
            model: "PKCS#15 emulated",
            manufacturer: "OpenSC Project",
            ..facts()
        };
        assert!(is_hardware(&opensc));
    }
}
