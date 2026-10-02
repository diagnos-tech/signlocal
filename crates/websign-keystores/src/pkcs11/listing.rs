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
//!   are assumed to be hidden, so every certificate counts except CA
//!   certificates (`basicConstraints cA`), which tokens store next to the
//!   holder's without a key.
//! * If it lists none and requires no login, there is no key.
//!
//! The provider text says which of the three applied.

use cryptoki::context::Pkcs11;
use cryptoki::error::Error;
use cryptoki::slot::{Slot, SlotInfo, TokenInfo};
use websign_core::SourceKind;

use super::device_link::{self, SlotFacts};
use super::errors::{self, Context};
use super::has_key::{counts, visibility};
use super::locator::Locator;
use super::objects::{self, StoredCertificate};
use super::provider::{self, KeyVisibility, PinFlags, TokenFacts};
use crate::{DeviceLink, FoundKey, KeystoreError, PinPrompt};
use log::trace;

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
    let mut readers = Readers::default();
    for slot in slots {
        trace!("{module_file}: reading slot {} without login", slot.id());
        match list_slot(pkcs11, slot, keystore, module_file, &mut readers) {
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

/// The PC/SC reader names, scanned at most once per listing and only when a
/// removable slot needs them.
#[derive(Debug, Default)]
struct Readers(Option<Vec<String>>);

impl Readers {
    fn names(&mut self) -> Vec<String> {
        self.0
            .get_or_insert_with(device_link::pcsc_reader_names)
            .clone()
    }
}

fn list_slot(
    pkcs11: &Pkcs11,
    slot: Slot,
    keystore: &str,
    module_file: &str,
    readers: &mut Readers,
) -> Result<Vec<FoundKey>, Error> {
    let token = pkcs11.get_token_info(slot)?;
    let slot_info = pkcs11.get_slot_info(slot)?;
    let session = pkcs11.open_ro_session(slot)?;
    let certificates = objects::certificates(&session)?;
    let key_ids = objects::visible_private_key_ids(&session);
    let kept: Vec<(StoredCertificate, KeyVisibility)> = certificates
        .into_iter()
        .map(|certificate| {
            let keys = visibility(token.login_required(), &key_ids, &certificate.id);
            (certificate, keys)
        })
        .filter(|(certificate, keys)| counts(*keys, &certificate.der))
        .collect();
    if kept.is_empty() {
        return Ok(Vec::new());
    }
    let device = device(&slot_info, &token, readers);
    Ok(kept
        .into_iter()
        .map(|(certificate, keys)| {
            let found = found_key(
                slot,
                &slot_info,
                &token,
                certificate,
                keys,
                keystore,
                module_file,
            );
            FoundKey {
                device: device.clone(),
                ..found
            }
        })
        .collect())
}

fn device(slot_info: &SlotInfo, token: &TokenInfo, readers: &mut Readers) -> Option<DeviceLink> {
    let facts = SlotFacts {
        model: token.model(),
        manufacturer: token.manufacturer_id(),
        slot_description: slot_info.slot_description(),
        removable: slot_info.removable_device(),
    };
    device_link::link(&facts, || readers.names())
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
        pin: PinFlags {
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
        // Filled by the caller, once per slot.
        device: None,
    }
}
