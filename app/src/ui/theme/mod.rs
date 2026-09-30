//! The design tokens of `docs/ux.md` §11 and their mapping onto egui
//! `Visuals`/`Style` (§11.5). A test compares every `--ws-*` token of
//! `design/tokens.css` with the constants here.
//!
//! Both themes are always installed; egui picks one from the OS
//! (`ThemePreference::System`) and switches live when the OS does. Widgets
//! read their colors through [`colors`], never by hard-coding a theme.

pub mod metrics;
pub mod motion;
pub mod tokens;
pub mod typography;
mod visuals;

#[cfg(test)]
mod contrast_tests;
#[cfg(test)]
mod css_expected;
#[cfg(test)]
mod css_parse;
#[cfg(test)]
mod css_tests;

use egui::{Context, Theme, ThemePreference};

pub use tokens::Colors;

/// Proof that [`install`] ran: fonts, icons and both themes are in place.
///
/// Our text styles name font families (`inter-medium`, …) that only exist
/// once installed, and egui **panics** when a frame asks for a family it
/// does not know. Window constructors take this token, so a window cannot
/// be built on a context that was never prepared.
#[derive(Debug, Clone, Copy)]
#[must_use = "pass it to the window you are creating"]
pub struct Installed(());

/// Everything a window needs before its first frame: fonts, icons, both
/// themes following the OS, and the OS "reduce motion" choice
/// (`crate::platform::motion::reduce_motion`).
///
/// It takes eframe's `CreationContext`, which only exists inside the app
/// creator passed to `eframe::run_native`: the one moment before the first
/// frame. Fonts set there are in place when that frame starts; set later,
/// they would only apply a frame after the widgets that need them.
pub fn install(creation: &eframe::CreationContext<'_>, reduce_motion: bool) -> Installed {
    install_on(&creation.egui_ctx, reduce_motion)
}

fn install_on(ctx: &Context, reduce_motion: bool) -> Installed {
    crate::ui::fonts::install(ctx);
    install_styles(ctx);
    ctx.set_theme(ThemePreference::System);
    motion::set_reduced(ctx, reduce_motion);
    Installed(())
}

/// Test harnesses (no `CreationContext`): installs everything on `ctx` with
/// a pinned theme and reduced motion (still snapshots). The caller must
/// then skip drawing for one frame, as the fonts apply from the next.
#[cfg(test)]
pub fn install_for_tests(ctx: &Context, dark: bool) -> Installed {
    let installed = install_on(ctx, true);
    apply(ctx, dark);
    installed
}

/// Applies the light or dark theme to `ctx`, pinning it instead of following
/// the OS (snapshot tests, and a future "theme" setting).
pub fn apply(ctx: &Context, dark: bool) {
    install_styles(ctx);
    ctx.set_theme(if dark { Theme::Dark } else { Theme::Light });
}

fn install_styles(ctx: &Context) {
    ctx.set_style_of(Theme::Light, visuals::style(&tokens::light(), false));
    ctx.set_style_of(Theme::Dark, visuals::style(&tokens::dark(), true));
    // Fresh styles carry the default animation time; keep a reduced one.
    motion::set_reduced(ctx, motion::reduced(ctx));
}

/// The colors of the theme `ctx` is showing now.
pub fn colors(ctx: &Context) -> Colors {
    tokens::of(ctx.theme() == Theme::Dark)
}
