//! Embedded fonts: Inter 400/500/600 and a JetBrains Mono ASCII subset
//! (OFL-1.1, files and licenses in `app/assets/fonts/`), registered as named
//! families because egui does not synthesize weights (`docs/ux.md` §11.2).
//!
//! Every family falls back to the Phosphor regular icons (so icons can sit
//! inside any label) and then to egui's bundled fonts (glyphs outside our
//! subsets, e.g. CJK in a certificate name, still render).

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

use super::icons;

/// A type face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Inter: all interface text.
    Ui,
    /// JetBrains Mono: codes, paths, IDs (tells 0/O and 1/l apart).
    Mono,
}

/// A weight the embedded files provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Regular,
    Medium,
    SemiBold,
}

const INTER_REGULAR: &str = "inter-regular";
const INTER_MEDIUM: &str = "inter-medium";
const INTER_SEMIBOLD: &str = "inter-semibold";
const MONO_REGULAR: &str = "mono-regular";
const MONO_MEDIUM: &str = "mono-medium";

/// `(font name, file)`; the name doubles as the family name of weights that
/// are not egui's default `Proportional`/`Monospace`.
const FILES: [(&str, &[u8]); 5] = [
    (
        INTER_REGULAR,
        include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
    ),
    (
        INTER_MEDIUM,
        include_bytes!("../../assets/fonts/Inter-Medium.ttf"),
    ),
    (
        INTER_SEMIBOLD,
        include_bytes!("../../assets/fonts/Inter-SemiBold.ttf"),
    ),
    (
        MONO_REGULAR,
        include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
    ),
    (
        MONO_MEDIUM,
        include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf"),
    ),
];

/// The egui family for `face` at `weight`. The mono subset has no 600, so
/// SemiBold mono uses Medium.
pub fn family(face: Face, weight: Weight) -> FontFamily {
    match (face, weight) {
        (Face::Ui, Weight::Regular) => FontFamily::Proportional,
        (Face::Ui, Weight::Medium) => FontFamily::Name(INTER_MEDIUM.into()),
        (Face::Ui, Weight::SemiBold) => FontFamily::Name(INTER_SEMIBOLD.into()),
        (Face::Mono, Weight::Regular) => FontFamily::Monospace,
        (Face::Mono, Weight::Medium | Weight::SemiBold) => FontFamily::Name(MONO_MEDIUM.into()),
    }
}

/// Our fonts and icons on top of egui's defaults.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in FILES {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    let proportional_fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let families = [
        (FontFamily::Proportional, INTER_REGULAR),
        (family(Face::Ui, Weight::Medium), INTER_MEDIUM),
        (family(Face::Ui, Weight::SemiBold), INTER_SEMIBOLD),
        (FontFamily::Monospace, MONO_REGULAR),
        (family(Face::Mono, Weight::Medium), MONO_MEDIUM),
    ];
    for (family, primary) in families {
        let mut chain = vec![primary.to_owned()];
        chain.extend(proportional_fallbacks.iter().cloned());
        fonts.families.insert(family, chain);
    }
    icons::add_to(&mut fonts);
    fonts
}

/// Installs the fonts, with the icons, on `ctx`. Windows go through
/// [`super::theme::install`], which also proves it ran.
///
/// Fonts and icons go in together because the icon font must sit right
/// after the text font of every family (before egui's emoji fonts, which use
/// the same private-use code points), and `Context::add_font` can only put a
/// font first or last.
pub(crate) fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_starts_with_its_own_font_then_icons() {
        let fonts = definitions();
        for (family, primary) in [
            (family(Face::Ui, Weight::Regular), INTER_REGULAR),
            (family(Face::Ui, Weight::SemiBold), INTER_SEMIBOLD),
            (family(Face::Mono, Weight::Medium), MONO_MEDIUM),
        ] {
            let chain = &fonts.families[&family];
            assert_eq!(chain[0], primary);
            assert_eq!(chain[1], icons::REGULAR_FONT);
        }
    }

    #[test]
    fn embedded_files_are_truetype() {
        for (name, bytes) in FILES {
            assert_eq!(bytes.get(..4), Some(&[0, 1, 0, 0][..]), "{name}");
        }
    }
}
