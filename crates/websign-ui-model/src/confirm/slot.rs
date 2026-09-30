//! What the code card is waiting for.

use std::time::{Duration, Instant};

use websign_protocol::VerificationCode;

/// After this long without a digest the card shows a skeleton instead of a
/// bare "Preparing…" that would flash on fast machines.
pub(super) const SKELETON_DELAY: Duration = Duration::from_millis(150);

/// The state of the verification code for the selected certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CodeSlot {
    /// Nothing to show yet, or choose mode.
    None,
    /// A caller that is not remembered has not pressed Continue (D11).
    Hint,
    /// The digest was asked for at this instant.
    Preparing(Instant),
    Ready(VerificationCode),
}
