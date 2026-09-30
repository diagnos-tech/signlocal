//! Our PIN field stays on screen (`docs/ux.md` §4.6): with a site notice,
//! the code card and a long list, the rows give up height so the field sits
//! inside the visible body, above the footer, without scrolling the body;
//! after a failure the whole PIN block (with the line under the field)
//! stays visible too. Where this OS asks for the card's PIN in its own
//! window, that card shows no field of ours at all.

use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use websign_protocol::types::BrowserName;
use websign_ui_model::certs::{CertCandidate, PinMode};

use super::fixtures::*;
use super::scenes::SCENES;
use super::support::{Rig, SIZE, THEMES};

/// The fixed footer's height (`view/mod.rs`).
const FOOTER: f32 = 64.0;
/// The line under the field while there is no error.
pub const PIN_PRIVACY: &str = "Your PIN stays on this computer and never goes through the browser.";

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

/// The PIN block of a scene: its caption, our field and the line under it.
fn pin_block(rig: &Rig, line: &str) -> Rect {
    let caption = rig
        .harness
        .query_all_by_value("Card PIN")
        .chain(rig.harness.query_all_by_value("Token PIN"))
        .map(|node| node.rect());
    let field = rig.harness.get_by_role(Role::PasswordInput).rect();
    let line = rig.harness.get_all_by_value(line).map(|node| node.rect());
    caption.chain(line).fold(field, Rect::union)
}

#[test]
fn the_whole_pin_block_stays_visible_after_a_failure() {
    // The main scenes sign with Ana's card, which only Linux (no system key
    // store) unlocks with our field; the PIN error scene uses a token.
    let card_asks_here = matches!(ana_card().pin, PinMode::App { .. });
    let cases = [
        ("ready", PIN_PRIVACY, card_asks_here),
        ("error-driver", PIN_PRIVACY, card_asks_here),
        (
            "pin-error",
            "Incorrect PIN. Last attempt: one more mistake locks the token.",
            true,
        ),
    ];
    for (dark, theme) in THEMES {
        for (name, line, ours) in cases {
            let (_, scene) = SCENES
                .into_iter()
                .find(|(scene, _)| *scene == name)
                .expect("a known scene");
            let mut rig = Rig::new(dark);
            scene(&mut rig);
            if !ours {
                assert!(
                    rig.harness.query_by_role(Role::PasswordInput).is_none(),
                    "{name}, {theme}: the system asks for this PIN, not our field"
                );
                continue;
            }
            let (body, block) = (rig.visible_body(), pin_block(&rig, line));
            assert!(
                body.contains_rect(block),
                "{name}, {theme}: the PIN block ({block:?}) is cut off ({body:?})"
            );
        }
    }
}
