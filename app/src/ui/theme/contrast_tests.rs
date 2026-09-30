//! WCAG 2.x contrast of the pairs `docs/ux.md` §11.1 promises (AA: 4.5 for
//! text, 3.0 for components and focus), in both themes, so a token edit
//! cannot silently break accessibility (§14).

use egui::Color32;

use super::tokens::{self, Colors, IDENTICON};

fn luminance(color: Color32) -> f64 {
    let channel = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

fn contrast(a: Color32, b: Color32) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

const TEXT: f64 = 4.5;
const COMPONENT: f64 = 3.0;

/// `(what, foreground, background, minimum)`: every text and component
/// color the widgets draw, on every background it is drawn on.
fn pairs(c: &Colors) -> Vec<(String, Color32, Color32, f64)> {
    let fixed = [
        // Status text on rows and chips, and the PIN error under the field.
        ("success on bg-surface", c.success, c.bg_surface, TEXT),
        ("warning on bg-surface", c.warning, c.bg_surface, TEXT),
        ("danger on bg-surface", c.danger, c.bg_surface, TEXT),
        ("danger on bg-canvas", c.danger, c.bg_canvas, TEXT),
        ("success on success-soft", c.success, c.success_soft, TEXT),
        ("warning on warning-soft", c.warning, c.warning_soft, TEXT),
        ("danger on danger-soft", c.danger, c.danger_soft, TEXT),
        // Notices: full-contrast text on their soft fills.
        ("fg on success-soft", c.fg, c.success_soft, TEXT),
        ("fg on warning-soft", c.fg, c.warning_soft, TEXT),
        ("fg on danger-soft", c.fg, c.danger_soft, TEXT),
        // The primary button in all its pointer states.
        ("on-accent on accent", c.on_accent, c.accent, TEXT),
        (
            "on-accent on accent-hover",
            c.on_accent,
            c.accent_hover,
            TEXT,
        ),
        (
            "on-accent on accent-pressed",
            c.on_accent,
            c.accent_pressed,
            TEXT,
        ),
        // Links, ghost buttons, selected tab, info icon.
        ("accent-fg on bg-canvas", c.accent_fg, c.bg_canvas, TEXT),
        // Component outlines (fields, radios, checkboxes) and the focus ring
        // against what surrounds them.
        (
            "border-strong on bg-surface",
            c.border_strong,
            c.bg_surface,
            COMPONENT,
        ),
        (
            "border-strong on bg-canvas",
            c.border_strong,
            c.bg_canvas,
            COMPONENT,
        ),
        (
            "border-strong on bg-hover",
            c.border_strong,
            c.bg_hover,
            COMPONENT,
        ),
        (
            "on-accent dot on accent (checked radio)",
            c.on_accent,
            c.accent,
            COMPONENT,
        ),
    ];
    let mut pairs: Vec<_> = fixed
        .into_iter()
        .map(|(what, fg, bg, minimum)| (what.to_owned(), fg, bg, minimum))
        .collect();
    // Text drawn on every background a row, tab, field or card can have.
    let backgrounds = [
        ("bg-surface", c.bg_surface),
        ("bg-canvas", c.bg_canvas),
        ("bg-sunken", c.bg_sunken),
        ("bg-hover", c.bg_hover),
        ("accent-soft", c.accent_soft),
    ];
    for (name, background) in backgrounds {
        for (what, fg) in [
            ("fg", c.fg),
            ("fg-muted", c.fg_muted),
            ("fg-subtle", c.fg_subtle),
            ("accent-fg", c.accent_fg),
        ] {
            pairs.push(((format!("{what} on {name}")), fg, background, TEXT));
        }
        pairs.push(((format!("focus on {name}")), c.focus, background, COMPONENT));
    }
    // Validity text on the selected row.
    for (what, fg) in [
        ("success", c.success),
        ("warning", c.warning),
        ("danger", c.danger),
    ] {
        pairs.push(((format!("{what} on accent-soft")), fg, c.accent_soft, TEXT));
    }
    pairs
}

fn check(colors: &Colors, theme: &str) {
    for (what, fg, bg, minimum) in pairs(colors) {
        let ratio = contrast(fg, bg);
        assert!(
            ratio >= minimum,
            "{theme}: {what} is {ratio:.2}, needs {minimum}"
        );
    }
}

#[test]
fn light_theme_meets_aa() {
    check(&tokens::light(), "light");
}

#[test]
fn dark_theme_meets_aa() {
    check(&tokens::dark(), "dark");
}

#[test]
fn identicon_colors_stand_out_in_both_themes() {
    for colors in [tokens::light(), tokens::dark()] {
        for (index, id) in IDENTICON.iter().enumerate() {
            for background in [colors.bg_surface, colors.bg_sunken] {
                let ratio = contrast(*id, background);
                assert!(ratio >= 3.0, "id-{index} is {ratio:.2} on {background:?}");
            }
        }
    }
}

#[test]
fn measures_known_ratios() {
    assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 1e-9);
    let light = tokens::light();
    assert!((contrast(light.fg, light.bg_surface) - 17.9).abs() < 0.1);
}
