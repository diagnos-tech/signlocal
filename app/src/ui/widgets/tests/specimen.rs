//! Every text token and every icon concept, so a font or icon subset that
//! lost a glyph shows up as a snapshot difference.

use egui::Vec2;

use super::support::{THEMES, harness, snapshot};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, typography as t};

const ICONS: [Icon; 42] = [
    icons::BRAND,
    icons::SIGNATURE_REQUEST,
    icons::CERTIFICATE,
    icons::SITE_HTTPS,
    icons::SITE_LOCAL,
    icons::VERIFICATION_CODE,
    icons::ON_THIS_COMPUTER,
    icons::TOKEN,
    icons::CARD,
    icons::DRIVER,
    icons::ADD_ON,
    icons::PIN,
    icons::PIN_PAD,
    icons::TOKEN_UNLOCKED,
    icons::PIN_LOCKED,
    icons::PRIVACY,
    icons::SHOW,
    icons::HIDE,
    icons::EXPIRES_SOON,
    icons::SUCCESS,
    icons::ATTENTION,
    icons::ERROR,
    icons::INFO,
    icons::NOT_APPLICABLE,
    icons::BROWSERS,
    icons::BROWSER,
    icons::ALLOWED_SITES,
    icons::HELP,
    icons::DIAGNOSTICS,
    icons::GETTING_STARTED,
    icons::DOWNLOAD,
    icons::EXTERNAL_LINK,
    icons::COPY,
    icons::SCAN_AGAIN,
    icons::ADD,
    icons::IMPORT,
    icons::FILTER,
    icons::EXPAND,
    icons::COLLAPSE,
    icons::CLOSE,
    icons::SHORTCUTS,
    icons::BRAND,
];

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut h = harness(Vec2::new(480.0, 420.0), dark, |ui| {
            let c = theme::colors(ui.ctx());
            ui.label(t::HEADLINE.rich("app.diagnos.health").color(c.fg));
            ui.label(
                t::TITLE
                    .rich("Sign with — Assinar com — Signer avec")
                    .color(c.fg),
            );
            ui.label(
                t::BODY_STRONG
                    .rich("Ana Beatriz Souza · Ärztin Ñandú")
                    .color(c.fg),
            );
            ui.label(
                t::BODY
                    .rich("wants you to sign a document. диаgnos Ωμέγα")
                    .color(c.fg_muted),
            );
            ui.label(t::BUTTON.rich("Use this certificate").color(c.accent_fg));
            ui.label(t::CAPTION.rich("Verification code").color(c.fg_muted));
            ui.label(
                t::SMALL
                    .rich("Check that the site shows the same code.")
                    .color(c.fg_subtle),
            );
            ui.label(t::CODE.rich("7F3A 9C21 E0B4 55D8 0O1lI").color(c.fg));
            ui.label(
                t::MONO
                    .rich("USB 0529:0620 · 3B:D5:18:FF:81:91")
                    .color(c.fg_subtle),
            );
            ui.horizontal_wrapped(|ui| {
                for icon in ICONS {
                    ui.label(icon.rich(20.0, c.fg));
                }
            });
            ui.label(
                t::BODY
                    .rich(format!("{} Inline icon in text", icons::PRIVACY.glyph()))
                    .color(c.fg),
            );
        });
        snapshot(&mut h, &format!("specimen-{theme}"));
    }
}
