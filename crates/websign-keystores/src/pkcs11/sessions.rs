//! Logged-in sessions kept between signatures (`docs/plan.md` D5).
//!
//! PKCS#11 login state belongs to the token, not to a session: while one
//! session of the process stays logged in, every other session on that token
//! is logged in too. So the first successful `C_Login` on a token parks its
//! session here, and later signatures open a fresh session that is already
//! authenticated. Signing never happens on the parked session itself, so a
//! failed or half-finished operation cannot leave it unusable.
//!
//! The PIN is never stored: what is kept is the token's own login.

use cryptoki::session::{Session, SessionState};
use cryptoki::slot::Slot;
use log::trace;

/// One parked, logged-in session per token.
#[derive(Debug, Default)]
pub struct Sessions {
    held: Vec<(Slot, Session)>,
}

impl Sessions {
    /// Keeps `session` (logged in on `slot`) so the token stays unlocked.
    /// A session already parked for the slot is kept instead; both hold the
    /// same login.
    pub fn park(&mut self, slot: Slot, session: Session) {
        if self.held.iter().any(|(held, _)| *held == slot) {
            return;
        }
        trace!(
            "token in slot {} stays unlocked for this session",
            slot.id()
        );
        self.held.push((slot, session));
    }

    /// Whether the token in `slot` is still logged in through a parked
    /// session. A session the module no longer knows (the token was pulled
    /// or reset) is dropped here, so the next signature asks for the PIN.
    pub fn is_unlocked(&mut self, slot: Slot) -> bool {
        let Some(index) = self.held.iter().position(|(held, _)| *held == slot) else {
            return false;
        };
        let alive = self.held[index]
            .1
            .get_session_info()
            .is_ok_and(|info| logged_in(info.session_state()));
        if !alive {
            trace!("parked session of slot {} is gone", slot.id());
            self.held.swap_remove(index);
        }
        alive
    }

    /// Logs every parked session out and closes it.
    pub fn end_all(&mut self) {
        for (slot, session) in self.held.drain(..) {
            trace!("C_Logout on slot {}", slot.id());
            // Best effort: a token that is gone is logged out already, and
            // closing the session below ends the login anyway.
            let _ = session.logout();
        }
    }
}

impl Drop for Sessions {
    fn drop(&mut self) {
        self.end_all();
    }
}

fn logged_in(state: SessionState) -> bool {
    matches!(state, SessionState::RoUser | SessionState::RwUser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_user_states_count_as_logged_in() {
        assert!(logged_in(SessionState::RoUser));
        assert!(logged_in(SessionState::RwUser));
        assert!(!logged_in(SessionState::RoPublic));
        assert!(!logged_in(SessionState::RwPublic));
        assert!(!logged_in(SessionState::RwSecurityOfficer));
    }

    #[test]
    fn nothing_is_unlocked_before_a_login() {
        let mut sessions = Sessions::default();
        let slot = Slot::try_from(1u64).unwrap();
        assert!(!sessions.is_unlocked(slot));
    }
}
