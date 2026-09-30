use egui::Vec2;
use egui::accesskit::{Live, Role};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::support::{THEMES, harness, snapshot};
use crate::ui::icons;
use crate::ui::widgets::badge::Badge;
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::chip::Chip;
use crate::ui::widgets::tone::Tone;

const NEW_SITE_A11Y: &str = "First time this site asks for anything on this computer";
const LOCKED: &str = "The token locked after too many wrong attempts.";

fn notices(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(Chip::new(Tone::Success, "Allowed site"));
        ui.add(Chip::new(Tone::Neutral, "New site").accessible_name(NEW_SITE_A11Y));
        ui.add(Chip::new(Tone::Warning, "Needs attention"));
        ui.add(Badge::new("ICP-Brasil A3"));
    });
    ui.add(
        Banner::new(
            Tone::Warning,
            "Address with special characters. Check it letter by letter.",
        )
        .compact(),
    );
    ui.add(Banner::new(
        Tone::Info,
        "The site will receive the name, type, issuer and validity of the chosen certificate. Nothing is signed now.",
    ));
    ui.add(
        Banner::new(Tone::Danger, LOCKED)
            .title("PIN locked")
            .icon(icons::PIN_LOCKED),
    );
}

#[test]
fn chips_and_banners_are_announced() {
    let h = harness(Vec2::new(480.0, 280.0), false, notices);
    assert!(
        h.query_by_label(NEW_SITE_A11Y).is_some(),
        "the chip reads its long name"
    );
    let name = format!("PIN locked. {LOCKED}");
    let error = h.get_by_label(&name);
    assert_eq!(error.accesskit_node().role(), Role::Alert);
    assert_eq!(error.accesskit_node().live(), Live::Assertive);
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut h = harness(Vec2::new(480.0, 280.0), dark, notices);
        snapshot(&mut h, &format!("notices-{theme}"));
    }
}
