//! Keeping Enter from acting as a click.
//!
//! egui turns Enter *and* Space on a focused widget into a click. In the
//! confirmation window that is unsafe: the window can appear while the
//! person is still typing on the site, and `docs/ux.md` §4.4 and §4.9 say
//! Enter never releases a certificate (Continue), never chooses one and, on
//! a list row, only moves focus. So the buttons and rows here take Enter out
//! of the frame's input before egui sees it and report it separately; the
//! window decides what it means (`websign_ui_model::confirm::UserInput`).
//! Space and pointer clicks keep working as clicks.

use egui::{Event, Id, Key, Ui};

/// Removes this frame's Enter presses when the widget `id` has keyboard
/// focus, and returns whether there was one. Call it before `ui.interact`
/// for `id`, so egui's own "Enter is a click" never fires.
pub fn take_enter(ui: &Ui, id: Id) -> bool {
    if !ui.memory(|memory| memory.has_focus(id)) {
        return false;
    }
    ui.input_mut(|input| {
        let before = input.events.len();
        input.events.retain(|event| !is_enter_press(event));
        before != input.events.len()
    })
}

fn is_enter_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Key {
            key: Key::Enter,
            pressed: true,
            ..
        }
    )
}
