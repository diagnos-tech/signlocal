//! The design tokens of `docs/ux.md` §11 and their mapping onto egui
//! `Visuals`/`Style` (§11.5). A test compares every `--ws-*` token of
//! `design/tokens.css` with the constants here.

pub mod tokens;

/// Applies the light or dark theme to `ctx`.
pub fn apply(ctx: &egui::Context, dark: bool) {
    let _ = (ctx, dark);
    todo!("ui track: ux.md §11.5")
}
