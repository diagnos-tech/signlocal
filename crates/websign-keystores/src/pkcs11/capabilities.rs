//! What a token can sign with: `CKM_RSA_PKCS`, `CKM_RSA_PKCS_PSS` and
//! `CKM_ECDSA` with `CKF_SIGN`, asked of the slot without a login.
//!
//! Many tokens (older SafeNet and Gemalto cards, most CCID PIV applets) have
//! no `CKM_RSA_PKCS_PSS`; offering PSS for them would fail only after the
//! certificate was released and the PIN typed.

use std::collections::HashMap;

use cryptoki::context::Pkcs11;
use cryptoki::mechanism::MechanismType;
use cryptoki::slot::Slot;
use log::trace;
use websign_core::SignatureAlgorithm;

use super::key_checks::{MechanismAnswer, mechanism_answer};
use super::locator::Locator;
use crate::{FoundKey, KeyCapabilities};

/// Capabilities by slot, read once per listing: a slot's token (and with it
/// its mechanisms) can only change between listings.
#[derive(Debug, Default)]
pub struct SlotCapabilities(HashMap<u64, KeyCapabilities>);

impl SlotCapabilities {
    /// Forgets every slot; called when the module lists again.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// What the slot of `key`'s locator can sign with; nothing for a
    /// locator that names no slot.
    pub fn of(&mut self, pkcs11: &Pkcs11, key: &FoundKey) -> KeyCapabilities {
        let Some(locator) = Locator::parse(&key.locator) else {
            return KeyCapabilities::NONE;
        };
        let Ok(slot) = Slot::try_from(locator.slot) else {
            return KeyCapabilities::NONE;
        };
        *self
            .0
            .entry(locator.slot)
            .or_insert_with(|| read(pkcs11, slot))
    }
}

/// The mechanism that produces `algorithm` from a digest (`mechanism.rs`).
pub fn mechanism_of(algorithm: SignatureAlgorithm) -> MechanismType {
    match algorithm {
        SignatureAlgorithm::Ecdsa => MechanismType::ECDSA,
        SignatureAlgorithm::RsaPkcs1v15 => MechanismType::RSA_PKCS,
        SignatureAlgorithm::RsaPss => MechanismType::RSA_PKCS_PSS,
    }
}

fn read(pkcs11: &Pkcs11, slot: Slot) -> KeyCapabilities {
    trace!(
        "slot {}: C_GetMechanismList + C_GetMechanismInfo",
        slot.id()
    );
    let listed = pkcs11.get_mechanism_list(slot).ok();
    decide(listed.as_deref(), |mechanism| {
        mechanism_answer(pkcs11, slot, mechanism)
    })
}

/// `C_GetMechanismInfo` decides; when it cannot answer, membership in the
/// mechanism list does, and without a list the mechanism is assumed, as the
/// signing side does (`key_checks::require_mechanism`).
fn decide(
    listed: Option<&[MechanismType]>,
    mut answer: impl FnMut(MechanismType) -> MechanismAnswer,
) -> KeyCapabilities {
    KeyCapabilities::from_fn(|algorithm| {
        let mechanism = mechanism_of(algorithm);
        match answer(mechanism) {
            MechanismAnswer::Signs => true,
            MechanismAnswer::Refuses => false,
            MechanismAnswer::Unknown => listed.is_none_or(|list| list.contains(&mechanism)),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

    #[test]
    fn a_token_without_pss_offers_only_pkcs1_and_ecdsa() {
        let capabilities = decide(None, |mechanism| {
            if mechanism == MechanismType::RSA_PKCS_PSS {
                MechanismAnswer::Refuses
            } else {
                MechanismAnswer::Signs
            }
        });
        assert_eq!(capabilities.algorithms(), [Ecdsa, RsaPkcs1v15]);
    }

    #[test]
    fn unknown_answers_fall_back_to_the_mechanism_list() {
        let listed = [MechanismType::RSA_PKCS];
        let capabilities = decide(Some(&listed), |_| MechanismAnswer::Unknown);
        assert_eq!(capabilities.algorithms(), [RsaPkcs1v15]);
        let capabilities = decide(None, |_| MechanismAnswer::Unknown);
        assert_eq!(capabilities, KeyCapabilities::ALL);
    }

    #[test]
    fn the_info_answer_wins_over_the_list() {
        let capabilities = decide(Some(&[]), |mechanism| {
            if mechanism == MechanismType::RSA_PKCS_PSS {
                MechanismAnswer::Signs
            } else {
                MechanismAnswer::Refuses
            }
        });
        assert_eq!(capabilities.algorithms(), [RsaPss]);
    }

    #[test]
    fn each_algorithm_maps_to_the_mechanism_that_signs_it() {
        for algorithm in SignatureAlgorithm::ALL {
            let hash = websign_core::HashAlgorithm::Sha256;
            let plan = super::super::mechanism::plan(hash, algorithm, &hash.digest(b"x")).unwrap();
            assert_eq!(
                plan.mechanism_type(),
                mechanism_of(algorithm),
                "{algorithm}"
            );
        }
    }
}
