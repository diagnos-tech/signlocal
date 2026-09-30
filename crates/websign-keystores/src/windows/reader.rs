//! Which PC/SC reader a key's card sits in, so the app can say "Card in
//! reader Identiv" and drop that reader from "possible certificates".
//!
//! The reader comes from the container name when it is fully qualified,
//! which costs nothing. Otherwise only Microsoft's smart card providers are
//! asked (`NCRYPT_READER_PROPERTY`, `PP_SMARTCARD_READER`): they sit on the
//! minidriver stack, honor the silent flags, and answer from the card's
//! public container map without a PIN. Third-party providers are never
//! asked while listing: one that ignored the silent flag would show a
//! dialog for a mere listing.

use websign_devices::anonymous_reader_name;

use super::key_info::KeyLocation;
use super::reader_query;
use crate::DeviceLink;

/// Providers trusted to answer a silent reader query.
const MINIDRIVER_PROVIDERS: [&str; 2] = [
    "Microsoft Smart Card Key Storage Provider",
    "Microsoft Base Smart Card Crypto Provider",
];

/// The reader of `location`'s card, when it can be learned without UI.
pub fn device_link(location: &KeyLocation) -> Option<DeviceLink> {
    let name = match location.qualified_reader() {
        Some(reader) => reader.to_owned(),
        None if is_minidriver(location) => {
            // TODO(gustavo): confirm with SafeNet, SafeSign, ePass2003 and
            // Watchdata cards that this query shows no UI and adds no
            // noticeable delay to listing.
            if location.is_cng() {
                reader_query::ksp_reader(location)?
            } else {
                reader_query::csp_reader(location)?
            }
        }
        None => return None,
    };
    let name = anonymous_reader_name(&name);
    (!name.is_empty()).then_some(DeviceLink::Reader { name })
}

/// Machine keys are never on a user's card, and the query would need the
/// machine key set, so they are left out.
fn is_minidriver(location: &KeyLocation) -> bool {
    !location.machine_keyset
        && MINIDRIVER_PROVIDERS
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&location.provider))
}

#[cfg(test)]
mod tests {
    use super::{device_link, is_minidriver};
    use crate::DeviceLink;
    use crate::windows::key_info::KeyLocation;

    fn location(provider: &str, container: &str) -> KeyLocation {
        KeyLocation {
            provider: provider.to_owned(),
            provider_type: 1,
            container: container.to_owned(),
            key_spec: 2,
            machine_keyset: false,
        }
    }

    #[test]
    fn qualified_container_names_the_reader_without_serials() {
        let key = location("Vendor CSP", r"\\.\Identiv uTrust 3700 F (55041234) 0\c1");
        assert_eq!(
            device_link(&key),
            Some(DeviceLink::Reader {
                name: "Identiv uTrust 3700 F 0".to_owned()
            })
        );
    }

    #[test]
    fn third_party_providers_are_never_asked() {
        assert_eq!(device_link(&location("Vendor CSP", "le-1234")), None);
    }

    #[test]
    fn only_microsoft_minidriver_providers_are_asked() {
        assert!(is_minidriver(&location(
            "Microsoft Smart Card Key Storage Provider",
            "x"
        )));
        assert!(is_minidriver(&location(
            "microsoft base smart card crypto provider",
            "x"
        )));
        assert!(!is_minidriver(&location(
            "Microsoft Software Key Storage Provider",
            "x"
        )));
        let mut machine = location("Microsoft Smart Card Key Storage Provider", "x");
        machine.machine_keyset = true;
        assert!(!is_minidriver(&machine));
    }
}
