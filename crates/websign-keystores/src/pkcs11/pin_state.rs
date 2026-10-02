//! `pin_state`: what the confirmation window needs to know about a token's
//! PIN before asking for it (`SPEC.md` §3.2), read without logging in.

use cryptoki::context::Pkcs11;
use cryptoki::slot::TokenInfo;

use super::errors::{self, Context};
use super::finder;
use super::locator::Locator;
use super::sessions::Sessions;
use crate::{FoundKey, KeystoreError, PinState};

/// The PIN state of `key`'s token. `always_authenticate` is only known when
/// the private key is readable, that is, when the token shows it without a
/// login or is unlocked; otherwise it reads `false`.
pub fn read(
    pkcs11: &Pkcs11,
    sessions: &mut Sessions,
    key: &FoundKey,
) -> Result<PinState, KeystoreError> {
    let locator = Locator::parse(&key.locator).ok_or(KeystoreError::NotFound)?;
    let located = finder::find_certificate(pkcs11, &locator, &key.cert_der)?;
    let token = pkcs11
        .get_token_info(located.slot)
        .map_err(errors::mapper(Context::default()))?;
    let always_authenticate = finder::find_private_key(&located.session, &located.id)
        .is_ok_and(|private_key| private_key.always_authenticate);
    Ok(PinState {
        unlocked: sessions.is_unlocked(located.slot),
        always_authenticate,
        ..from_token(&token)
    })
}

/// Limits and warnings from `CK_TOKEN_INFO`.
fn from_token(token: &TokenInfo) -> PinState {
    PinState {
        length: length(token.min_pin_length(), token.max_pin_length()),
        count_low: token.user_pin_count_low(),
        final_try: token.user_pin_final_try(),
        locked: token.user_pin_locked(),
        ..PinState::default()
    }
}

/// `(min, max)` when the token states both and they make sense. Tokens
/// report zeros or `CK_UNAVAILABLE_INFORMATION` (all bits set, which is
/// `u32::MAX` where `CK_ULONG` is 32 bits wide) when they do not know.
fn length(min: usize, max: usize) -> Option<(u32, u32)> {
    let stated = |value: usize| u32::try_from(value).ok().filter(|&v| v > 0 && v < u32::MAX);
    let (min, max) = (stated(min)?, stated(max)?);
    (min <= max).then_some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stated_limits_are_reported() {
        assert_eq!(length(4, 8), Some((4, 8)));
        assert_eq!(length(6, 6), Some((6, 6)));
    }

    #[test]
    fn missing_or_inconsistent_limits_are_dropped() {
        assert_eq!(length(0, 8), None);
        assert_eq!(length(4, 0), None);
        assert_eq!(length(9, 8), None);
        assert_eq!(length(4, usize::MAX), None);
        assert_eq!(length(4, u32::MAX as usize), None);
    }
}
