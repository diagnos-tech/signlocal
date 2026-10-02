//! Rules the types cannot express, checked right after the body parses
//! (`SPEC.md` §5 step 7). Only what the frame alone decides: a digest's
//! length depends on its request's `hash`, so the app checks that.

use crate::limits::{MAX_CHAIN_LEN, MAX_ORIGIN_LEN, MAX_SHORT_TEXT_LEN};
use crate::messages::{AppMessage, ClientMessage};
use crate::types::{Certificate, SignatureAlgorithmName, WebContext};

/// A broken rule: the field's path and what is wrong, never its value.
pub(super) type Violation = String;

/// Checks a client message.
pub(super) fn check_client(message: &ClientMessage) -> Result<(), Violation> {
    match message {
        ClientMessage::Hello(hello) => {
            short_text("client.name", &hello.client.name)?;
            short_text("client.version", &hello.client.version)?;
            match &hello.browser {
                Some(browser) => short_text("browser.version", &browser.version),
                None => Ok(()),
            }
        }
        ClientMessage::Status(status) => web(status.web.as_ref()),
        ClientMessage::Choose(choose) => {
            web(choose.web.as_ref())?;
            let algorithms = choose.filter.as_ref().and_then(|f| f.algorithms.as_ref());
            not_empty("filter.algorithms", algorithms)
        }
        ClientMessage::SignBegin(begin) => {
            web(begin.web.as_ref())?;
            not_empty("algorithms", begin.algorithms.as_ref())
        }
        ClientMessage::SignDigest(_)
        | ClientMessage::Cancel(_)
        | ClientMessage::OpenDiagnostics(_) => Ok(()),
    }
}

/// Checks an app message.
pub(super) fn check_app(message: &AppMessage) -> Result<(), Violation> {
    match message {
        AppMessage::ChooseResult(result) if result.certificates.is_empty() => {
            Err("field certificates must hold at least one certificate".to_owned())
        }
        AppMessage::ChooseResult(result) => result.certificates.iter().try_for_each(chain),
        AppMessage::NeedDigest(need) => chain(&need.certificate),
        AppMessage::SignResult(result) => chain(&result.certificate),
        AppMessage::Hello(_)
        | AppMessage::Status(_)
        | AppMessage::Done(_)
        | AppMessage::Error(_) => Ok(()),
    }
}

fn web(web: Option<&WebContext>) -> Result<(), Violation> {
    let Some(web) = web else { return Ok(()) };
    for (field, origin) in [
        ("web.origin", &web.origin),
        ("web.topOrigin", &web.top_origin),
    ] {
        if origin.len() > MAX_ORIGIN_LEN {
            return Err(format!("field {field} exceeds {MAX_ORIGIN_LEN} bytes"));
        }
    }
    Ok(())
}

fn short_text(field: &str, text: &str) -> Result<(), Violation> {
    if text.len() > MAX_SHORT_TEXT_LEN {
        return Err(format!("field {field} exceeds {MAX_SHORT_TEXT_LEN} bytes"));
    }
    Ok(())
}

/// An empty list would disable every certificate; absent means "any".
fn not_empty(
    field: &str,
    algorithms: Option<&Vec<SignatureAlgorithmName>>,
) -> Result<(), Violation> {
    match algorithms {
        Some(list) if list.is_empty() => Err(format!("field {field} must not be empty")),
        _ => Ok(()),
    }
}

fn chain(certificate: &Certificate) -> Result<(), Violation> {
    if certificate.chain.len() > MAX_CHAIN_LEN {
        return Err(format!("field chain exceeds {MAX_CHAIN_LEN} certificates"));
    }
    Ok(())
}
