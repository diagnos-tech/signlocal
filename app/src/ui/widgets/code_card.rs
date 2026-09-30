//! The verification-code card (`docs/ux.md` §4.4): identicon, "Verification
//! code", the hash badge, the code in mono and "Check that the site shows the
//! same code." While the site prepares the digest, a skeleton takes the
//! identicon's and the code's places and "Preparing the document…" the help
//! line's, so the card keeps its height when the code arrives.
//!
//! The code is selectable (Ctrl/⌘+C copies it) and its accessible name is
//! the spelled-out form the caller passes (`code.a11y`: "7 F 3 A, 9 C 2 1…"),
//! because screen readers would otherwise read "7F3A" as a word.

use egui::{Frame, Label, Margin, Response, Stroke, Ui, Vec2, WidgetInfo, WidgetType};
use websign_protocol::code::VerificationCode;

use super::badge::Badge;
use super::identicon::Identicon;
use super::skeleton::Skeleton;
use crate::ui::theme::{self, metrics, typography};

/// Between the label, code and help lines (`gap: 2px`).
const LINE_GAP: f32 = 2.0;
/// The label line: as tall as the badge it holds.
const HEADING_HEIGHT: f32 = 20.0;
/// Label (20) + code (24) + help (16) lines and their two gaps.
const TEXT_HEIGHT: f32 = HEADING_HEIGHT + 24.0 + 16.0 + 2.0 * LINE_GAP;
/// About the width of "7F3A 9C21 E0B4 55D8" in `text-code`.
const CODE_WIDTH: f32 = 180.0;

/// What the card shows.
#[derive(Debug, Clone, Copy)]
pub enum CodeState<'a> {
    Ready {
        code: &'a VerificationCode,
        /// `code.a11y` with the code filled in.
        spoken: &'a str,
    },
    /// Waiting for the digest; `preparing` is "Preparing the document…".
    Preparing { preparing: &'a str },
}

/// The card's texts and state.
#[derive(Debug, Clone, Copy)]
pub struct CodeCard<'a> {
    pub label: &'a str,
    /// `SHA-256`, `SHA-384` or `SHA-512`.
    pub hash: &'a str,
    pub help: &'a str,
    pub state: CodeState<'a>,
}

impl CodeCard<'_> {
    pub fn show(self, ui: &mut Ui) -> Response {
        let c = theme::colors(ui.ctx());
        Frame::new()
            .fill(c.bg_surface)
            .stroke(Stroke::new(1.0, c.border))
            .corner_radius(metrics::RADIUS_LG)
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                // `gap: 16px; align-items: center` in the mockups. egui lays
                // out in one pass, so the identicon is centered by hand on
                // the text block's known height.
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(metrics::SPACE_4, 0.0);
                    ui.vertical(|ui| {
                        ui.add_space((TEXT_HEIGHT - metrics::IDENTICON) / 2.0);
                        match self.state {
                            CodeState::Ready { code, .. } => ui.add(Identicon::new(code)),
                            CodeState::Preparing { .. } => {
                                ui.add(Skeleton::new(Vec2::splat(metrics::IDENTICON)))
                            }
                        };
                    });
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, LINE_GAP);
                        self.heading(ui);
                        self.code(ui);
                        let (note, color) = match self.state {
                            CodeState::Ready { .. } => (self.help, c.fg_subtle),
                            CodeState::Preparing { preparing } => (preparing, c.fg_muted),
                        };
                        ui.label(typography::SMALL.rich(note).color(color));
                    });
                });
            })
            .response
    }

    /// "Verification code" and the hash badge on one 20 px line (the
    /// badge's height), both centered on it. The label stays a real label
    /// for screen readers; the badge is read as its own text.
    fn heading(&self, ui: &mut Ui) {
        let muted = theme::colors(ui.ctx()).fg_muted;
        let (badge, badge_size) = Badge::new(self.hash).measure(ui);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), HEADING_HEIGHT),
            egui::Sense::hover(),
        );
        let label = Label::new(typography::CAPTION.rich(self.label).color(muted));
        let label_rect = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.max.x - badge_size.x - metrics::SPACE_2, rect.max.y),
        );
        let left = egui::UiBuilder::new()
            .max_rect(label_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center));
        ui.scope_builder(left, |ui| ui.add(label));
        let badge_at = egui::pos2(
            rect.max.x - badge_size.x,
            rect.center().y - badge_size.y / 2.0,
        );
        Badge::paint(ui, badge_at, badge, badge_size);
        let hash = self.hash;
        let badge_rect = egui::Rect::from_min_size(badge_at, badge_size);
        ui.interact(badge_rect, ui.id().with("hash"), egui::Sense::hover())
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, hash));
    }

    fn code(&self, ui: &mut Ui) {
        let c = theme::colors(ui.ctx());
        match self.state {
            CodeState::Ready { code, spoken } => {
                let label = typography::CODE.rich(&code.text).color(c.fg);
                let response = ui.add(Label::new(label).selectable(true));
                response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, spoken));
            }
            CodeState::Preparing { .. } => {
                // The skeleton sits in the code's 24 px line box.
                let line = typography::CODE.line_height;
                let bar = Skeleton::line(CODE_WIDTH, line);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(CODE_WIDTH, line), egui::Sense::hover());
                ui.put(egui::Rect::from_center_size(rect.center(), bar.size()), bar);
            }
        }
    }
}
