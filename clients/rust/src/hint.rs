//! What a developer can do about each error code.

use websign_protocol::ErrorCode;

/// The code's section on the project site (a stable anchor, shared with the
/// web SDK and `@websign/desktop`).
pub(crate) fn docs_url(code: ErrorCode) -> String {
    format!("https://diagnos-tech.github.io/signlocal/developers.html#error-{code:?}")
}

/// Advice for `code`. A `match` without a wildcard: the compiler fails when
/// the protocol gains a code. Kept in step with `@websign/desktop`'s hints.
pub(crate) fn hint(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::ExtensionMissing => {
            "This code is for web pages; desktop programs do not use the browser extension."
        }
        ErrorCode::AppMissing => {
            "Install the SignLocal app, or point `ConnectOptions::executable` (or WEBSIGN_EXECUTABLE) at it. To test without the app, enable the `testing` feature and use `websign_client::testing::FakeApp`."
        }
        ErrorCode::AppOutdated => {
            "The installed app is too old for this request: ask the user to update SignLocal."
        }
        ErrorCode::ExtensionOutdated => {
            "This code is for web pages; update the app and the browser extension together."
        }
        ErrorCode::ClientOutdated => {
            "The app speaks a newer protocol than this crate: upgrade `websign-client`."
        }
        ErrorCode::InsecureOrigin => {
            "This code is for web pages; desktop programs are never refused for their origin."
        }
        ErrorCode::Aborted => {
            "Your `prepare` closure failed, so the request was cancelled in the app. The closure's message is in the error."
        }
        ErrorCode::UserCancelled => {
            "The person closed the window or pressed Cancel. Not a failure to report: offer to retry."
        }
        ErrorCode::Timeout => {
            "Nobody answered in time (the app waits 300 s for the person; `connect` waits 10 s for `hello`). Offer to retry."
        }
        ErrorCode::NoCertificates => {
            "The person has no usable certificate, or closed the window without choosing. Point them to the app's diagnostics (`open_diagnostics`)."
        }
        ErrorCode::CertificateUnavailable => {
            "The certificate is gone: the token was unplugged or the certificate removed. Ask the person to choose again."
        }
        ErrorCode::CertificateNotValid => {
            "The certificate is expired or not yet valid. Ask the person to choose another."
        }
        ErrorCode::InvalidRequest => {
            "The request broke a rule: a digest of the wrong length, another hash or algorithm than asked, or a malformed option. Read the message."
        }
        ErrorCode::UnsupportedAlgorithm => {
            "The chosen key cannot produce that algorithm. Accept several in `SignOptions::algorithms` (preferred first) or filter `certificates`."
        }
        ErrorCode::PinIncorrect => {
            "The PIN was wrong. The app normally retries inside its window; ask the person to try again."
        }
        ErrorCode::PinLocked => {
            "The token's PIN is blocked. The person must unblock it with the issuer's tool (PUK)."
        }
        ErrorCode::TokenRemoved => {
            "The token left while signing. Ask the person to reinsert it and retry."
        }
        ErrorCode::DriverFailure => {
            "The OS key store or the token's driver failed. Ask the person to open Diagnostics in the app."
        }
        ErrorCode::Busy => {
            "Too many requests are waiting in the app. Wait for the current one, then retry."
        }
        ErrorCode::Internal => {
            "A bug in the app or the library, or a broken connection: drop the client and connect again; report it if it repeats."
        }
    }
}
