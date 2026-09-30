//! Which error a caller receives when the person closes the window
//! (`docs/ux.md` §15): the code of the last blocking condition on screen
//! (`NoCertificates`, `PinLocked`, `CertificateUnavailable`), else
//! `UserCancelled`.

use websign_protocol::ErrorCode;

use super::machine::ConfirmState;

/// The code for closing in `state`.
pub fn cancel_code(state: &ConfirmState) -> ErrorCode {
    match state {
        ConfirmState::Empty => ErrorCode::NoCertificates,
        ConfirmState::PinLocked => ErrorCode::PinLocked,
        ConfirmState::Error {
            code: ErrorCode::CertificateUnavailable,
        } => ErrorCode::CertificateUnavailable,
        _ => ErrorCode::UserCancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockers_map_to_their_codes() {
        assert_eq!(cancel_code(&ConfirmState::Empty), ErrorCode::NoCertificates);
        assert_eq!(cancel_code(&ConfirmState::PinLocked), ErrorCode::PinLocked);
        let unavailable = ConfirmState::Error {
            code: ErrorCode::CertificateUnavailable,
        };
        assert_eq!(cancel_code(&unavailable), ErrorCode::CertificateUnavailable);
    }

    #[test]
    fn everything_else_is_user_cancelled() {
        let other = ConfirmState::Error {
            code: ErrorCode::DriverFailure,
        };
        for state in [
            ConfirmState::LoadingCerts,
            ConfirmState::Choosing,
            ConfirmState::Ready,
            ConfirmState::PinError,
            ConfirmState::Signing,
            other,
        ] {
            assert_eq!(cancel_code(&state), ErrorCode::UserCancelled);
        }
    }
}
