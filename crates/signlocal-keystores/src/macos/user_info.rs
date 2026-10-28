//! PIN attempts left, as a token driver may attach them to a `CFError`.
//!
//! CryptoTokenKit reports a blocked PIN as `TKErrorCodeAuthenticationFailed`,
//! the same code as a wrong PIN; only the `userInfo` can tell them apart.
//! Apple documents no key for the count, so any numeric entry whose key
//! speaks of attempts or retries is taken.

use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::error::{CFError, CFErrorCopyUserInfo};
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;

/// The attempts left reported in `error`'s `userInfo`, if any.
pub fn remaining_attempts(error: &CFError) -> Option<i64> {
    // SAFETY: `error` keeps the CFErrorRef alive for the call. The result
    // follows the Copy rule and may be NULL, which is checked before wrapping.
    let raw = unsafe { CFErrorCopyUserInfo(error.as_concrete_TypeRef()) };
    if raw.is_null() {
        return None;
    }
    // SAFETY: `raw` is a non-NULL CFDictionary we own; the wrapper takes over
    // that single reference. Keys and values are wrapped as untyped CFType
    // and only then downcast, so no type is assumed.
    let info: CFDictionary<CFType, CFType> = unsafe { CFDictionary::wrap_under_create_rule(raw) };
    let (keys, values) = info.get_keys_and_values();
    let entries = keys.into_iter().zip(values).filter_map(|(key, value)| {
        // SAFETY: both pointers come from `info`, which is alive and keeps
        // them alive; the Get rule adds the reference each wrapper releases.
        let (key, value) = unsafe {
            (
                CFType::wrap_under_get_rule(key),
                CFType::wrap_under_get_rule(value),
            )
        };
        let name = key.downcast::<CFString>()?.to_string();
        let count = value.downcast::<CFNumber>()?.to_i64()?;
        Some((name, count))
    });
    pick(entries)
}

/// The count of the first entry named like an attempts counter.
fn pick(entries: impl IntoIterator<Item = (String, i64)>) -> Option<i64> {
    entries
        .into_iter()
        .find(|(name, _)| names_attempts(name))
        .map(|(_, count)| count)
}

fn names_attempts(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    ["attempt", "retr", "tries"]
        .iter()
        .any(|word| key.contains(word))
}

#[cfg(test)]
mod tests {
    use super::pick;

    #[test]
    fn finds_counters_by_name() {
        let entries = [
            ("NSLocalizedDescription".to_owned(), 7),
            ("RemainingAttempts".to_owned(), 0),
        ];
        assert_eq!(pick(entries), Some(0));
        assert_eq!(pick([("pinRetries".to_owned(), 2)]), Some(2));
    }

    #[test]
    fn ignores_unrelated_numbers() {
        assert_eq!(pick([("code".to_owned(), 0)]), None);
    }
}
