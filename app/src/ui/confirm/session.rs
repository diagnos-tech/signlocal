//! What the window remembers about the request on screen beyond the model:
//! the PIN being typed, what is expanded and where focus goes next.
//! Everything resets with the request, and the PIN is wiped whenever the
//! model forgets its length.

use websign_core::Fingerprint;
use websign_ui_model::confirm::port::RequestKey;
use zeroize::{Zeroize, Zeroizing};

/// A widget to give keyboard focus to on the next frame (`docs/ux.md` §4.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Pin,
    SelectedRow,
    Row(Fingerprint),
    Primary,
    Cancel,
}

/// Per-request window state.
#[derive(Debug)]
pub struct Session {
    pub key: Option<RequestKey>,
    pub pin: Zeroizing<String>,
    pub pin_shown: bool,
    /// "Can't sign (n)" is expanded.
    pub disabled_open: bool,
    /// "Details" of the selected row is expanded.
    pub details_open: bool,
    /// "Technical details" of the error notice is expanded.
    pub technical_open: bool,
    /// The inline possible-certificate row whose card is expanded.
    pub possible_open: Option<usize>,
    pub filter: String,
    pub focus: Option<FocusTarget>,
    /// The first listing of this request has been placed: initial focus is
    /// chosen once, never again under the person's hands.
    pub placed: bool,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            key: None,
            pin: Zeroizing::new(String::new()),
            pin_shown: false,
            disabled_open: false,
            details_open: false,
            technical_open: false,
            possible_open: None,
            filter: String::new(),
            focus: None,
            placed: false,
        }
    }
}

impl Session {
    /// A new request takes the screen.
    pub fn start(&mut self, key: RequestKey) {
        self.clear_pin();
        *self = Session {
            key: Some(key),
            ..Session::default()
        };
    }

    /// Nothing on screen any more.
    pub fn end(&mut self) {
        self.clear_pin();
        self.key = None;
    }

    /// Wipes the typed PIN in place (its buffer is reused, never freed
    /// unwiped).
    pub fn clear_pin(&mut self) {
        self.pin.zeroize();
        self.pin_shown = false;
    }

    /// Takes the focus request for `target`, if it is the pending one.
    pub fn take_focus(&mut self, target: FocusTarget) -> bool {
        let wanted = self.focus == Some(target);
        if wanted {
            self.focus = None;
        }
        wanted
    }
}
