use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::support::{THEMES, harness, snapshot};
use crate::ui::theme::{metrics, typography};
use crate::ui::widgets::list;
use crate::ui::widgets::skeleton::Skeleton;
use crate::ui::widgets::spinner::Spinner;

#[test]
fn the_spinner_is_a_named_progress_indicator() {
    let h = harness(Vec2::new(200.0, 60.0), false, |ui| {
        ui.add(Spinner::new("Looking for certificates…"));
    });
    let node = h.get_by_label("Looking for certificates…");
    assert_eq!(node.accesskit_node().role(), Role::ProgressIndicator);
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut h = harness(Vec2::new(320.0, 140.0), dark, |ui| {
            ui.horizontal(|ui| {
                ui.add(Spinner::new("Loading"));
                ui.add(Spinner::new("Loading").size(metrics::ICON_LG));
            });
            // Skeletons sit on surfaces (the code card, the list).
            list::show(ui, |ui| {
                egui::Frame::new().inner_margin(12).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing = Vec2::new(12.0, 8.0);
                    ui.horizontal(|ui| {
                        ui.add(Skeleton::new(Vec2::splat(metrics::IDENTICON)));
                        ui.vertical(|ui| {
                            ui.add(Skeleton::line(200.0, typography::BODY_STRONG.line_height));
                            ui.add(Skeleton::line(140.0, typography::SMALL.line_height));
                        });
                    });
                });
            });
        });
        snapshot(&mut h, &format!("loading-{theme}"));
    }
}
