//! Phosphor icons (`egui-phosphor`), regular by default, fill for status
//! (`docs/ux.md` §12). One constant per concept so a concept changes icon in
//! one place.
//!
//! Only the glyphs listed in [`glyphs`] are embedded: `egui_phosphor::subset!`
//! cuts them out of the full fonts at compile time (a few KB instead of
//! ~940 KB). A concept built from `glyphs::fill::X` does not compile unless
//! `X` is in the fill subset, so a missing glyph cannot ship.

use egui::{Color32, FontDefinitions, FontFamily, FontId, RichText};

egui_phosphor::subset! {
    /// The Phosphor glyphs the windows use, per weight.
    pub mod glyphs {
        use regular::{
            ARROW_CLOCKWISE, ARROW_SQUARE_OUT, BROWSER, BROWSERS, CARET_DOWN, CARET_RIGHT,
            CERTIFICATE, CIRCLE_DASHED, CLOCK, COPY, DESKTOP, DOTS_NINE, DOWNLOAD_SIMPLE, EYE,
            EYE_SLASH, FILE_PLUS, GLOBE, HASH, IDENTIFICATION_CARD, KEYBOARD, LIST_CHECKS,
            LOCK_KEY_OPEN, LOCK_SIMPLE, MAGNIFYING_GLASS, PASSWORD, PLUG, PLUS, PUZZLE_PIECE,
            QUESTION, SHIELD_CHECK, SIGNATURE, STETHOSCOPE, TERMINAL_WINDOW, USB, X,
        };
        use fill::{CHECK_CIRCLE, INFO, LOCK_KEY, SEAL_CHECK, WARNING, X_CIRCLE};
    }
}

/// Name of the regular font, the fallback of every text family.
pub const REGULAR_FONT: &str = glyphs::regular::FONT_NAME;

/// A glyph and the weight it is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Icon {
    glyph: &'static str,
    fill: bool,
}

const fn regular(glyph: &'static str) -> Icon {
    Icon { glyph, fill: false }
}

const fn fill(glyph: &'static str) -> Icon {
    Icon { glyph, fill: true }
}

impl Icon {
    /// The character to put in a text run.
    pub fn glyph(self) -> &'static str {
        self.glyph
    }

    /// The family that renders this weight. Regular glyphs also render from
    /// every text family, so they may be mixed into labels.
    pub fn family(self) -> FontFamily {
        if self.fill {
            glyphs::fill::family()
        } else {
            glyphs::regular::family()
        }
    }

    /// The font at `size` points (§12: 16 in text, 20 in buttons and tabs,
    /// 32 in status cards).
    pub fn font_id(self, size: f32) -> FontId {
        FontId::new(size, self.family())
    }

    /// The icon as a text run of `size` in `color`.
    pub fn rich(self, size: f32, color: Color32) -> RichText {
        RichText::new(self.glyph)
            .font(self.font_id(size))
            .color(color)
    }
}

pub const BRAND: Icon = fill(glyphs::fill::SEAL_CHECK);
pub const SIGNATURE_REQUEST: Icon = regular(glyphs::regular::SIGNATURE);
pub const CERTIFICATE: Icon = regular(glyphs::regular::CERTIFICATE);
pub const SITE_HTTPS: Icon = regular(glyphs::regular::LOCK_SIMPLE);
pub const SITE_LOCAL: Icon = regular(glyphs::regular::TERMINAL_WINDOW);
pub const VERIFICATION_CODE: Icon = regular(glyphs::regular::HASH);
pub const ON_THIS_COMPUTER: Icon = regular(glyphs::regular::DESKTOP);
pub const TOKEN: Icon = regular(glyphs::regular::USB);
pub const CARD: Icon = regular(glyphs::regular::IDENTIFICATION_CARD);
pub const DRIVER: Icon = regular(glyphs::regular::PLUG);
pub const ADD_ON: Icon = regular(glyphs::regular::PUZZLE_PIECE);
pub const PIN: Icon = regular(glyphs::regular::PASSWORD);
pub const PIN_PAD: Icon = regular(glyphs::regular::DOTS_NINE);
pub const TOKEN_UNLOCKED: Icon = regular(glyphs::regular::LOCK_KEY_OPEN);
pub const PIN_LOCKED: Icon = fill(glyphs::fill::LOCK_KEY);
pub const PRIVACY: Icon = regular(glyphs::regular::SHIELD_CHECK);
pub const SHOW: Icon = regular(glyphs::regular::EYE);
pub const HIDE: Icon = regular(glyphs::regular::EYE_SLASH);
pub const EXPIRES_SOON: Icon = regular(glyphs::regular::CLOCK);
pub const SUCCESS: Icon = fill(glyphs::fill::CHECK_CIRCLE);
pub const ATTENTION: Icon = fill(glyphs::fill::WARNING);
pub const ERROR: Icon = fill(glyphs::fill::X_CIRCLE);
pub const INFO: Icon = fill(glyphs::fill::INFO);
pub const NOT_APPLICABLE: Icon = regular(glyphs::regular::CIRCLE_DASHED);
pub const BROWSERS: Icon = regular(glyphs::regular::BROWSERS);
pub const BROWSER: Icon = regular(glyphs::regular::BROWSER);
pub const ALLOWED_SITES: Icon = regular(glyphs::regular::GLOBE);
pub const HELP: Icon = regular(glyphs::regular::QUESTION);
pub const DIAGNOSTICS: Icon = regular(glyphs::regular::STETHOSCOPE);
pub const GETTING_STARTED: Icon = regular(glyphs::regular::LIST_CHECKS);
pub const DOWNLOAD: Icon = regular(glyphs::regular::DOWNLOAD_SIMPLE);
pub const EXTERNAL_LINK: Icon = regular(glyphs::regular::ARROW_SQUARE_OUT);
pub const COPY: Icon = regular(glyphs::regular::COPY);
pub const SCAN_AGAIN: Icon = regular(glyphs::regular::ARROW_CLOCKWISE);
pub const ADD: Icon = regular(glyphs::regular::PLUS);
pub const IMPORT: Icon = regular(glyphs::regular::FILE_PLUS);
pub const FILTER: Icon = regular(glyphs::regular::MAGNIFYING_GLASS);
pub const EXPAND: Icon = regular(glyphs::regular::CARET_RIGHT);
pub const COLLAPSE: Icon = regular(glyphs::regular::CARET_DOWN);
pub const CLOSE: Icon = regular(glyphs::regular::X);
pub const SHORTCUTS: Icon = regular(glyphs::regular::KEYBOARD);

/// Adds both weights to `fonts`: regular right after the text font of every
/// family already defined, fill as its own family.
pub(crate) fn add_to(fonts: &mut FontDefinitions) {
    glyphs::fill::add_as_family(fonts);
    glyphs::regular::add_as_family(fonts);
    let text_families: Vec<FontFamily> = fonts
        .families
        .keys()
        .filter(|family| {
            **family != glyphs::fill::family() && **family != glyphs::regular::family()
        })
        .cloned()
        .collect();
    for family in text_families {
        if let Some(chain) = fonts.families.get_mut(&family) {
            chain.insert(chain.len().min(1), REGULAR_FONT.to_owned());
        }
    }
}
