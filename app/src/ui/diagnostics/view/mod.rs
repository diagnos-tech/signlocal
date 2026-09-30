//! Drawing the window (`docs/ux.md` §8.1): a 200 px sidebar (brand, overall
//! light, tabs, "Copy diagnostics") and the selected tab's content, which
//! scrolls. Views only read a [`Screen`] and return [`Action`]s.

mod action;
mod browsers;
mod cert_text;
mod certificates;
mod devices;
mod drivers;
mod faq;
mod header;
mod help;
mod onboarding;
mod onboarding_steps;
mod presence;
mod row;
mod section;
mod sidebar;
mod sites;
mod spans;
mod words;

use egui::{Rect, ScrollArea, Stroke, Ui, UiBuilder, pos2};
use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_host::store::Settings;
use websign_i18n::Catalog;
use websign_protocol::messages::DiagnosticsTab;

use super::facts::Facts;
use super::lights::Lights;
use super::remembered::Remembered;
use super::report::Environment;
use super::state::{Action, UiState};
use crate::ui::theme::{self, metrics};

/// Everything a frame shows.
#[derive(Debug, Clone, Copy)]
pub struct Screen<'a> {
    pub catalog: &'a Catalog,
    pub environment: &'a Environment,
    /// `None` until the first scan ends.
    pub facts: Option<&'a Facts>,
    pub lights: Option<Lights>,
    pub sites: &'a [Remembered],
    pub programs: &'a [Remembered],
    pub settings: &'a Settings,
    pub state: &'a UiState,
    /// The "Copy diagnostics" text of the current facts.
    pub report: Option<&'a str>,
    pub now: Timestamp,
    pub zone: &'a TimeZone,
    /// egui's clock, for the timed states (confirm revoke, notices).
    pub clock: f64,
    pub scanning: bool,
    /// An OS file dialog is open: its buttons wait.
    pub picking: bool,
}

impl Screen<'_> {
    pub fn tab(&self) -> DiagnosticsTab {
        self.state.tab.unwrap_or(DiagnosticsTab::Browsers)
    }
}

/// Draws the whole window into `ui`.
pub fn draw(ui: &mut Ui, screen: &Screen<'_>) -> Vec<Action> {
    let c = theme::colors(ui.ctx());
    let mut actions = Vec::new();
    let whole = ui.max_rect();
    ui.painter().rect_filled(whole, 0.0, c.bg_canvas);
    let side = Rect::from_min_max(whole.min, pos2(whole.min.x + metrics::SIDEBAR, whole.max.y));
    ui.painter().rect_filled(side, 0.0, c.bg_surface);
    ui.painter()
        .vline(side.max.x - 0.5, side.y_range(), Stroke::new(1.0, c.border));
    let mut side_ui = ui.new_child(UiBuilder::new().id_salt("sidebar").max_rect(side));
    sidebar::show(&mut side_ui, screen, &mut actions);

    let content = Rect::from_min_max(pos2(side.max.x, whole.min.y), whole.max);
    let mut content_ui = ui.new_child(UiBuilder::new().id_salt("content").max_rect(content));
    ScrollArea::vertical()
        .id_salt(("tab", screen.tab() as u8))
        .auto_shrink(false)
        .show(&mut content_ui, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 24,
                    right: 24,
                    top: 22,
                    bottom: 24,
                })
                .show(ui, |ui| tab(ui, screen, &mut actions));
        });
    actions
}

fn tab(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    ui.set_width(ui.available_width());
    // Vertical rhythm comes from explicit spaces, as the mockups' margins.
    ui.spacing_mut().item_spacing.y = 0.0;
    let tab = screen.tab();
    header::show(ui, screen, tab, actions);
    if let Some(facts) = screen.facts {
        onboarding::show(ui, screen, facts, actions);
    }
    match (tab, screen.facts) {
        (DiagnosticsTab::Help, _) => help::show(ui, screen, actions),
        (_, None) => header::loading(ui, screen),
        (DiagnosticsTab::Browsers, Some(facts)) => {
            browsers::show(ui, screen, facts, actions);
            sites::show(ui, screen, actions);
        }
        (DiagnosticsTab::Devices, Some(facts)) => {
            devices::show(ui, screen, facts, actions);
            drivers::show(ui, screen, facts, actions);
        }
        (DiagnosticsTab::Certificates, Some(facts)) => {
            certificates::show(ui, screen, facts, actions);
        }
    }
}
