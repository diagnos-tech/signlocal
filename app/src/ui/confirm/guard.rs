//! Keystrokes that arrive before the window is armed are dropped
//! (`docs/ux.md` §4.7: "keys and clicks are discarded, except Esc").
//!
//! The window can appear while the person is still typing on the site. Those
//! keys must never become a PIN, an Enter on a button or a selection, so they
//! are removed from the frame's input before any widget reads it, and typed
//! text is wiped first so no copy of it stays in egui's buffers. Esc always
//! passes: leaving is never gated. Key releases pass too, so egui never
//! believes a key is still held. Once the model takes selections again
//! (`ConfirmModel::accepts_selection`: the window has been armed since it
//! gained focus) the list's navigation keys pass as well, because moving
//! the selection approves nothing and re-arms Sign; without that, each
//! arrow would wait 600 ms for the one before it.

use egui::{Event, ImeEvent, InputState, Key};
use zeroize::Zeroize;

/// The keys that only move the list's selection.
const NAVIGATION: [Key; 4] = [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End];

/// Removes this frame's key presses (except Esc, and the navigation keys
/// when `navigation` is true), typed text, pastes and IME compositions,
/// wiping their text.
pub fn drop_keystrokes(input: &mut InputState, navigation: bool) {
    input.events.retain_mut(|event| keep(event, navigation));
    input.raw.events.retain_mut(|event| keep(event, navigation));
}

fn keep(event: &mut Event, navigation: bool) -> bool {
    match event {
        Event::Key {
            key, pressed: true, ..
        } => *key == Key::Escape || (navigation && NAVIGATION.contains(key)),
        Event::Text(text) | Event::Paste(text) => {
            text.zeroize();
            false
        }
        Event::Ime(ImeEvent::Preedit { text, .. } | ImeEvent::Commit(text)) => {
            text.zeroize();
            false
        }
        Event::Ime(_) | Event::Copy | Event::Cut => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: Key, pressed: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Default::default(),
        }
    }

    #[test]
    fn only_escape_and_releases_survive() {
        let mut input = InputState::default();
        input.events = vec![
            Event::Text("1234".into()),
            key(Key::Enter, true),
            key(Key::Space, true),
            key(Key::Escape, true),
            key(Key::Enter, false),
            Event::Paste("99".into()),
            Event::PointerGone,
        ];
        input.raw.events = vec![Event::Text("1234".into()), key(Key::Tab, true)];
        drop_keystrokes(&mut input, false);
        assert_eq!(
            input.events,
            vec![
                key(Key::Escape, true),
                key(Key::Enter, false),
                Event::PointerGone
            ]
        );
        assert!(input.raw.events.is_empty());
    }

    #[test]
    fn navigation_passes_only_when_selections_are_taken() {
        let events = vec![
            key(Key::ArrowDown, true),
            key(Key::End, true),
            key(Key::Enter, true),
            Event::Text("x".into()),
        ];
        let mut input = InputState::default();
        input.events = events.clone();
        drop_keystrokes(&mut input, false);
        assert!(input.events.is_empty());
        input.events = events;
        drop_keystrokes(&mut input, true);
        assert_eq!(
            input.events,
            vec![key(Key::ArrowDown, true), key(Key::End, true)]
        );
    }
}
