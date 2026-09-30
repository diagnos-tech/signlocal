//! The body reads whole under a long list (`docs/ux.md` §4.5, §4.10): its
//! top (the Continue hint, or what Choose mode shares) says what the primary
//! button does, and "Remember" says what ticking it grants, so both stay in
//! the visible body, between the header and the footer; the list scrolls
//! inside itself instead of the body.

use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use websign_protocol::types::BrowserName;
use websign_ui_model::certs::CertCandidate;
use websign_ui_model::confirm::port::{CallerView, Mode};

use super::fixtures::*;
use super::pin_fit::PIN_PRIVACY;
use super::support::{Rig, THEMES};

/// Where things landed for one request.
struct Seen {
    /// Between the header and the footer.
    body: Rect,
    /// The list's caption, right under the body's top block.
    caption: Rect,
    /// The checkbox with its label and help text.
    remember: Rect,
    /// The line under our PIN field, when it shows one.
    pin_line: Option<Rect>,
}

fn open(mode: Mode, caller: fn() -> CallerView, keys: Vec<CertCandidate>, dark: bool) -> Seen {
    let caller = caller();
    let (caption, remember) = match (mode, &caller) {
        (Mode::Choose, _) => (
            "Choose a certificate",
            "Remember this site on this computer",
        ),
        (_, CallerView::Web { .. }) => ("Sign with", "Remember this site on this computer"),
        (_, CallerView::Desktop { .. }) => ("Sign with", "Remember this program on this computer"),
    };
    let mut rig = Rig::new(dark);
    rig.open(request(mode, caller, false));
    rig.list(keys, Vec::new());
    rig.wait(700);
    Seen {
        body: rig.visible_body(),
        caption: rig
            .harness
            .get_all_by_value(caption)
            .map(|node| node.rect())
            .reduce(Rect::union)
            .expect("the list has its caption"),
        remember: rig
            .harness
            .get_by_role_and_label(Role::CheckBox, remember)
            .rect(),
        pin_line: rig
            .harness
            .query_all_by_value(PIN_PRIVACY)
            .map(|node| node.rect())
            .reduce(Rect::union),
    }
}

/// With `many` keys the body is not scrolled (its caption sits where it
/// does with one key, so the block above it is whole) and "Remember" is
/// whole inside it.
fn assert_readable(
    name: &str,
    mode: Mode,
    caller: fn() -> CallerView,
    one: fn() -> Vec<CertCandidate>,
    many: fn() -> Vec<CertCandidate>,
) {
    for (dark, theme) in THEMES {
        let unscrolled = open(mode, caller, one(), dark).caption;
        let seen = open(mode, caller, many(), dark);
        assert_eq!(
            seen.caption.top(),
            unscrolled.top(),
            "{name}, {theme}: the body scrolled its top block out of view"
        );
        assert!(
            seen.body.contains_rect(seen.caption),
            "{name}, {theme}: the list's caption is out of the body"
        );
        assert!(
            seen.body.contains_rect(seen.remember),
            "{name}, {theme}: Remember ({:?}) is not whole in the body ({:?})",
            seen.remember,
            seen.body
        );
        if let Some(line) = seen.pin_line {
            assert!(
                seen.body.contains_rect(line),
                "{name}, {theme}: the line under the PIN field is cut off"
            );
        }
    }
}

/// A localhost page: its header adds the development-site notice.
fn localhost() -> CallerView {
    web("http://localhost:35909", BrowserName::Chromium)
}

/// An unverified program: its header adds a warning.
fn unverified() -> CallerView {
    desktop("laudos.exe", false)
}

#[test]
fn choosing_among_many_certificates_keeps_the_body_whole() {
    assert_readable(
        "choose",
        Mode::Choose,
        localhost,
        || store_keys(1, 0),
        || store_keys(9, 3),
    );
}

#[test]
fn continuing_under_many_certificates_keeps_the_body_whole() {
    let one = || store_keys(1, 0);
    assert_readable("continue, system PIN", SIGN, localhost, one, || {
        store_keys(9, 3)
    });
    let one = || token_keys(1, 0);
    assert_readable("continue, our PIN field", SIGN, localhost, one, || {
        token_keys(9, 3)
    });
    assert_readable(
        "continue, unverified program",
        SIGN,
        unverified,
        one,
        || token_keys(2, 1),
    );
}
