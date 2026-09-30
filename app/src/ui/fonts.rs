//! Embedded fonts: Inter 400/500/600 and a JetBrains Mono ASCII subset
//! (OFL-1.1, files in `app/assets/fonts/`), registered as named families
//! because egui does not synthesize weights (`docs/ux.md` §11.2).

/// Installs the fonts on `ctx`.
pub fn install(ctx: &egui::Context) {
    let _ = ctx;
    todo!("ui track: ux.md §11.2")
}
