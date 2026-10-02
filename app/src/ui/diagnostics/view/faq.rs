//! "Common questions" (`docs/ux.md` §8.6): an accordion, the first question
//! open. Each question is a button exposing its expanded state; the answer
//! follows it.

use egui::accesskit::Role;
use egui::{Frame, Label, Margin, Sense, Stroke, Ui, WidgetInfo, WidgetType, vec2};
use websign_i18n::{Catalog, Key, k};

use super::words::tr;
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::focus;
use crate::ui::widgets::list::Position;

/// Question height (`summary { height: 44px }`).
const QUESTION: f32 = 44.0;
/// The answer lines up with the question's text, after the caret.
const ANSWER_INDENT: i8 = 40;

/// What the answers are.
enum Answer {
    Text(Key),
    Shortcuts,
}

const QUESTIONS: [(Key, Answer); 4] = [
    (k::HELP_Q_MISSING, Answer::Text(k::HELP_A_MISSING)),
    (k::HELP_Q_PIN, Answer::Text(k::HELP_A_PIN)),
    (k::HELP_Q_PRIVACY, Answer::Text(k::HELP_A_PRIVACY)),
    (k::HELP_Q_SHORTCUTS, Answer::Shortcuts),
];

pub fn show(ui: &mut Ui, catalog: &Catalog, open: &[bool; 4], actions: &mut Vec<Action>) {
    let c = theme::colors(ui.ctx());
    Frame::new()
        .fill(c.bg_surface)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(metrics::RADIUS_LG)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (index, (question, answer)) in QUESTIONS.iter().enumerate() {
                let is_open = open.get(index).copied().unwrap_or(false);
                let position = Position::of(index, QUESTIONS.len());
                if summary(ui, &tr(catalog, *question), is_open, position) {
                    actions.push(Action::ToggleFaq(index));
                }
                if is_open {
                    let text = match answer {
                        Answer::Text(key) => tr(catalog, *key),
                        Answer::Shortcuts => shortcuts(catalog),
                    };
                    Frame::new()
                        .inner_margin(Margin {
                            left: ANSWER_INDENT,
                            right: 16,
                            top: 0,
                            bottom: 14,
                        })
                        .show(ui, |ui| {
                            ui.add(
                                Label::new(typography::BODY.rich(text).color(c.fg_muted)).wrap(),
                            );
                        });
                }
            }
        });
}

/// One question; `true` when toggled.
fn summary(ui: &mut Ui, question: &str, open: bool, position: Position) -> bool {
    let c = theme::colors(ui.ctx());
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), QUESTION), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, question));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(Role::Button);
        node.set_label(question.to_owned());
        node.set_expanded(open);
    });
    if position.divider() {
        ui.painter()
            .hline(rect.x_range(), rect.min.y + 0.5, Stroke::new(1.0, c.border));
    }
    if response.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(1.0), position.corners(), c.bg_hover);
    }
    let caret = if open { icons::COLLAPSE } else { icons::EXPAND };
    let glyph = crate::ui::widgets::text::whole(ui, caret.rich(metrics::ICON_SM, c.fg_muted));
    let middle = rect.center().y;
    ui.painter().galley(
        egui::pos2(rect.min.x + 16.0, middle - glyph.size().y / 2.0),
        glyph,
        c.fg_muted,
    );
    let text = crate::ui::widgets::text::line(
        ui,
        typography::BUTTON.rich(question).color(c.fg),
        rect.width() - 56.0,
    );
    ui.painter().galley(
        egui::pos2(rect.min.x + 40.0, middle - text.size().y / 2.0),
        text,
        c.fg,
    );
    focus::ring_inside(ui, &response, rect, position.corners());
    response.clicked()
}

/// The keys of both windows, one per line (`docs/ux.md` §4.9, §8.8).
fn shortcuts(catalog: &Catalog) -> String {
    let command = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    };
    let lines = [
        ("Esc".to_owned(), k::SHORTCUTS_CANCEL),
        ("↑ ↓".to_owned(), k::SHORTCUTS_MOVE_SELECTION),
        ("Tab".to_owned(), k::SHORTCUTS_FOCUS_NEXT),
        (format!("{command}+C"), k::SHORTCUTS_COPY_CODE),
        (format!("{command}+1…4"), k::SHORTCUTS_SWITCH_TAB),
        (format!("F5, {command}+R"), k::SHORTCUTS_SCAN_AGAIN),
        (format!("{command}+Shift+C"), k::SHORTCUTS_COPY_DIAGNOSTICS),
        (format!("{command}+W"), k::SHORTCUTS_CLOSE_WINDOW),
    ];
    lines
        .iter()
        .map(|(key, action)| {
            catalog
                .tr(k::SHORTCUTS_LINE)
                .arg("key", key)
                .arg("action", tr(catalog, *action))
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
