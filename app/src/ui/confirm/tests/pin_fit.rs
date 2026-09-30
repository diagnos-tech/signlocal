//! Our PIN field stays on screen (`docs/ux.md` §4.6): with a site notice,
//! the code card and a long list, the rows give up height so the field sits
//! inside the visible body, above the footer, without scrolling the body.

use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use websign_protocol::types::BrowserName;
use websign_ui_model::certs::CertCandidate;

use super::fixtures::*;
use super::support::{Rig, SIZE, THEMES};

/// The fixed footer's height (`view/mod.rs`).
const FOOTER: f32 = 64.0;

/// A localhost page (its header carries the development-site notice) asking
/// to sign with one of `keys`, the first selected with its code shown.
/// Returns the code card's caption and our PIN field.
fn ready_with(keys: Vec<CertCandidate>, dark: bool) -> (Rect, Rect) {
    let mut rig = Rig::new(dark);
    let caller = web("http://localhost:35909", BrowserName::Chromium);
    rig.open(request(SIGN, caller, true));
    rig.list(keys, Vec::new());
    rig.digest(40);
    rig.wait(700);
    let caption = rig
        .harness
        .get_all_by_value("Verification code")
        .map(|node| node.rect())
        .reduce(Rect::union)
        .expect("the code card is shown");
    let field = rig
        .harness
        .get_by_role_and_label(Role::PasswordInput, "Token PIN")
        .rect();
    (caption, field)
}

#[test]
fn the_pin_field_is_visible_without_scrolling_under_a_long_list() {
    for (dark, theme) in THEMES {
        // One key: nothing can push the body; where its top sits unscrolled.
        let (unscrolled, _) = ready_with(token_keys(1, 0), dark);
        for keys in [token_keys(6, 5), token_keys(9, 3)] {
            let count = keys.len();
            let (caption, field) = ready_with(keys, dark);
            assert_eq!(
                caption.top(),
                unscrolled.top(),
                "{theme}, {count} keys: the body scrolled"
            );
            assert!(
                field.bottom() <= SIZE.y - FOOTER,
                "{theme}, {count} keys: the footer covers the PIN field ({field:?})"
            );
        }
    }
}
