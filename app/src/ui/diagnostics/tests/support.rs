//! The harness of the window tests: the real window over fake OS actions, a
//! scanner that never scans, a fixed clock and zone, English, a pinned
//! theme, and snapshots under `snapshots/<os>/` (reviewed baselines, as in
//! `widgets/tests/support.rs`).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use egui_kittest::{Harness, SnapshotError, SnapshotOptions};
use jiff::tz::TimeZone;
use websign_host::store::{MemoryStores, Stores};
use websign_i18n::{Catalog, Locale};
use websign_protocol::messages::DiagnosticsTab;

use super::fixture;
use crate::platform::file_picker::{FileRequest, Picked};
use crate::ui::diagnostics::facts::{Facts, ScanInput};
use crate::ui::diagnostics::services::{Os, Owner, Scanner};
use crate::ui::diagnostics::window::{DiagnosticsWindow, Parts};
use crate::ui::theme;
use crate::ui::theme::metrics;

/// `(dark, name suffix)` for both themes.
pub const THEMES: [(bool, &str); 2] = [(false, "light"), (true, "dark")];

/// What the fake OS was asked to do, and how its file picker answers.
#[derive(Debug)]
pub struct OsLog {
    pub urls: Vec<String>,
    pub repairs: usize,
    /// The file of each `.pfx` import (`None`: the OS asked for it).
    pub imports: Vec<Option<PathBuf>>,
    pub pickers: Vec<FileRequest>,
    pub picker_answer: Picked,
}

impl Default for OsLog {
    fn default() -> OsLog {
        OsLog {
            urls: Vec::new(),
            repairs: 0,
            imports: Vec::new(),
            pickers: Vec::new(),
            picker_answer: Picked::Cancelled,
        }
    }
}

struct FakeOs(Rc<RefCell<OsLog>>);

impl Os for FakeOs {
    fn open_url(&mut self, url: &str) {
        self.0.borrow_mut().urls.push(url.to_owned());
    }
    fn repair_registration(&mut self) -> bool {
        self.0.borrow_mut().repairs += 1;
        true
    }
    fn view_certificate(&mut self, _der: &[u8], _owner: Owner) {}
    fn import_pfx(&mut self, file: Option<&Path>, _owner: Owner) -> bool {
        self.0
            .borrow_mut()
            .imports
            .push(file.map(Path::to_path_buf));
        false
    }
    fn choose_file(
        &mut self,
        request: FileRequest,
        _owner: Owner,
        done: Box<dyn FnOnce(Picked) + Send>,
    ) {
        let mut log = self.0.borrow_mut();
        log.pickers.push(request);
        done(log.picker_answer.clone());
    }
}

/// Counts scans and never finishes one (it stays busy): tests hand the
/// window its facts.
struct FakeScanner(Rc<RefCell<usize>>);

impl Scanner for FakeScanner {
    fn start(&mut self, _input: ScanInput, _wake: Box<dyn FnOnce() + Send>) {
        *self.0.borrow_mut() += 1;
    }
    fn finished(&mut self) -> Option<Facts> {
        None
    }
    fn busy(&self) -> bool {
        *self.0.borrow() > 0
    }
}

/// How a test window starts.
pub struct Setup {
    pub dark: bool,
    pub facts: Option<Facts>,
    pub stores: Box<dyn Stores>,
    pub tab: Option<DiagnosticsTab>,
    /// Pixels per point; the report states it as the scale.
    pub scale: f32,
    /// Taller than the window when a test clicks rows below the fold
    /// (a click never reaches a widget scrolled out of view).
    pub height: f32,
}

impl Default for Setup {
    fn default() -> Setup {
        Setup {
            dark: false,
            facts: Some(fixture::facts()),
            stores: Box::new(MemoryStores::new()),
            tab: None,
            scale: 1.0,
            height: metrics::WINDOW_DIAGNOSTICS[1],
        }
    }
}

/// The harness, its window (inside the state once the first frame ran) and
/// the fakes' logs.
pub struct Window {
    pub harness: Harness<'static, Option<DiagnosticsWindow>>,
    pub os: Rc<RefCell<OsLog>>,
    pub scans: Rc<RefCell<usize>>,
}

impl Window {
    pub fn window(&mut self) -> &mut DiagnosticsWindow {
        match self.harness.state_mut() {
            Some(window) => window,
            None => panic!("the window is created on the first frame"),
        }
    }
}

pub fn open(setup: Setup) -> Window {
    let os = Rc::new(RefCell::new(OsLog::default()));
    let scans = Rc::new(RefCell::new(0));
    let parts = Parts {
        catalog: Catalog::new(Locale::En),
        environment: fixture::environment(),
        stores: setup.stores,
        os: Box::new(FakeOs(Rc::clone(&os))),
        scanner: Box::new(FakeScanner(Rc::clone(&scans))),
        data_dir: None,
        zone: TimeZone::UTC,
        clock: Box::new(fixture::now),
        tab: setup.tab,
        live: None,
    };
    let mut pending = Some((parts, setup.facts));
    let dark = setup.dark;
    let [width, _] = metrics::WINDOW_DIAGNOSTICS;
    let height = setup.height;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .with_pixels_per_point(setup.scale)
        .wgpu()
        .build_ui_state(
            move |ui, window: &mut Option<DiagnosticsWindow>| {
                if let Some(window) = window {
                    // The harness keeps an 8 px margin eframe does not: draw
                    // over the whole viewport, as in the real window.
                    let whole = ui.ctx().content_rect();
                    let mut full = ui.new_child(egui::UiBuilder::new().max_rect(whole));
                    full.set_clip_rect(whole);
                    window.show(&mut full);
                    return;
                }
                // Fonts set during a frame apply from the next one: the
                // first frame only installs.
                let installed = theme::install_for_tests(ui.ctx(), dark);
                if let Some((parts, facts)) = pending.take() {
                    let mut created = DiagnosticsWindow::new(parts, installed);
                    if let Some(facts) = facts {
                        created.set_facts(facts);
                    }
                    *window = Some(created);
                }
                ui.ctx().request_repaint();
            },
            None,
        );
    harness.run();
    Window { harness, os, scans }
}

/// Compares the harness's image with `snapshots/<os>/<name>.png`.
pub fn snapshot(harness: &mut Harness<'_, Option<DiagnosticsWindow>>, name: &str) {
    let dir = format!("src/ui/diagnostics/snapshots/{}", std::env::consts::OS);
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
