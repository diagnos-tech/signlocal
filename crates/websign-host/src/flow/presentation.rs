//! What the window needs to show about who asks and where the request
//! stands, which only the engine knows (the flows never see the caller or
//! the queue).

use websign_core::Fingerprint;
use websign_protocol::limits::DECISION_TIMEOUT;
use websign_ui_model::confirm::port::{CallerView, Mode, OpenRequest, RequestKey};

/// The engine's part of an [`OpenRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    pub caller: CallerView,
    /// "Remember this site" may be offered.
    pub can_remember: bool,
    /// 1-based position and queue length.
    pub position: (u32, u32),
}

impl Presentation {
    /// The command that puts the request on screen.
    pub(crate) fn open(
        &self,
        key: RequestKey,
        mode: Mode,
        remembered: bool,
        consented: Vec<Fingerprint>,
    ) -> OpenRequest {
        OpenRequest {
            key,
            mode,
            caller: self.caller.clone(),
            remembered,
            consented,
            can_remember: self.can_remember,
            position: self.position,
            timeout_secs: u32::try_from(DECISION_TIMEOUT.as_secs()).unwrap_or(u32::MAX),
        }
    }
}
