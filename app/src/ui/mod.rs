//! The two windows (egui): confirmation and diagnostics.
//!
//! This layer only renders what `websign-ui-model` decides and turns egui
//! input into model input. Look and feel come from [`theme`] (the tokens of
//! `docs/ux.md` §11), [`fonts`] (Inter and JetBrains Mono, embedded),
//! [`icons`] (Phosphor) and [`widgets`] (one custom widget per file).

#[cfg(test)]
mod audit;
pub mod bridge;
pub mod confirm;
pub mod diagnostics;
pub mod fonts;
pub mod i18n;
pub mod icons;
pub mod renderer;
pub mod theme;
pub mod widgets;
