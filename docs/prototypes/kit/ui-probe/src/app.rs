//! The eframe application: draws the UI, captures one screenshot after a few
//! frames, then closes the window.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{ViewportCommand, ViewportId};

use crate::renderer::{Backend, Used};
use crate::screenshot;
use crate::ui::{self, UiState};

/// Frames rendered before the screenshot is requested, so fonts and layout
/// have settled.
const WARMUP_FRAMES: u32 = 5;

/// Outcome shared with `main` once the window has closed.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Renderer detected at creation.
    pub used: Option<Used>,
    /// Screenshot error, if any.
    pub error: Option<String>,
    /// True once the PNG has been written.
    pub saved: bool,
}

/// The probe application.
#[derive(Debug)]
pub struct ProbeApp {
    out: PathBuf,
    label: String,
    state: UiState,
    frames: u32,
    requested: bool,
    outcome: Arc<Mutex<Outcome>>,
}

impl ProbeApp {
    /// Builds the app from the creation context, detecting the renderer.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        backend: Backend,
        out: PathBuf,
        outcome: Arc<Mutex<Outcome>>,
    ) -> Self {
        ui::install_fonts(&cc.egui_ctx);
        let description = describe(cc, backend);
        let label = format!("{backend} / {description}");
        outcome.lock().expect("outcome lock").used = Some(Used {
            backend,
            description,
        });
        Self {
            out,
            label,
            state: UiState::default(),
            frames: 0,
            requested: false,
            outcome,
        }
    }

    fn finish(&self, result: Result<(), screenshot::Error>) {
        let mut outcome = self.outcome.lock().expect("outcome lock");
        match result {
            Ok(()) => outcome.saved = true,
            Err(e) => outcome.error = Some(e.to_string()),
        }
    }
}

impl eframe::App for ProbeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui::draw(ui, &mut self.state, &self.label);
        self.frames += 1;
        let ctx = ui.ctx().clone();
        if self.frames >= WARMUP_FRAMES && !self.requested {
            self.requested = true;
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|event| match event {
                egui::Event::Screenshot {
                    viewport_id, image, ..
                } if *viewport_id == ViewportId::ROOT => Some(Arc::clone(image)),
                _ => None,
            })
        });
        if let Some(image) = shot {
            self.finish(screenshot::save_png(&image, &self.out));
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        // Keep frames flowing even when nothing changes.
        ctx.request_repaint();
    }
}

/// Human-readable renderer/adapter description.
fn describe(cc: &eframe::CreationContext<'_>, backend: Backend) -> String {
    match backend {
        Backend::Wgpu => cc.wgpu_render_state.as_ref().map_or_else(
            || "unknown wgpu adapter".to_owned(),
            |state| {
                let info = state.adapter.get_info();
                format!("{} ({:?}, {:?})", info.name, info.backend, info.device_type)
            },
        ),
        Backend::Glow => cc.gl.as_ref().map_or_else(
            || "unknown GL context".to_owned(),
            |gl| {
                use eframe::glow::HasContext as _;
                // SAFETY: querying a static string on the current context.
                unsafe { gl.get_parameter_string(eframe::glow::RENDERER) }
            },
        ),
    }
}
