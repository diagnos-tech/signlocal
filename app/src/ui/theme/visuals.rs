//! The egui `Style` built from the tokens, following the table of
//! `docs/ux.md` §11.5 row by row.

use egui::style::{Selection, TextCursorStyle, WidgetVisuals, Widgets};
use egui::{CornerRadius, Shadow, Stroke, Style, Vec2, Visuals};

use super::metrics::{
    CONTROL_MD, ICON_SM, RADIUS_LG, RADIUS_MD, SHADOW_POPOVER_BLUR, SHADOW_POPOVER_OFFSET, SPACE_2,
    SPACE_3, SPACE_4,
};
use super::motion::MOTION_FAST;
use super::tokens::Colors;
use super::typography;

/// The complete style of one theme.
pub fn style(colors: &Colors, dark: bool) -> Style {
    let mut style = Style {
        visuals: visuals(colors, dark),
        text_styles: typography::text_styles(),
        animation_time: MOTION_FAST,
        ..Style::default()
    };
    let spacing = &mut style.spacing;
    spacing.item_spacing = Vec2::splat(SPACE_2);
    spacing.button_padding = Vec2::new(SPACE_3, 0.0);
    spacing.interact_size = Vec2::new(32.0, CONTROL_MD);
    spacing.icon_width = ICON_SM;
    spacing.icon_width_inner = ICON_SM / 2.0;
    spacing.icon_spacing = SPACE_2;
    spacing.indent = SPACE_4;
    style
}

fn visuals(colors: &Colors, dark: bool) -> Visuals {
    let base = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    Visuals {
        dark_mode: dark,
        widgets: widgets(colors),
        selection: Selection {
            bg_fill: colors.accent_soft,
            stroke: Stroke::new(1.0, colors.accent_fg),
        },
        hyperlink_color: colors.accent_fg,
        faint_bg_color: colors.bg_hover,
        extreme_bg_color: colors.bg_sunken,
        text_edit_bg_color: Some(colors.bg_sunken),
        code_bg_color: colors.bg_sunken,
        warn_fg_color: colors.warning,
        error_fg_color: colors.danger,
        window_corner_radius: CornerRadius::same(RADIUS_LG),
        // The OS draws the window shadow; a second one would double it.
        window_shadow: Shadow::NONE,
        window_fill: colors.bg_surface,
        window_stroke: Stroke::new(1.0, colors.border),
        menu_corner_radius: CornerRadius::same(RADIUS_MD),
        panel_fill: colors.bg_canvas,
        popup_shadow: Shadow {
            offset: SHADOW_POPOVER_OFFSET,
            blur: SHADOW_POPOVER_BLUR,
            spread: 0,
            color: colors.shadow_color,
        },
        text_cursor: TextCursorStyle {
            stroke: Stroke::new(2.0, colors.accent_fg),
            ..base.text_cursor
        },
        ..base
    }
}

fn widgets(colors: &Colors) -> Widgets {
    let radius = CornerRadius::same(RADIUS_MD);
    let visual = |fill, stroke| WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: stroke,
        corner_radius: radius,
        fg_stroke: Stroke::new(1.0, colors.fg),
        expansion: 0.0,
    };
    let hovered = visual(colors.bg_hover, Stroke::new(1.0, colors.border_strong));
    Widgets {
        noninteractive: visual(colors.bg_surface, Stroke::new(1.0, colors.border)),
        inactive: visual(colors.bg_surface, Stroke::new(1.0, colors.border_strong)),
        hovered,
        active: visual(colors.bg_sunken, Stroke::new(1.0, colors.fg_subtle)),
        open: hovered,
    }
}
