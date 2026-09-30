//! The harness of the window tests: the real [`ConfirmWindow`] at 480 × 600
//! in `egui_kittest`, a hand-moved clock, the engine's end of the event
//! channel, and per-OS snapshots under `snapshots/<os>/` (reviewed like
//! the widgets' baselines: an OS folder counts once it has a `SUMMARY.md`).

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use egui::Vec2;
use egui_kittest::{Harness, SnapshotError, SnapshotOptions};
use websign_host::EngineEvent;
use websign_i18n::{Catalog, Locale};
use websign_ui_model::certs::CertCandidate;
use websign_ui_model::confirm::port::{Failure, Finish, OpenRequest};
use websign_ui_model::confirm::{UiCommand, UiEvent};
use websign_ui_model::possible::PossibleCard;

use super::fixtures::{KEY, code, context, fingerprint};
use crate::ui::confirm::clock::{Clock, ManualTime};
use crate::ui::confirm::window::ConfirmWindow;
use crate::ui::theme;

pub const THEMES: [(bool, &str); 2] = [(false, "light"), (true, "dark")];
pub const SIZE: Vec2 = Vec2::new(480.0, 600.0);

/// The window before its first frame installs fonts and themes.
pub struct Slot {
    pub window: Option<ConfirmWindow>,
    parts: Option<(Catalog, std::sync::mpsc::Sender<EngineEvent>, Clock)>,
    dark: bool,
}

pub struct Rig {
    pub harness: Harness<'static, Slot>,
    pub time: ManualTime,
    pub events: Receiver<EngineEvent>,
}

impl Rig {
    pub fn new(dark: bool) -> Rig {
        let (clock, time) = Clock::manual();
        let (sender, events) = channel();
        let slot = Slot {
            window: None,
            parts: Some((Catalog::new(Locale::En), sender, clock)),
            dark,
        };
        let mut harness = Harness::builder()
            .with_size(SIZE)
            .wgpu()
            .build_ui_state(frame, slot);
        harness.run_steps(2);
        Rig {
            harness,
            time,
            events,
        }
    }

    pub fn window(&mut self) -> &mut ConfirmWindow {
        self.harness
            .state_mut()
            .window
            .as_mut()
            .expect("installed in the first frame")
    }

    /// Applies a host command and lets the window draw it.
    pub fn apply(&mut self, command: UiCommand) {
        self.window().apply(command);
        self.settle();
    }

    pub fn wait(&mut self, ms: u64) {
        self.time.advance(Duration::from_millis(ms));
        self.settle();
    }

    pub fn settle(&mut self) {
        self.harness.run_steps(3);
    }

    pub fn open(&mut self, request: OpenRequest) {
        self.apply(UiCommand::Open(request));
    }

    pub fn list(&mut self, candidates: Vec<CertCandidate>, possible: Vec<PossibleCard>) {
        self.apply(UiCommand::Certificates {
            key: KEY,
            candidates,
            possible,
            context: context(),
        });
    }

    pub fn digest(&mut self, seed: u8) {
        self.apply(UiCommand::DigestReady {
            key: KEY,
            fingerprint: fingerprint(seed),
            code: code(),
        });
    }

    pub fn fail(&mut self, failure: Failure) {
        self.apply(UiCommand::Signing { key: KEY });
        self.apply(UiCommand::Failed { key: KEY, failure });
    }

    pub fn finish(&mut self, finish: Finish) {
        self.apply(UiCommand::Finished { key: KEY, finish });
    }

    /// Every decision the window sent so far.
    pub fn sent(&self) -> Vec<UiEvent> {
        self.events
            .try_iter()
            .filter_map(|event| match event {
                EngineEvent::Ui(event) => Some(event),
                _ => None,
            })
            .collect()
    }

    pub fn snapshot(&mut self, name: &str) {
        snapshot(&mut self.harness, name);
    }
}

fn frame(ui: &mut egui::Ui, slot: &mut Slot) {
    if let Some((tr, events, clock)) = slot.parts.take() {
        // Our fonts apply from the next frame: this one only installs.
        let installed = theme::install_for_tests(ui.ctx(), slot.dark);
        slot.window = Some(ConfirmWindow::new(installed, tr, events, clock));
        ui.ctx().request_repaint();
        return;
    }
    if let Some(window) = &mut slot.window {
        // kittest wraps the app in a margin; eframe gives the window the
        // whole surface, so the tests do too.
        let whole = ui.ctx().content_rect();
        let mut surface = ui.new_child(egui::UiBuilder::new().max_rect(whole));
        window.frame(&mut surface);
    }
}

fn snapshot(harness: &mut Harness<'_, Slot>, name: &str) {
    let dir = format!("src/ui/confirm/snapshots/{}", std::env::consts::OS);
    let options = SnapshotOptions::new().output_path(&dir);
    match harness.try_snapshot_options(name, &options) {
        Ok(()) => {}
        Err(SnapshotError::OpenSnapshot { .. })
            if !std::path::Path::new(&dir).join("SUMMARY.md").exists() =>
        {
            eprintln!("no reviewed baselines for this OS yet: wrote {dir}/{name}.new.png");
        }
        Err(error) => panic!("{error}"),
    }
}
