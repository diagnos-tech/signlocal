//! Which error a caller receives when the person closes the window
//! (`docs/ux.md` §15): the code of the last blocking condition on screen
//! (`NoCertificates`, `PinLocked`, `CertificateUnavailable`), else
//! `UserCancelled`.

use websign_protocol::ErrorCode;

use super::machine::ConfirmState;

/// The code for closing in `state`.
pub fn cancel_code(state: &ConfirmState) -> ErrorCode {
    let _ = state;
    todo!("SPEC.md §2.3")
}
