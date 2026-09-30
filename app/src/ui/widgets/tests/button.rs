use std::cell::Cell;

use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::support::{THEMES, harness, snapshot};
use crate::ui::icons;
use crate::ui::widgets::button::{Button, Size};

fn row(ui: &mut egui::Ui, armed: bool, clicks: &mut u32) {
    ui.horizontal(|ui| {
        if ui.add(Button::primary("Sign").armed(armed)).clicked() {
            *clicks += 1;
        }
        ui.add(Button::secondary("Cancel").size(Size::Large));
    });
}

#[test]
fn an_unarmed_button_is_disabled_and_ignores_clicks() {
    let mut clicks = 0;
    let mut h = harness(Vec2::new(360.0, 120.0), false, |ui| {
        row(ui, false, &mut clicks)
    });
    let sign = h.get_by_role_and_label(Role::Button, "Sign");
    assert!(sign.accesskit_node().is_disabled());
    sign.click();
    h.run();
    drop(h);
    assert_eq!(clicks, 0);
}

#[test]
fn an_armed_button_is_enabled_and_clicks() {
    let mut clicks = 0;
    let mut h = harness(Vec2::new(360.0, 120.0), false, |ui| {
        row(ui, true, &mut clicks)
    });
    let sign = h.get_by_role_and_label(Role::Button, "Sign");
    assert!(!sign.accesskit_node().is_disabled());
    sign.click();
    h.run();
    drop(h);
    assert_eq!(clicks, 1);
}

#[test]
fn enter_on_a_focused_button_is_reported_but_never_clicks() {
    let (clicks, enters) = (Cell::new(0), Cell::new(0));
    let mut h = harness(Vec2::new(360.0, 120.0), false, |ui| {
        let sign = Button::primary("Continue").show(ui);
        clicks.set(clicks.get() + u32::from(sign.response.clicked()));
        enters.set(enters.get() + u32::from(sign.enter));
    });
    h.get_by_role_and_label(Role::Button, "Continue").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(
        (clicks.get(), enters.get()),
        (0, 1),
        "Enter never releases (ux.md §4.4)"
    );
    h.key_press(egui::Key::Space);
    h.run();
    assert_eq!(clicks.get(), 1, "Space activates the focused button");
}

#[test]
fn a_busy_button_ignores_clicks() {
    let mut clicks = 0;
    let mut h = harness(Vec2::new(360.0, 120.0), false, |ui| {
        if ui.add(Button::primary("Signing…").busy(true)).clicked() {
            clicks += 1;
        }
    });
    let busy = h.get_by_role_and_label(Role::Button, "Signing…");
    assert!(busy.accesskit_node().is_disabled());
    busy.click();
    h.run();
    drop(h);
    assert_eq!(clicks, 0);
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut h = harness(Vec2::new(440.0, 200.0), dark, |ui| {
            ui.horizontal(|ui| {
                ui.add(Button::primary("Sign"));
                ui.add(Button::primary("Sign").armed(false));
                ui.add(Button::primary("Signing…").busy(true));
            });
            ui.horizontal(|ui| {
                ui.add(Button::secondary("Scan again").icon(icons::SCAN_AGAIN));
                ui.add(Button::ghost("Details").size(Size::Small));
                ui.add(Button::secondary("Disabled").enabled(false));
            });
            let focused = ui.add(Button::secondary("Focused").size(Size::Large));
            ui.memory_mut(|memory| memory.request_focus(focused.id));
        });
        h.run();
        snapshot(&mut h, &format!("button-{theme}"));
    }
}
