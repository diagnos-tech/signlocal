//! UI-probe: proves that the real app window can be rendered and captured on
//! every CI target, and provides the renderer-fallback logic meant to be
//! promoted into the app.

pub mod app;
pub mod renderer;
pub mod screenshot;
pub mod ui;

/// Fixed window size in logical points; keeps screenshots comparable.
pub const WINDOW_SIZE: [f32; 2] = [480.0, 360.0];
