//! The holder name a person recognizes (`docs/ux.md` §5.2, vectors §16.3).

mod title_case;

pub use title_case::title_case;

use super::visible::visible;
use crate::cert::CertInfo;

/// Hex digits of the fingerprint shown when the certificate names no one
/// (the same prefix as [`CertInfo::display_name`]).
const FINGERPRINT_CHARS: usize = 16;

/// The name to show for `info`.
///
/// The first non-blank of the ICP-Brasil holder (CN without its document
/// suffix), the CN, `givenName` + `surname`, and the organization, without
/// control or bidirectional formatting characters, and in [`title_case`] when
/// it has letters and all of them are uppercase (certificate authorities
/// often issue names in capitals; people do not write them that way).
/// Without any (or when nothing visible is left), the fingerprint prefix,
/// untouched.
pub fn display_name(info: &CertInfo) -> String {
    let Some(name) = info
        .name_candidate()
        .map(|name| visible(&name))
        .filter(|name| !name.trim().is_empty())
    else {
        return info
            .fingerprint
            .to_hex()
            .chars()
            .take(FINGERPRINT_CHARS)
            .collect();
    };
    if is_all_uppercase(&name) {
        title_case(&name)
    } else {
        name
    }
}

/// At least one letter, and no lowercase one.
fn is_all_uppercase(text: &str) -> bool {
    let mut letters = text.chars().filter(|c| c.is_alphabetic()).peekable();
    letters.peek().is_some() && letters.all(char::is_uppercase)
}
