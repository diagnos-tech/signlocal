//! `websign:` URLs (`docs/architecture/desktop-api.md` §8).
//!
//! Any web page can open a `websign:` URL, so a URL never carries data the
//! app acts on: it names one of the documented actions, and anything else
//! only opens diagnostics.

/// What a `websign:` URL asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlAction {
    /// `websign:activate`: register with every browser, then diagnostics.
    Activate,
    /// Any other `websign:` URL: open diagnostics, nothing else.
    Diagnostics,
}

/// Longest URL looked at; the only real one is 16 bytes.
const MAX_LEN: usize = 256;

/// `Some` when `arg` is a `websign:` URL (scheme compared case-insensitively,
/// as the OS may pass it either way).
pub fn parse(arg: &str) -> Option<UrlAction> {
    let scheme = websign_project::URL_SCHEME;
    let rest = arg
        .get(..scheme.len())
        .filter(|s| s.eq_ignore_ascii_case(scheme))
        .and_then(|_| arg[scheme.len()..].strip_prefix(':'))?;
    if arg.len() > MAX_LEN {
        return Some(UrlAction::Diagnostics);
    }
    // Browsers normalize `websign:activate` to `websign://activate/` in
    // some versions; both spellings are the same action, nothing more.
    let action = rest.strip_prefix("//").unwrap_or(rest);
    let action = action.strip_suffix('/').unwrap_or(action);
    Some(if action.eq_ignore_ascii_case("activate") {
        UrlAction::Activate
    } else {
        UrlAction::Diagnostics
    })
}
