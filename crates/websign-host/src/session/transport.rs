//! The two transports and what each guarantees about the caller.

use websign_core::present::caller::DesktopCaller;

use crate::launch::BrowserLaunch;

/// How this process was reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    /// A browser started us for our extension. The browser checked the
    /// extension ID against the manifest's `allowed_origins`/
    /// `allowed_extensions`; requests carry the page origin in `web`.
    NativeMessaging { launch: BrowserLaunch },
    /// `websign connect` (or `sign`/`choose`) started by a program. The
    /// caller was identified from the parent process before the first frame;
    /// requests must not carry `web`.
    Desktop { caller: DesktopCaller },
}
