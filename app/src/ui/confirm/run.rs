//! The event loop of a host process's window: host commands arrive between
//! frames (also while the window is hidden, through `App::logic`), the
//! window draws in `App::ui`, and the OS window follows its state.

use std::sync::mpsc::{Receiver, TryRecvError};

use websign_host::runtime::EventSender;
use websign_ui_model::confirm::UiCommand;

use super::clock::Clock;
use super::viewport::{self, Shown};
use super::window::ConfirmWindow;
use crate::ui::{bridge, i18n, renderer, theme};

/// See [`super::run`].
pub fn run(
    first: UiCommand,
    commands: &Receiver<UiCommand>,
    events: EventSender,
) -> Result<(), eframe::Error> {
    let options = viewport::native_options(renderer::choose());
    let result = eframe::run_native(
        websign_project::PRODUCT_NAME,
        options,
        Box::new(move |creation| {
            let installed = theme::install(creation, crate::platform::motion::reduce_motion());
            bridge::connect(&creation.egui_ctx);
            #[cfg(feature = "e2e")]
            crate::e2e::attach(&creation.egui_ctx, crate::e2e::Window::Confirm);
            let mut window = ConfirmWindow::new(installed, i18n::catalog(), events, Clock::System);
            window.apply(first);
            Ok(Box::new(App {
                window,
                commands,
                ended: false,
                shown: Shown::default(),
                native: None,
            }))
        }),
    );
    bridge::disconnect();
    result
}

struct App<'a> {
    window: ConfirmWindow,
    commands: &'a Receiver<UiCommand>,
    /// The engine dropped its end: close once nothing is on screen.
    ended: bool,
    shown: Shown,
    native: Option<isize>,
}

impl eframe::App for App<'_> {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.native.is_none() {
            self.native = viewport::native_handle(frame);
            bridge::set_native_handle(self.native);
        }
        loop {
            match self.commands.try_recv() {
                Ok(command) => self.window.apply(command),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.ended = true;
                    break;
                }
            }
        }
        let next = self.window.tick();
        if ctx.input(|input| input.viewport().close_requested()) && !self.ended {
            // The close button is Cancel (`docs/ux.md` §4.1); the window
            // itself only closes when the engine is done.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.window.close_button(ctx);
        }
        if self.ended && !self.window.is_holding() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.shown.sync(ctx, &self.window, self.native);
        if let Some(at) = next {
            ctx.request_repaint_after(at.saturating_duration_since(std::time::Instant::now()));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.window.frame(ui);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}
