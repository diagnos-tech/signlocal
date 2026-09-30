//! The harness every widget test uses: our fonts, a pinned theme, the
//! canvas background, and snapshots under `snapshots/<os>/`.
//!
//! Baselines are per OS (fonts rasterize slightly differently) and are
//! reviewed by a person, never accepted by CI. An OS folder counts as
//! reviewed once it has its `SUMMARY.md`. Until then the snapshot is still
//! rendered and written as `<name>.new.png` for review (a CI artifact) and
//! the test passes; afterwards a missing or different image fails.
//! `UPDATE_SNAPSHOTS=1 cargo test` rewrites the baselines locally.

use egui::{Frame, Margin, Ui, Vec2};
use egui_kittest::{Harness, SnapshotError, SnapshotOptions};

use crate::ui::theme;

/// `(dark, name suffix)` for both themes.
pub const THEMES: [(bool, &str); 2] = [(false, "light"), (true, "dark")];

/// A harness drawing `content` on the canvas color with 16 px margins.
pub fn harness<'a>(size: Vec2, dark: bool, mut content: impl FnMut(&mut Ui) + 'a) -> Harness<'a> {
    // Fonts set during a frame apply from the next one, and our named
    // families do not exist before that: the first frame only installs.
    let mut installed = false;
    let mut harness = Harness::builder()
        .with_size(size)
        .wgpu()
        .build_ui(move |ui| {
            if !installed {
                installed = true;
                let _ = theme::install_for_tests(ui.ctx(), dark);
                ui.ctx().request_repaint();
                return;
            }
            let canvas = theme::colors(ui.ctx()).bg_canvas;
            Frame::new()
                .fill(canvas)
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    content(ui);
                });
        });
    harness.run();
    harness
}

fn baseline_dir() -> String {
    format!("src/ui/widgets/snapshots/{}", std::env::consts::OS)
}

/// Compares the harness's image with `snapshots/<os>/<name>.png`.
pub fn snapshot(harness: &mut Harness<'_>, name: &str) {
    let dir = baseline_dir();
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
