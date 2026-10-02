use egui::Vec2;
use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::support::{THEMES, harness, snapshot};
use crate::ui::icons;
use crate::ui::widgets::cert_row::{CertRow, CertRowText};
use crate::ui::widgets::list::{self, Position};
use crate::ui::widgets::tone::Tone;

const ANA_A3: &str = "Ana Beatriz Souza, ICP-Brasil A3, CPF partially hidden 456 789, AC SOLUTI Multipla v5, Card in reader, expires in 23 days";
const ANA_A1: &str = "Ana Beatriz Souza, ICP-Brasil A1, On this computer, valid until 14 Mar 2027";
const OLD: &str = "Ana Beatriz Souza, ICP-Brasil A1, expired on 10 May 2026";

fn text(
    name: &'static str,
    badge: &'static str,
    status: &'static str,
    tone: Tone,
    accessible: &'static str,
) -> CertRowText<'static> {
    CertRowText {
        name,
        badge,
        detail: "CPF •••.456.789-•• · AC SOLUTI Multipla v5",
        location_icon: icons::CARD,
        location: "Card in reader Identiv uTrust 2700 R · via driver",
        status,
        status_tone: tone,
        status_icon: match tone {
            Tone::Danger => Some(icons::ERROR),
            Tone::Warning => Some(icons::EXPIRES_SOON),
            _ => None,
        },
        accessible_name: accessible,
    }
}

/// Three rows: selected, usable, expired; returns which row was clicked.
fn list(ui: &mut egui::Ui, selected: &mut usize) {
    list::show(ui, |ui| {
        let rows = [
            (
                text(
                    "Ana Beatriz Souza",
                    "ICP-Brasil A3",
                    "Expires in 23 days",
                    Tone::Warning,
                    ANA_A3,
                ),
                true,
            ),
            (
                text(
                    "Ana Beatriz Souza",
                    "ICP-Brasil A1",
                    "Valid until 14 Mar 2027",
                    Tone::Neutral,
                    ANA_A1,
                ),
                true,
            ),
            (
                text(
                    "Ana Beatriz Souza",
                    "ICP-Brasil A1",
                    "Expired on 10 May 2026",
                    Tone::Danger,
                    OLD,
                ),
                false,
            ),
        ];
        let len = rows.len();
        for (index, (text, enabled)) in rows.into_iter().enumerate() {
            let row = CertRow {
                text,
                selected: *selected == index,
                enabled,
                radio: true,
                details: Some("Details"),
                position: Position::of(index, len),
            }
            .show(ui);
            if row.row.clicked() {
                *selected = index;
            }
        }
    });
}

#[test]
fn rows_are_radio_buttons_with_full_names() {
    let mut selected = 0;
    let h = harness(Vec2::new(480.0, 280.0), false, |ui| list(ui, &mut selected));
    let first = h.get_by_role_and_label(Role::RadioButton, ANA_A3);
    assert_eq!(first.accesskit_node().toggled(), Some(Toggled::True));
    let second = h.get_by_role_and_label(Role::RadioButton, ANA_A1);
    assert_eq!(second.accesskit_node().toggled(), Some(Toggled::False));
    let expired = h.get_by_role_and_label(Role::RadioButton, OLD);
    assert!(expired.accesskit_node().is_disabled());
    assert_eq!(
        h.query_all_by_role_and_label(Role::Link, "Details").count(),
        1
    );
}

#[test]
fn clicking_selects_but_a_disabled_row_does_not() {
    let mut selected = 0;
    let mut h = harness(Vec2::new(480.0, 280.0), false, |ui| list(ui, &mut selected));
    h.get_by_role_and_label(Role::RadioButton, ANA_A1).click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, OLD).click();
    h.run();
    drop(h);
    assert_eq!(selected, 1);
}

#[test]
fn enter_on_a_row_does_not_select_it() {
    let mut selected = 0;
    let mut h = harness(Vec2::new(480.0, 280.0), false, |ui| list(ui, &mut selected));
    h.get_by_role_and_label(Role::RadioButton, ANA_A1).focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    h.run();
    drop(h);
    assert_eq!(selected, 0, "Enter only moves focus (ux.md §4.9)");
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut selected = 0;
        let mut h = harness(Vec2::new(480.0, 260.0), dark, |ui| list(ui, &mut selected));
        snapshot(&mut h, &format!("cert-row-{theme}"));
    }
}
