//! The [`websign_host::ports::ConfirmUi`] implementation: forwards
//! `UiCommand`s to the UI thread and wakes egui; the window posts
//! `UiEvent`s back into the engine's channel.

use std::sync::mpsc::Sender;

use websign_host::ports::ConfirmUi;
use websign_ui_model::confirm::UiCommand;

/// The engine-side handle of the window.
#[derive(Debug)]
pub struct WindowBridge {
    commands: Sender<UiCommand>,
}

impl WindowBridge {
    /// A bridge sending to the UI thread's receiver.
    pub fn new(commands: Sender<UiCommand>) -> WindowBridge {
        WindowBridge { commands }
    }
}

impl ConfirmUi for WindowBridge {
    fn command(&mut self, command: UiCommand) {
        let _ = (command, &self.commands);
        todo!("ui/confirm track")
    }

    fn parent_window(&self) -> Option<isize> {
        todo!("ui/confirm track")
    }
}
