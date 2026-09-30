//! Who is asking, established by the transport — never by the payload.

use websign_core::present::caller::{DesktopCaller, consent_key};
use websign_core::present::origin::{FormattedOrigin, OriginError, format_origin};
use websign_protocol::types::{BrowserInfo, WebContext};

/// The requester of one message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    /// A page, through our extension. `origin` comes from the browser's
    /// `MessageSender` via the extension.
    Web {
        origin: FormattedOrigin,
        /// Set when the frame's origin differs from the tab's.
        top: Option<FormattedOrigin>,
        browser: BrowserInfo,
    },
    /// A desktop program that started `websign connect` (or `sign`/`choose`).
    Desktop(DesktopCaller),
}

impl Caller {
    /// The web caller of `context`, refusing insecure or malformed origins
    /// (defense in depth: the extension refuses them first).
    pub fn web(context: &WebContext, browser: &BrowserInfo) -> Result<Caller, OriginError> {
        let origin = format_origin(&context.origin)?;
        let top = if context.top_origin == context.origin {
            None
        } else {
            Some(format_origin(&context.top_origin)?)
        };
        Ok(Caller::Web {
            origin,
            top,
            browser: browser.clone(),
        })
    }

    /// The key consent is stored under: the canonical origin for the web,
    /// `app:`/`path:` keys for programs.
    pub fn consent_key(&self) -> String {
        match self {
            Caller::Web { origin, .. } => origin.canonical.clone(),
            Caller::Desktop(program) => consent_key(program),
        }
    }

    /// Whether "Remember" may be offered (not for IP or IDN origins).
    pub fn can_remember(&self) -> bool {
        match self {
            Caller::Web { origin, .. } => origin.can_remember,
            Caller::Desktop(_) => true,
        }
    }
}
