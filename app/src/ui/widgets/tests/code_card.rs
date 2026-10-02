use egui::Vec2;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use websign_protocol::code::verification_code;

use super::support::{THEMES, harness, snapshot};
use crate::ui::widgets::code_card::{CodeCard, CodeState};

const SPOKEN: &str = "7 F 3 A, 9 C 2 1, E 0 B 4, 5 5 D 8";

fn card(ui: &mut egui::Ui, ready: bool) -> egui::Response {
    card_with(ui, ready, false)
}

fn card_with(ui: &mut egui::Ui, ready: bool, compact: bool) -> egui::Response {
    let code =
        verification_code(&[0x7F, 0x3A, 0x9C, 0x21, 0xE0, 0xB4, 0x55, 0xD8]).expect("8 bytes");
    let state = if ready {
        CodeState::Ready {
            code: &code,
            spoken: SPOKEN,
        }
    } else {
        CodeState::Preparing {
            preparing: "Preparing the document…",
        }
    };
    CodeCard {
        label: "Verification code",
        hash: "SHA-256",
        help: "Check that the site shows the same code.",
        state,
        compact,
    }
    .show(ui)
}

#[test]
fn the_code_is_read_character_by_character() {
    let h = harness(Vec2::new(480.0, 140.0), false, |ui| {
        card(ui, true);
    });
    let code = h.get_by_label(SPOKEN);
    assert_eq!(code.accesskit_node().role(), Role::Label);
    assert!(
        h.query_by_role(Role::Image).is_none(),
        "the identicon is decorative"
    );
}

#[test]
fn the_compact_card_drops_only_the_help_line() {
    let heights = std::cell::Cell::new((0.0, 0.0));
    let h = harness(Vec2::new(480.0, 140.0), false, |ui| {
        let full = card_with(ui, true, false).rect.height();
        let compact = card_with(ui, true, true).rect.height();
        heights.set((full, compact));
    });
    let (full, compact) = heights.get();
    assert!(compact <= full - 16.0, "{compact} vs {full}");
    assert_eq!(
        h.query_all_by_label("Check that the site shows the same code.")
            .count(),
        1,
        "only the full card says it"
    );
    assert_eq!(h.get_all_by_label(SPOKEN).count(), 2);
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut h = harness(Vec2::new(480.0, 350.0), dark, |ui| {
            card(ui, true);
            ui.add_space(12.0);
            card(ui, false);
            ui.add_space(12.0);
            card_with(ui, true, true);
        });
        snapshot(&mut h, &format!("code-card-{theme}"));
    }
}
