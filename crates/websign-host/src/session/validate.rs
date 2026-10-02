//! Transport-specific rules on top of strict parsing (`SPEC.md` §2.2).

use websign_protocol::ClientMessage;
use websign_protocol::ErrorCode;
use websign_protocol::WireError;

use super::Transport;

/// Checks `message` against the transport: over native messaging, `hello`
/// needs `browser` and `status`/`choose`/`sign.begin` need `web`; over
/// `websign connect`, both are refused; `diagnostics.open` is accepted on both
/// (the extension never forwards it from pages).
///
/// The caller's identity comes from the transport, so a desktop program
/// that claims a web origin (or a browser that omits it) is refused rather
/// than guessed at.
pub fn validate(transport: &Transport, message: &ClientMessage) -> Result<(), WireError> {
    let native = matches!(transport, Transport::NativeMessaging { .. });
    match message {
        ClientMessage::Hello(hello) => field_rule("browser", native, hello.browser.is_some()),
        ClientMessage::Status(status) => field_rule("web", native, status.web.is_some()),
        ClientMessage::Choose(choose) => field_rule("web", native, choose.web.is_some()),
        ClientMessage::SignBegin(begin) => field_rule("web", native, begin.web.is_some()),
        ClientMessage::SignDigest(_)
        | ClientMessage::Cancel(_)
        | ClientMessage::OpenDiagnostics(_) => Ok(()),
    }
}

/// `field` is required when `required`, refused otherwise.
fn field_rule(field: &str, required: bool, present: bool) -> Result<(), WireError> {
    let message = match (required, present) {
        (true, false) => format!("{field} is required over native messaging"),
        (false, true) => format!("{field} is not allowed over this transport"),
        _ => return Ok(()),
    };
    Err(WireError {
        code: ErrorCode::InvalidRequest,
        message,
        details: None,
    })
}

#[cfg(test)]
mod tests {
    use websign_core::present::caller::DesktopCaller;
    use websign_protocol::ProtocolRange;
    use websign_protocol::messages::{Cancel, Hello, Status};
    use websign_protocol::types::{BrowserInfo, BrowserName, ClientInfo, HelloReason, WebContext};

    use super::*;
    use crate::launch::BrowserLaunch;

    fn native() -> Transport {
        Transport::NativeMessaging {
            launch: BrowserLaunch::manual(),
        }
    }

    fn desktop() -> Transport {
        Transport::Desktop {
            caller: DesktopCaller {
                executable: "/bin/app".into(),
                product_name: None,
                signer: None,
            },
        }
    }

    fn hello(browser: bool) -> ClientMessage {
        ClientMessage::Hello(Hello {
            client: ClientInfo {
                name: "c".into(),
                version: "1".into(),
            },
            protocols: ProtocolRange { min: 1, max: 1 },
            browser: browser.then(|| BrowserInfo {
                name: BrowserName::Chrome,
                version: "1".into(),
                reason: HelloReason::Page,
            }),
        })
    }

    fn status(web: bool) -> ClientMessage {
        ClientMessage::Status(Status {
            web: web.then(|| WebContext {
                origin: "https://a.example".into(),
                top_origin: "https://a.example".into(),
            }),
        })
    }

    #[test]
    fn native_messaging_needs_browser_and_web() {
        assert!(validate(&native(), &hello(true)).is_ok());
        assert!(validate(&native(), &hello(false)).is_err());
        assert!(validate(&native(), &status(true)).is_ok());
        assert!(validate(&native(), &status(false)).is_err());
    }

    #[test]
    fn desktop_refuses_browser_and_web() {
        assert!(validate(&desktop(), &hello(false)).is_ok());
        assert!(validate(&desktop(), &hello(true)).is_err());
        assert!(validate(&desktop(), &status(false)).is_ok());
        let error = validate(&desktop(), &status(true)).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert!(error.message.contains("web"));
    }

    #[test]
    fn continuations_are_allowed_everywhere() {
        let cancel = ClientMessage::Cancel(Cancel {});
        assert!(validate(&native(), &cancel).is_ok());
        assert!(validate(&desktop(), &cancel).is_ok());
    }
}
