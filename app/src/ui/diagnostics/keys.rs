//! Window shortcuts (`docs/ux.md` §8.8): Ctrl/⌘+1…4 switch tab, F5 or
//! Ctrl/⌘+R scans again, Ctrl/⌘+Shift+C copies the diagnostics, Ctrl/⌘+W
//! closes. ↑/↓ between tabs is handled by the sidebar, where focus is known.

use egui::{Context, Key, KeyboardShortcut, Modifiers};

use super::state::{Action, TABS};

const TAB_KEYS: [Key; 4] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4];

/// The actions of the shortcuts pressed this frame (consumed).
pub fn shortcuts(ctx: &Context) -> Vec<Action> {
    let copy = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::C);
    let mut actions = Vec::new();
    ctx.input_mut(|input| {
        // The window system may turn Ctrl+Shift+C into a copy event instead
        // of a key press; nothing in this window is selectable text, so a
        // copy with Shift held is this shortcut.
        let copy_event = input.modifiers.shift
            && input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Copy));
        if input.consume_shortcut(&copy) || copy_event {
            actions.push(Action::CopyReport);
        }
        for (key, tab) in TAB_KEYS.into_iter().zip(TABS) {
            if input.consume_key(Modifiers::COMMAND, key) {
                actions.push(Action::SelectTab(tab));
            }
        }
        if input.consume_key(Modifiers::NONE, Key::F5)
            || input.consume_key(Modifiers::COMMAND, Key::R)
        {
            actions.push(Action::Rescan);
        }
        if input.consume_key(Modifiers::COMMAND, Key::W) {
            actions.push(Action::Close);
        }
    });
    actions
}
