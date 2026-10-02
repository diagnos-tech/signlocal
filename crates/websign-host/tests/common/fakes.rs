//! Recording fakes for the engine's ports and a manual clock.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use websign_host::ports::{
    Clock, ConfirmUi, KeyCommand, KeyService, Launcher, Outbound, OutboundError,
};
use websign_protocol::AppEnvelope;
use websign_protocol::messages::DiagnosticsTab;
use websign_ui_model::confirm::UiCommand;

/// 2026-09-29 12:00 UTC, inside the validity of every fixture certificate.
pub const WALL_START: u64 = 1_790_683_200;

/// The native handle the fake window reports.
pub const PARENT_WINDOW: isize = 4242;

/// Everything the engine sent through a port since the last drain.
#[derive(Default)]
pub struct Recorded {
    pub frames: Vec<AppEnvelope>,
    pub ui: Vec<UiCommand>,
    pub keys: Vec<KeyCommand>,
    pub diagnostics: Vec<Option<DiagnosticsTab>>,
    /// Writes fail, as when the browser closed the port.
    pub client_gone: bool,
    /// The launcher reports an error.
    pub launcher_fails: bool,
}

pub type Shared = Rc<RefCell<Recorded>>;

/// A clock that moves only when the test says so.
#[derive(Clone)]
pub struct ManualClock {
    base: Instant,
    offset: Rc<Cell<Duration>>,
}

impl ManualClock {
    pub fn new() -> ManualClock {
        ManualClock {
            base: Instant::now(),
            offset: Rc::new(Cell::new(Duration::ZERO)),
        }
    }

    pub fn advance(&self, by: Duration) {
        self.offset.set(self.offset.get() + by);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.base + self.offset.get()
    }

    fn wall(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(WALL_START) + self.offset.get()
    }
}

pub struct FakeOutbound(pub Shared);

impl Outbound for FakeOutbound {
    fn send(&mut self, envelope: &AppEnvelope) -> Result<(), OutboundError> {
        let mut rec = self.0.borrow_mut();
        if rec.client_gone {
            return Err(OutboundError("closed".to_owned()));
        }
        rec.frames.push(envelope.clone());
        Ok(())
    }
}

pub struct FakeUi(pub Shared);

impl ConfirmUi for FakeUi {
    fn command(&mut self, command: UiCommand) {
        self.0.borrow_mut().ui.push(command);
    }

    fn parent_window(&self) -> Option<isize> {
        Some(PARENT_WINDOW)
    }
}

pub struct FakeKeys(pub Shared);

impl KeyService for FakeKeys {
    fn send(&mut self, command: KeyCommand) {
        self.0.borrow_mut().keys.push(command);
    }
}

pub struct FakeLauncher(pub Shared);

impl Launcher for FakeLauncher {
    fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), String> {
        let mut rec = self.0.borrow_mut();
        rec.diagnostics.push(tab);
        if rec.launcher_fails {
            Err("cannot start".to_owned())
        } else {
            Ok(())
        }
    }
}
