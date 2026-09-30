//! The diagnostics window (`docs/ux.md` §8), 760 × 540: Browsers, Devices,
//! Certificates, Help.
//!
//! One scan of this computer ([`facts`]) feeds everything: the tabs, the
//! traffic lights ([`lights`], decided by `websign-ui-model`) and the exact
//! "Copy diagnostics" text ([`report`]). The scan runs off the UI thread
//! ([`services`]) and again when devices change ([`live`]); drawing
//! ([`view`]) only reads state and returns actions, which [`apply`] carries
//! out after the frame. `websign doctor` prints the same report through
//! [`report::terminal_input`].

mod apply;
mod counts;
pub mod facts;
mod keys;
mod lights;
mod live;
mod pick;
mod remembered;
pub mod report;
mod services;
mod state;
mod view;
mod window;

#[cfg(test)]
mod tests;

use egui::ViewportBuilder;
use websign_host::store::{DiskStores, MemoryStores, Stores, data_dir};
use websign_i18n::k;
use websign_protocol::messages::DiagnosticsTab;

pub use window::DiagnosticsWindow;

use crate::ui::theme::{self, metrics};
use crate::ui::{i18n, renderer};

/// Runs the diagnostics window on the calling (main) thread until it is
/// closed, on `tab` or else the first tab with a problem. The app creator
/// calls `theme::install(creation, platform::motion::reduce_motion())` and
/// starts the backend of `renderer::choose()`; the caller handles a
/// renderer failure (`cli::diagnostics`: glow re-exec).
pub fn run(tab: Option<DiagnosticsTab>) -> Result<(), eframe::Error> {
    let catalog = i18n::catalog();
    let title = catalog.tr(k::DIAG_WINDOW_TITLE).to_string();
    eframe::run_native(
        websign_project::PRODUCT_NAME,
        native_options(&title),
        Box::new(move |creation| {
            let installed = theme::install(creation, crate::platform::motion::reduce_motion());
            let ctx = creation.egui_ctx.clone();
            let scale = (ctx.pixels_per_point() * 100.0).round() as u32;
            let dir = data_dir();
            let stores: Box<dyn Stores> = match &dir {
                Some(dir) => Box::new(DiskStores::new(dir)),
                None => Box::new(MemoryStores::new()),
            };
            let parts = window::Parts {
                catalog,
                environment: report::Environment::of_this_app(scale, renderer::describe(creation)),
                stores,
                os: Box::new(services::RealOs),
                scanner: Box::new(services::ThreadScanner::default()),
                data_dir: dir,
                zone: jiff::tz::TimeZone::system(),
                clock: Box::new(jiff::Timestamp::now),
                tab,
                live: Some(live::Live::start(move || ctx.request_repaint())),
            };
            Ok(Box::new(App(DiagnosticsWindow::new(parts, installed))))
        }),
    )
}

/// 760 × 540, fixed; minimize allowed, maximize not (§8.1).
fn native_options(title: &str) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer: renderer::choose().eframe(),
        viewport: ViewportBuilder::default()
            .with_title(title)
            .with_app_id(websign_project::SLUG)
            .with_inner_size(metrics::WINDOW_DIAGNOSTICS)
            .with_resizable(false)
            .with_maximize_button(false),
        centered: true,
        persist_window: false,
        ..Default::default()
    }
}

struct App(DiagnosticsWindow);

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.0.owner.is_none() {
            self.0.set_owner(dialog_owner(frame));
        }
        self.0.show(ui);
    }
}

/// The handle OS dialogs take as their owner: the `HWND` on Windows. The
/// macOS panels are app-modal and the Linux ones separate programs, so they
/// need none.
fn dialog_owner(frame: &eframe::Frame) -> Option<isize> {
    use eframe::egui_wgpu::wgpu::rwh::{HasWindowHandle, RawWindowHandle};
    match frame.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(win) => Some(win.hwnd.get()),
        _ => None,
    }
}
