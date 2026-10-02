//! The main thread of a host process: idle until the engine first needs the
//! window, then the confirmation window's event loop until the engine ends.

use std::sync::mpsc::Receiver;

use websign_host::EngineEvent;
use websign_host::runtime::EventSender;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::port::RequestKey;
use websign_ui_model::confirm::{UiCommand, UiEvent};

use crate::ui::renderer;

/// Waits for the first command that shows something and hands the channel
/// to the window. Returns when the engine has dropped its end (the
/// connection is over); a connection that never shows a window never starts
/// the UI toolkit, so `status` and remembered `choose` stay fast and work
/// without a display.
pub fn run(inbox: Receiver<UiCommand>, events: Receiver<EventSender>) {
    let Some(first) = inbox
        .iter()
        .find(|command| !matches!(command, UiCommand::Hide))
    else {
        return;
    };
    let Ok(events) = events.recv() else {
        return;
    };
    log::info!(
        "opening the confirmation window ({})",
        renderer::choose().name()
    );
    if let Err(error) = crate::ui::confirm::run(first.clone(), &inbox, events.clone()) {
        window_failed(&error, first, &inbox, &events);
    }
}

/// Without a window nothing can be confirmed: every request that reaches
/// the screen fails with `Internal`, so callers get an answer instead of a
/// timeout. Stdin is already read, so re-executing with glow would lose the
/// request (`ui::renderer`): a renderer failure is remembered for the next
/// launch instead.
fn window_failed(
    error: &eframe::Error,
    first: UiCommand,
    inbox: &Receiver<UiCommand>,
    events: &EventSender,
) {
    log::error!("the confirmation window could not open: {error}");
    if renderer::is_renderer_failure(error) && renderer::may_fall_back_to_glow() {
        renderer::remember_glow_needed();
    }
    for command in std::iter::once(first).chain(inbox.iter()) {
        if let UiCommand::Open(request) = command {
            refuse(request.key, events);
        }
    }
}

fn refuse(key: RequestKey, events: &EventSender) {
    let code = ErrorCode::Internal;
    let _ = events.send(EngineEvent::Ui(UiEvent::Cancel { key, code }));
}
