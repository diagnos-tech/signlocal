//! Finding, at signing time, the certificate and private key that `list`
//! reported.
//!
//! Slot numbers change when readers are plugged in a different order, so the
//! locator's slot is only tried first. What identifies the key is the
//! certificate's DER, and what links it to the private key is `CKA_ID`.

use cryptoki::context::Pkcs11;
use cryptoki::error::Error;
use cryptoki::object::{Attribute, AttributeType, KeyType, ObjectClass, ObjectHandle};
use cryptoki::session::Session;
use cryptoki::slot::Slot;

use super::errors::{self, Context};
use super::locator::Locator;
use super::objects;
use crate::keystores::KeystoreError;

/// A read-only session on the slot that holds the certificate.
#[derive(Debug)]
pub struct Located {
    pub slot: Slot,
    pub session: Session,
    /// `CKA_ID` as stored now (it may differ from the locator's if the token was reissued).
    pub id: Vec<u8>,
}

/// Opens the slot that holds `cert_der`: the locator's slot if it still does,
/// otherwise the first other slot with a token that does.
pub fn find_certificate(
    pkcs11: &Pkcs11,
    locator: &Locator,
    cert_der: &[u8],
) -> Result<Located, KeystoreError> {
    let mut slots: Vec<Slot> = Slot::try_from(locator.slot).into_iter().collect();
    let mapper = errors::mapper(Context::default());
    for slot in pkcs11.get_slots_with_token().map_err(&mapper)? {
        if !slots.contains(&slot) {
            slots.push(slot);
        }
    }

    let mut first_error = None;
    for slot in slots {
        match holds_certificate(pkcs11, slot, cert_der) {
            Ok(Some(located)) => return Ok(located),
            Ok(None) => {}
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    Err(first_error.map_or(KeystoreError::NotFound, |error| {
        errors::map(error, Context::default())
    }))
}

fn holds_certificate(
    pkcs11: &Pkcs11,
    slot: Slot,
    cert_der: &[u8],
) -> Result<Option<Located>, Error> {
    let session = pkcs11.open_ro_session(slot)?;
    let certificate = objects::certificates(&session)?
        .into_iter()
        .find(|certificate| certificate.der == cert_der);
    Ok(certificate.map(|certificate| Located {
        slot,
        session,
        id: certificate.id,
    }))
}

/// The private key to sign with, and what it demands.
#[derive(Debug, Clone, Copy)]
pub struct PrivateKey {
    pub handle: ObjectHandle,
    pub key_type: Option<KeyType>,
    /// `CKA_ALWAYS_AUTHENTICATE`: the PIN must be entered again for every
    /// signature (qualified signature keys of eIDAS cards).
    pub always_authenticate: bool,
}

/// The private key with `CKA_ID` = `id`, which must be visible (log in first).
/// When several share the id, the first that may sign wins.
pub fn find_private_key(session: &Session, id: &[u8]) -> Result<PrivateKey, KeystoreError> {
    if id.is_empty() {
        return Err(KeystoreError::Unsupported(
            "the certificate has no CKA_ID, so it cannot be linked to a private key".to_owned(),
        ));
    }
    let mapper = errors::mapper(Context::default());
    let template = [
        Attribute::Class(ObjectClass::PRIVATE_KEY),
        Attribute::Id(id.to_vec()),
    ];
    let candidates = session.find_objects(&template).map_err(&mapper)?;
    let wanted = [
        AttributeType::Sign,
        AttributeType::KeyType,
        AttributeType::AlwaysAuthenticate,
    ];

    let mut described = Vec::new();
    for handle in candidates {
        let mut key = PrivateKey {
            handle,
            key_type: None,
            always_authenticate: false,
        };
        let mut may_sign = true;
        for attribute in session.get_attributes(handle, &wanted).unwrap_or_default() {
            match attribute {
                Attribute::Sign(value) => may_sign = value,
                Attribute::KeyType(kind) => key.key_type = Some(kind),
                Attribute::AlwaysAuthenticate(value) => key.always_authenticate = value,
                _ => {}
            }
        }
        described.push((may_sign, key));
    }
    described
        .into_iter()
        .find(|(may_sign, _)| *may_sign)
        .map(|(_, key)| key)
        .ok_or(KeystoreError::NotFound)
}
