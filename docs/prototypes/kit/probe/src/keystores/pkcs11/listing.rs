//! `list`: the signing certificates on every token of a module, without a PIN.
//!
//! Whether a certificate has a private key is the awkward part. PKCS#11 links
//! them by `CKA_ID`, but many tokens hide private key objects until the user
//! logs in, and logging in is exactly what `list` must not do. The rule:
//!
//! * If the token lists any private key without a login, only certificates
//!   whose `CKA_ID` matches one of them count. Certificates without a match
//!   (CA certificates stored on the token) are left out.
//! * If it lists none and requires a login (`CKF_LOGIN_REQUIRED`), the keys
//!   are assumed to be hidden, so every certificate counts. CA certificates
//!   may sneak in; the caller filters those by their key usage.
//! * If it lists none and requires no login, there is no key.
//!
//! The provider text says which of the three applied.

use cryptoki::context::Pkcs11;
use cryptoki::error::Error;
use cryptoki::slot::{Slot, SlotInfo, TokenInfo};
use probe_core::SourceKind;

use super::errors::{self, Context};
use super::locator::Locator;
use super::objects::{self, StoredCertificate};
use super::provider::{self, KeyVisibility, PinState, TokenFacts};
use crate::keystores::{FoundKey, KeystoreError, PinPrompt};
use crate::trace::trace;

/// Lists the keys of every slot with a token. A slot that fails is skipped;
/// the failures are reported only when nothing at all could be listed.
pub fn list(
    pkcs11: &Pkcs11,
    keystore: &str,
    module_file: &str,
) -> Result<Vec<FoundKey>, KeystoreError> {
    trace!("{module_file}: C_GetSlotList(token present)");
    let slots = pkcs11
        .get_slots_with_token()
        .map_err(errors::mapper(Context::default()))?;
    let (mut keys, mut problems) = (Vec::new(), Vec::new());
    for slot in slots {
        trace!("{module_file}: reading slot {} without login", slot.id());
        match list_slot(pkcs11, slot, keystore, module_file) {
            Ok(found) => keys.extend(found),
            Err(error) => problems.push(format!(
                "slot {}: {}",
                slot.id(),
                errors::map(error, Context::default())
            )),
        }
    }
    if keys.is_empty() && !problems.is_empty() {
        return Err(KeystoreError::Other(problems.join("; ")));
    }
    Ok(keys)
}

fn list_slot(
    pkcs11: &Pkcs11,
    slot: Slot,
    keystore: &str,
    module_file: &str,
) -> Result<Vec<FoundKey>, Error> {
    let token = pkcs11.get_token_info(slot)?;
    let slot_info = pkcs11.get_slot_info(slot)?;
    let session = pkcs11.open_ro_session(slot)?;
    let certificates = objects::certificates(&session)?;
    let key_ids = objects::visible_private_key_ids(&session);
    Ok(certificates
        .into_iter()
        .filter_map(|certificate| {
            let keys = visibility(token.login_required(), &key_ids, &certificate.id);
            (keys != KeyVisibility::Absent).then(|| {
                found_key(
                    slot,
                    &slot_info,
                    &token,
                    certificate,
                    keys,
                    keystore,
                    module_file,
                )
            })
        })
        .collect())
}

/// The three-way rule from the module documentation.
fn visibility(login_required: bool, key_ids: &[Vec<u8>], certificate_id: &[u8]) -> KeyVisibility {
    if !key_ids.is_empty() {
        return if key_ids.iter().any(|id| id == certificate_id) {
            KeyVisibility::Visible
        } else {
            KeyVisibility::Absent
        };
    }
    if login_required {
        KeyVisibility::AfterLogin
    } else {
        KeyVisibility::Absent
    }
}

fn found_key(
    slot: Slot,
    slot_info: &SlotInfo,
    token: &TokenInfo,
    certificate: StoredCertificate,
    keys: KeyVisibility,
    keystore: &str,
    module_file: &str,
) -> FoundKey {
    let facts = TokenFacts {
        model: token.model(),
        manufacturer: token.manufacturer_id(),
        slot_description: slot_info.slot_description(),
        module_file,
        keys,
        pin: PinState {
            count_low: token.user_pin_count_low(),
            final_try: token.user_pin_final_try(),
            locked: token.user_pin_locked(),
        },
    };
    FoundKey {
        locator: Locator {
            slot: slot.id(),
            id: certificate.id,
        }
        .format(),
        cert_der: certificate.der,
        keystore: keystore.to_owned(),
        kind: SourceKind::Pkcs11,
        provider: provider::describe(&facts),
        hardware: Some(provider::is_hardware(&facts)),
        pin: PinPrompt::App {
            protected_path: token.protected_authentication_path(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&[u8]]) -> Vec<Vec<u8>> {
        list.iter().map(|id| id.to_vec()).collect()
    }

    #[test]
    fn a_visible_key_with_the_same_id_makes_the_certificate_count() {
        let keys = ids(&[&[1], &[2]]);
        assert_eq!(visibility(true, &keys, &[2]), KeyVisibility::Visible);
        assert_eq!(visibility(false, &keys, &[2]), KeyVisibility::Visible);
    }

    #[test]
    fn once_keys_are_visible_certificates_without_one_are_left_out() {
        // CA certificates stored next to the user's certificate on the token.
        let keys = ids(&[&[1]]);
        assert_eq!(visibility(true, &keys, &[9]), KeyVisibility::Absent);
        assert_eq!(visibility(true, &keys, &[]), KeyVisibility::Absent);
    }

    #[test]
    fn hidden_keys_are_assumed_when_a_login_is_required() {
        assert_eq!(visibility(true, &[], &[7]), KeyVisibility::AfterLogin);
    }

    #[test]
    fn no_keys_and_no_login_means_no_key() {
        assert_eq!(visibility(false, &[], &[7]), KeyVisibility::Absent);
    }
}
