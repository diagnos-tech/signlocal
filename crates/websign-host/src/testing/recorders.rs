//! Ports that record what the engine did.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use websign_protocol::messages::DiagnosticsTab;
use websign_protocol::{AppEnvelope, AppMessage};
use websign_ui_model::confirm::UiCommand;

use crate::ports::{ConfirmUi, KeyCommand, KeyService, Launcher, Outbound, OutboundError};

/// Frames the engine sent.
#[derive(Debug, Clone, Default)]
pub struct FakeOutbound {
    sent: Rc<RefCell<Vec<AppEnvelope>>>,
}

impl FakeOutbound {
    /// Everything sent so far.
    pub fn sent(&self) -> Vec<AppEnvelope> {
        self.sent.borrow().clone()
    }

    /// Everything sent since the last call.
    pub fn take(&self) -> Vec<AppEnvelope> {
        std::mem::take(&mut *self.sent.borrow_mut())
    }

    /// The messages only, without envelopes.
    pub fn messages(&self) -> Vec<AppMessage> {
        self.sent
            .borrow()
            .iter()
            .map(|envelope| envelope.message.clone())
            .collect()
    }
}

impl Outbound for FakeOutbound {
    fn send(&mut self, envelope: &AppEnvelope) -> Result<(), OutboundError> {
        self.sent.borrow_mut().push(envelope.clone());
        Ok(())
    }
}

/// Commands the engine gave the window.
#[derive(Debug, Clone, Default)]
pub struct FakeUi {
    commands: Rc<RefCell<Vec<UiCommand>>>,
    parent: Rc<Cell<Option<isize>>>,
}

impl FakeUi {
    pub fn commands(&self) -> Vec<UiCommand> {
        self.commands.borrow().clone()
    }

    pub fn take(&self) -> Vec<UiCommand> {
        std::mem::take(&mut *self.commands.borrow_mut())
    }

    /// The handle the fake window reports for OS PIN dialogs.
    pub fn set_parent_window(&self, handle: Option<isize>) {
        self.parent.set(handle);
    }
}

impl ConfirmUi for FakeUi {
    fn command(&mut self, command: UiCommand) {
        self.commands.borrow_mut().push(command);
    }

    fn parent_window(&self) -> Option<isize> {
        self.parent.get()
    }
}

/// Commands the engine gave the key store worker.
#[derive(Debug, Clone, Default)]
pub struct FakeKeys {
    commands: Rc<RefCell<Vec<KeyCommand>>>,
}

impl FakeKeys {
    /// Removes and returns the commands sent so far (they are not `Clone`:
    /// a signing command carries the PIN).
    pub fn take(&self) -> Vec<KeyCommand> {
        std::mem::take(&mut *self.commands.borrow_mut())
    }

    pub fn len(&self) -> usize {
        self.commands.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.borrow().is_empty()
    }
}

impl KeyService for FakeKeys {
    fn send(&mut self, command: KeyCommand) {
        self.commands.borrow_mut().push(command);
    }
}

/// Diagnostics launches requested.
#[derive(Debug, Clone, Default)]
pub struct FakeLauncher {
    opened: Rc<RefCell<Vec<Option<DiagnosticsTab>>>>,
    fail: Rc<Cell<bool>>,
}

impl FakeLauncher {
    pub fn opened(&self) -> Vec<Option<DiagnosticsTab>> {
        self.opened.borrow().clone()
    }

    /// Makes the next launches fail, as when the executable is missing.
    pub fn set_failing(&self, fail: bool) {
        self.fail.set(fail);
    }
}

impl Launcher for FakeLauncher {
    fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), String> {
        if self.fail.get() {
            return Err("cannot start the process".to_owned());
        }
        self.opened.borrow_mut().push(tab);
        Ok(())
    }
}
