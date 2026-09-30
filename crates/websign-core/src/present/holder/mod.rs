//! The holder name a person recognizes (`docs/ux.md` §5.2, vectors §16.3).

mod title_case;

pub use title_case::title_case;

use crate::cert::CertInfo;

/// Length of the fingerprint prefix [`CertInfo::display_name`] falls back to.
const FALLBACK_LEN: usize = 16;

/// The name to show for `info`: [`CertInfo::display_name`] (ICP-Brasil holder
/// without the document suffix, CN, O, or a fingerprint prefix), then, when
/// it has letters and all of them are uppercase, [`title_case`].
///
/// Before falling back to the fingerprint, `givenName` + `surname` is tried:
/// eIDAS personal certificates often carry no CN.
pub fn display_name(info: &CertInfo) -> String {
    let name = info.display_name();
    let name = if is_fingerprint_fallback(info, &name) {
        given_and_surname(info).unwrap_or(name)
    } else {
        name
    };
    if is_all_uppercase(&name) {
        title_case(&name)
    } else {
        name
    }
}

fn is_fingerprint_fallback(info: &CertInfo, name: &str) -> bool {
    let hex = info.fingerprint.to_hex();
    hex.get(..FALLBACK_LEN) == Some(name)
}

fn given_and_surname(info: &CertInfo) -> Option<String> {
    fn usable(text: &Option<String>) -> Option<&str> {
        text.as_deref().map(str::trim).filter(|t| !t.is_empty())
    }
    let given = usable(&info.subject.given_name)?;
    let surname = usable(&info.subject.surname)?;
    Some(format!("{given} {surname}"))
}

/// At least one letter, and no lowercase one.
fn is_all_uppercase(text: &str) -> bool {
    let mut letters = text.chars().filter(|c| c.is_alphabetic()).peekable();
    letters.peek().is_some() && letters.all(char::is_uppercase)
}
