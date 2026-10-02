//! Reading keystrokes into the PIN field without leaving them behind.
//!
//! egui keeps each frame's typed text twice (`InputState::events` and
//! `InputState::raw.events`) until the next frame replaces it. The field
//! takes its events out of both lists and wipes their text first, so no
//! other widget sees the PIN and egui drops no readable copy. The copies
//! winit and the OS make before egui are out of our reach.
//!
//! Decisions (`docs/ux.md` §4.6):
//! - **No clipboard.** Copy and cut would put the PIN where every program
//!   can read it; paste is refused too, because a PIN that is in the
//!   clipboard has already leaked and should not be encouraged.
//! - **No IME.** The field never asks for an input method, so composition
//!   windows and IME dictionaries never see or learn the PIN; typed
//!   characters arrive as plain text events.

use egui::{Event, ImeEvent, InputState, Key};
use zeroize::{Zeroize, Zeroizing};

use super::pin_buffer;

/// What the frame's keystrokes did to the PIN.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Typed {
    pub changed: bool,
    /// Enter was pressed.
    pub submitted: bool,
}

/// Applies this frame's typing, Backspace and Enter to `pin` (at most
/// `max_chars`), removing those events and wiping their text.
pub fn read(input: &mut InputState, pin: &mut Zeroizing<String>, max_chars: usize) -> Typed {
    let mut typed = Typed::default();
    input.events.retain_mut(|event| match event {
        Event::Text(text) => {
            typed.changed |= pin_buffer::push(pin, text, max_chars);
            text.zeroize();
            false
        }
        Event::Paste(text) => {
            text.zeroize();
            false
        }
        Event::Ime(ime) => {
            wipe_ime(ime);
            false
        }
        Event::Copy | Event::Cut => false,
        Event::Key {
            key: Key::Backspace,
            pressed: true,
            ..
        } => {
            typed.changed |= pin_buffer::pop(pin);
            false
        }
        Event::Key {
            key: Key::Enter,
            pressed: true,
            ..
        } => {
            typed.submitted = true;
            false
        }
        _ => true,
    });
    input.raw.events.retain_mut(|event| match event {
        Event::Text(text) | Event::Paste(text) => {
            text.zeroize();
            false
        }
        Event::Ime(ime) => {
            wipe_ime(ime);
            false
        }
        _ => true,
    });
    typed
}

/// A composition left over from a field that had the IME on (the filter):
/// dropped unread, never typed into the PIN.
fn wipe_ime(ime: &mut ImeEvent) {
    match ime {
        ImeEvent::Preedit { text, .. } | ImeEvent::Commit(text) => text.zeroize(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_and_wipes_typed_text_and_refuses_the_clipboard() {
        let mut input = InputState::default();
        input.events = vec![
            Event::Text("12".into()),
            Event::Paste("99".into()),
            Event::Copy,
            Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            },
            Event::PointerGone,
        ];
        input.raw.events = vec![Event::Text("12".into())];
        let mut pin = Zeroizing::new(String::new());
        let typed = read(&mut input, &mut pin, 8);
        assert_eq!(pin.as_str(), "12");
        assert_eq!(
            typed,
            Typed {
                changed: true,
                submitted: true
            }
        );
        assert_eq!(input.events, vec![Event::PointerGone]);
        assert!(input.raw.events.is_empty());
    }
}
