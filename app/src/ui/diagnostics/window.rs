//! The window's state and its frame: collect the finished scan, read the
//! shortcuts, draw, then apply what the person did. Everything slow (the
//! scan) runs elsewhere; everything that changes a store happens here, after
//! drawing.

use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_host::store::{Settings, Stores};
use websign_i18n::Catalog;
use websign_protocol::messages::DiagnosticsTab;

use super::facts::{Facts, ScanInput};
use super::lights::Lights;
use super::live::Live;
use super::pick::Picks;
use super::remembered::{self, Remembered};
use super::report::{self, Environment};
use super::services::{Os, Owner, Scanner};
use super::state::UiState;
use super::view::{self, Screen};
use crate::ui::theme::Installed;

/// Where the window gets the current time (fixed in tests).
pub type Clock = Box<dyn Fn() -> Timestamp>;

/// Everything the window is built from.
pub struct Parts {
    pub catalog: Catalog,
    pub environment: Environment,
    pub stores: Box<dyn Stores>,
    pub os: Box<dyn Os>,
    pub scanner: Box<dyn Scanner>,
    /// The folder the stores live in, for the scan's records.
    pub data_dir: Option<std::path::PathBuf>,
    pub zone: TimeZone,
    pub clock: Clock,
    /// The tab asked for; `None` opens on the first with a problem.
    pub tab: Option<DiagnosticsTab>,
    /// Device events that call for a new scan.
    pub live: Option<Live>,
}

/// The eframe app state of the diagnostics window.
pub struct DiagnosticsWindow {
    pub(super) parts: Parts,
    pub(super) facts: Option<Facts>,
    pub(super) lights: Option<Lights>,
    pub(super) report: Option<String>,
    pub(super) sites: Vec<Remembered>,
    pub(super) programs: Vec<Remembered>,
    pub(super) settings: Settings,
    pub(super) state: UiState,
    /// Whether the window had focus last frame (a `.pfx` import finished in
    /// another window when it comes back).
    pub(super) focused: bool,
    /// Devices changed while a scan ran: scan again once it ends.
    pub(super) rescan_pending: bool,
    /// The file picker's dialog, while open.
    pub(super) picks: Picks,
    /// This window's native handle, owner of the OS dialogs.
    pub(super) owner: Owner,
}

impl std::fmt::Debug for DiagnosticsWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiagnosticsWindow")
            .field("tab", &self.state.tab)
            .field("scanned", &self.facts.is_some())
            .finish_non_exhaustive()
    }
}

impl DiagnosticsWindow {
    /// A window over `parts`; the first scan starts with the first frame.
    pub fn new(parts: Parts, _installed: Installed) -> DiagnosticsWindow {
        let tab = parts.tab;
        let mut window = DiagnosticsWindow {
            parts,
            facts: None,
            lights: None,
            report: None,
            sites: Vec::new(),
            programs: Vec::new(),
            settings: Settings::default(),
            state: UiState::new(tab),
            focused: true,
            rescan_pending: false,
            picks: Picks::default(),
            owner: None,
        };
        window.reload_stores();
        window
    }

    /// The native window the OS dialogs belong to (modal over it).
    pub fn set_owner(&mut self, owner: Owner) {
        self.owner = owner;
    }

    /// Shows `facts` as if a scan had just returned them.
    pub fn set_facts(&mut self, facts: Facts) {
        let lights = Lights::of(&facts);
        if self.state.tab.is_none() {
            self.state.tab = Some(lights.first_problem());
        }
        self.report = Some(report::text(
            &facts,
            &self.parts.environment,
            (self.parts.clock)(),
        ));
        self.lights = Some(lights);
        self.facts = Some(facts);
    }

    /// Draws one frame into `ui` and applies what the person did.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.style_mut().interaction.selectable_labels = false;
        self.follow_scale(&ctx);
        if let Some(facts) = self.parts.scanner.finished() {
            self.set_facts(facts);
        }
        let devices_changed = self.parts.live.as_ref().is_some_and(Live::changed);
        self.rescan_pending |= devices_changed;
        if (self.facts.is_none() || self.rescan_pending) && !self.parts.scanner.busy() {
            self.rescan_pending = false;
            self.rescan(&ctx);
        }
        if let Some(outcome) = self.picks.answered() {
            self.picked(&ctx, outcome);
        }
        self.refresh_on_focus(&ctx);
        let mut actions = super::keys::shortcuts(&ctx);
        let clock = ctx.input(|input| input.time);
        let screen = Screen {
            catalog: &self.parts.catalog,
            environment: &self.parts.environment,
            facts: self.facts.as_ref(),
            lights: self.lights,
            sites: &self.sites,
            programs: &self.programs,
            settings: &self.settings,
            state: &self.state,
            report: self.report.as_deref(),
            now: (self.parts.clock)(),
            zone: &self.parts.zone,
            clock,
            scanning: self.parts.scanner.busy(),
            picking: self.picks.is_open(),
        };
        actions.extend(view::draw(ui, &screen));
        for action in actions {
            self.apply(&ctx, action, clock);
        }
        self.schedule_timers(&ctx, clock);
    }

    /// Starts a scan with the current settings.
    pub(super) fn rescan(&mut self, ctx: &egui::Context) {
        let input = ScanInput {
            data_dir: self.parts.data_dir.clone(),
            user_modules: self.settings.user_modules.clone(),
            now: (self.parts.clock)(),
            zone: self.parts.zone.clone(),
        };
        let ctx = ctx.clone();
        self.parts
            .scanner
            .start(input, Box::new(move || ctx.request_repaint()));
    }

    /// Re-reads remembered callers and settings (cheap JSON files).
    pub(super) fn reload_stores(&mut self) {
        let records = self.parts.stores.consent().list().unwrap_or_else(|error| {
            log::warn!("diagnostics: remembered callers unreadable: {error}");
            Vec::new()
        });
        (self.sites, self.programs) = remembered::split(records);
        self.settings = self.parts.stores.settings().get().unwrap_or_else(|error| {
            log::warn!("diagnostics: settings unreadable: {error}");
            Settings::default()
        });
    }

    /// The report states the UI scale; keep it current when the window
    /// moves to another monitor.
    fn follow_scale(&mut self, ctx: &egui::Context) {
        let scale = (ctx.pixels_per_point() * 100.0).round() as u32;
        if scale != self.parts.environment.scale_percent {
            self.parts.environment.scale_percent = scale;
            if let Some(facts) = &self.facts {
                let now = (self.parts.clock)();
                self.report = Some(report::text(facts, &self.parts.environment, now));
            }
        }
    }

    /// The Certificates tab re-reads the list when the window regains
    /// focus: a `.pfx` import finishes in the OS's own window (§8.5).
    fn refresh_on_focus(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|input| input.viewport().focused.unwrap_or(true));
        if focused && !self.focused && self.state.tab == Some(DiagnosticsTab::Certificates) {
            self.rescan(ctx);
        }
        self.focused = focused;
    }

    fn schedule_timers(&self, ctx: &egui::Context, clock: f64) {
        let deadlines = [
            self.state.revoke_armed.as_ref().map(|(_, until)| *until),
            self.state.notice.as_ref().map(|(_, until)| *until),
        ];
        if let Some(next) = deadlines
            .into_iter()
            .flatten()
            .filter(|until| *until > clock)
            .reduce(f64::min)
        {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(next - clock));
        }
    }
}
