//! Type tokens (`docs/ux.md` §11.2) and their egui `TextStyle`s (§11.5).
//!
//! egui has no line height on a `FontId`, so a token carries its own and
//! [`TextToken::rich`] applies it; labels built that way match the mockups'
//! vertical rhythm exactly.

use std::collections::BTreeMap;

use egui::{FontId, RichText, TextStyle};

use crate::ui::fonts::{self, Face, Weight};

/// One `text-*` token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextToken {
    pub size: f32,
    pub line_height: f32,
    pub weight: Weight,
    pub face: Face,
    /// Extra spacing between letters, in points.
    pub letter_spacing: f32,
}

const fn ui(size: f32, line_height: f32, weight: Weight) -> TextToken {
    TextToken {
        size,
        line_height,
        weight,
        face: Face::Ui,
        letter_spacing: 0.0,
    }
}

/// Section labels, badges, chips.
pub const CAPTION: TextToken = ui(12.0, 16.0, Weight::Medium);
/// Metadata, help.
pub const SMALL: TextToken = ui(12.0, 16.0, Weight::Regular);
/// Default text (14, not 12: the audience includes older physicians).
pub const BODY: TextToken = ui(14.0, 20.0, Weight::Regular);
/// Holder name, browser or device name.
pub const BODY_STRONG: TextToken = ui(14.0, 20.0, Weight::SemiBold);
pub const BUTTON: TextToken = ui(14.0, 20.0, Weight::Medium);
/// Section titles, long origins.
pub const TITLE: TextToken = ui(16.0, 22.0, Weight::SemiBold);
/// The origin in the confirmation window, tab titles.
pub const HEADLINE: TextToken = ui(20.0, 26.0, Weight::SemiBold);
/// The verification code: mono so 0/O and 1/l differ.
pub const CODE: TextToken = TextToken {
    size: 18.0,
    line_height: 24.0,
    weight: Weight::Medium,
    face: Face::Mono,
    letter_spacing: 0.5,
};
/// VID:PID, ATR, paths, fingerprints.
pub const MONO: TextToken = TextToken {
    size: 12.0,
    line_height: 16.0,
    weight: Weight::Regular,
    face: Face::Mono,
    letter_spacing: 0.0,
};

impl TextToken {
    pub fn font_id(&self) -> FontId {
        FontId::new(self.size, fonts::family(self.face, self.weight))
    }

    /// `text` in this token's font, line height and letter spacing.
    pub fn rich(&self, text: impl Into<String>) -> RichText {
        RichText::new(text)
            .font(self.font_id())
            .line_height(Some(self.line_height))
            .extra_letter_spacing(self.letter_spacing)
    }
}

/// Names of the tokens egui has no built-in `TextStyle` for.
pub const CAPTION_STYLE: &str = "caption";
pub const BODY_STRONG_STYLE: &str = "body-strong";
pub const TITLE_STYLE: &str = "title";
pub const CODE_STYLE: &str = "code";

/// egui's text styles mapped onto the tokens (§11.5).
pub fn text_styles() -> BTreeMap<TextStyle, FontId> {
    [
        (TextStyle::Small, SMALL),
        (TextStyle::Body, BODY),
        (TextStyle::Button, BUTTON),
        (TextStyle::Heading, HEADLINE),
        (TextStyle::Monospace, MONO),
        (TextStyle::Name(CAPTION_STYLE.into()), CAPTION),
        (TextStyle::Name(BODY_STRONG_STYLE.into()), BODY_STRONG),
        (TextStyle::Name(TITLE_STYLE.into()), TITLE),
        (TextStyle::Name(CODE_STYLE.into()), CODE),
    ]
    .into_iter()
    .map(|(style, token)| (style, token.font_id()))
    .collect()
}
