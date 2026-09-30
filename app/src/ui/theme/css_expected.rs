//! What `design/tokens.css` must say, built from the Rust constants: the
//! color table of one theme and every scalar token as its CSS text.

use egui::Color32;

use super::metrics as m;
use super::motion;
use super::tokens::Colors;
use super::typography::{self as t, TextToken};
use crate::ui::fonts::{Face, Weight};

/// `cubic-out`, the curve `motion::easing` uses, as CSS writes it.
const CUBIC_OUT_CSS: &str = "cubic-bezier(0.33, 1, 0.68, 1)";

/// `(token name, color)` for every color token of one theme.
pub fn color_table(c: &Colors) -> Vec<(&'static str, Color32)> {
    vec![
        ("bg-canvas", c.bg_canvas),
        ("bg-surface", c.bg_surface),
        ("bg-sunken", c.bg_sunken),
        ("bg-hover", c.bg_hover),
        ("accent-soft", c.accent_soft),
        ("border", c.border),
        ("border-strong", c.border_strong),
        ("fg", c.fg),
        ("fg-muted", c.fg_muted),
        ("fg-subtle", c.fg_subtle),
        ("accent", c.accent),
        ("accent-hover", c.accent_hover),
        ("accent-pressed", c.accent_pressed),
        ("accent-fg", c.accent_fg),
        ("on-accent", c.on_accent),
        ("focus", c.focus),
        ("success", c.success),
        ("success-soft", c.success_soft),
        ("success-border", c.success_border),
        ("warning", c.warning),
        ("warning-soft", c.warning_soft),
        ("warning-border", c.warning_border),
        ("danger", c.danger),
        ("danger-soft", c.danger_soft),
        ("danger-border", c.danger_border),
        ("shadow-color", c.shadow_color),
    ]
}

fn px(value: f32) -> String {
    format!("{value}px")
}

/// A CSS length as written by hand: `0` or `4px`.
fn length(value: f32) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        px(value)
    }
}

fn ms(seconds: f32) -> String {
    format!("{}ms", (seconds * 1000.0).round())
}

fn font(token: TextToken) -> String {
    let weight = match token.weight {
        Weight::Regular => 400,
        Weight::Medium => 500,
        Weight::SemiBold => 600,
    };
    let face = match token.face {
        Face::Ui => "ui",
        Face::Mono => "mono",
    };
    format!(
        "{weight} {}px/{}px var(--ws-font-{face})",
        token.size, token.line_height
    )
}

/// Every non-color token of the base block, as its expected CSS text.
pub fn scalar_table() -> Vec<(String, String)> {
    let mut table: Vec<(String, String)> = [
        ("space-1", m::SPACE_1),
        ("space-2", m::SPACE_2),
        ("space-3", m::SPACE_3),
        ("space-4", m::SPACE_4),
        ("space-5", m::SPACE_5),
        ("space-6", m::SPACE_6),
        ("space-8", m::SPACE_8),
        ("radius-sm", f32::from(m::RADIUS_SM)),
        ("radius-md", f32::from(m::RADIUS_MD)),
        ("radius-lg", f32::from(m::RADIUS_LG)),
        ("control-sm", m::CONTROL_SM),
        ("control-md", m::CONTROL_MD),
        ("control-lg", m::CONTROL_LG),
        ("row-cert", m::ROW_CERT),
        ("row-compact", m::ROW_COMPACT),
        ("row-diag", m::ROW_DIAG),
        ("icon-sm", m::ICON_SM),
        ("icon-md", m::ICON_MD),
        ("icon-lg", m::ICON_LG),
        ("identicon", m::IDENTICON),
        ("sidebar", m::SIDEBAR),
        ("popup-width", m::POPUP_WIDTH),
        ("window-confirm-width", m::WINDOW_CONFIRM[0]),
        ("window-confirm-height", m::WINDOW_CONFIRM[1]),
        ("window-diagnostics-width", m::WINDOW_DIAGNOSTICS[0]),
        ("window-diagnostics-height", m::WINDOW_DIAGNOSTICS[1]),
        ("focus-width", m::FOCUS_WIDTH),
        ("focus-offset", m::FOCUS_OFFSET),
        ("code-letter-spacing", t::CODE.letter_spacing),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), px(value)))
    .collect();
    for (name, token) in [
        ("text-caption", t::CAPTION),
        ("text-small", t::SMALL),
        ("text-body", t::BODY),
        ("text-body-strong", t::BODY_STRONG),
        ("text-button", t::BUTTON),
        ("text-title", t::TITLE),
        ("text-headline", t::HEADLINE),
        ("text-code", t::CODE),
        ("text-mono", t::MONO),
    ] {
        table.push((name.to_owned(), font(token)));
    }
    for (name, seconds) in [
        ("motion-fast", motion::MOTION_FAST),
        ("motion-base", motion::MOTION_BASE),
        ("motion-slow", motion::MOTION_SLOW),
        ("delay-arming", motion::DELAY_ARMING.as_secs_f32()),
        ("delay-loading", motion::DELAY_LOADING.as_secs_f32()),
        ("delay-slow-hint", motion::DELAY_SLOW_HINT.as_secs_f32()),
        ("hold-success", motion::HOLD_SUCCESS.as_secs_f32()),
        (
            "hold-site-cancelled",
            motion::HOLD_SITE_CANCELLED.as_secs_f32(),
        ),
    ] {
        table.push((name.to_owned(), ms(seconds)));
    }
    let timeout = format!("{}s", motion::TIMEOUT_REQUEST.as_secs());
    table.push(("timeout-request".to_owned(), timeout));
    table.push(("easing".to_owned(), CUBIC_OUT_CSS.to_owned()));
    let [x, y] = m::SHADOW_POPOVER_OFFSET.map(|v| length(f32::from(v)));
    let blur = length(f32::from(m::SHADOW_POPOVER_BLUR));
    let shadow = format!("{x} {y} {blur} 0 var(--ws-shadow-color)");
    table.push(("shadow-popover".to_owned(), shadow));
    table
}
