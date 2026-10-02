use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::support::{THEMES, harness, snapshot};
use crate::ui::icons;
use crate::ui::theme::metrics;
use crate::ui::widgets::tabs::{Tab, TabStatus};
use crate::ui::widgets::tone::Tone;

fn sidebar(ui: &mut egui::Ui, selected: &mut usize) {
    ui.set_width(metrics::SIDEBAR - 24.0);
    let ok = TabStatus {
        icon: icons::SUCCESS,
        tone: Tone::Success,
        name: "Works",
    };
    let warn = TabStatus {
        icon: icons::ATTENTION,
        tone: Tone::Warning,
        name: "Needs attention",
    };
    let tabs = [
        (icons::BROWSERS, "Browsers", Some(ok)),
        (icons::TOKEN, "Devices", Some(warn)),
        (icons::CERTIFICATE, "Certificates", Some(ok)),
        (icons::HELP, "Help", None),
    ];
    for (index, (icon, label, status)) in tabs.into_iter().enumerate() {
        if (Tab {
            icon,
            label,
            status,
            selected: *selected == index,
        })
        .show(ui)
        .clicked()
        {
            *selected = index;
        }
    }
}

#[test]
fn tabs_expose_selection_and_status() {
    let mut selected = 0;
    let mut h = harness(Vec2::new(220.0, 220.0), false, |ui| {
        sidebar(ui, &mut selected)
    });
    let devices = h.get_by_role_and_label(Role::Tab, "Devices, Needs attention");
    assert_eq!(devices.accesskit_node().is_selected(), Some(false));
    devices.click();
    h.run();
    drop(h);
    assert_eq!(selected, 1);
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut selected = 1;
        let mut h = harness(Vec2::new(220.0, 220.0), dark, |ui| {
            sidebar(ui, &mut selected)
        });
        snapshot(&mut h, &format!("tabs-{theme}"));
    }
}
