//! The OS window around the confirmation (`docs/ux.md` §4.1): 480 × 600,
//! fixed, centered, no minimize or maximize, on top while it waits for a
//! decision, hidden between requests.
//!
//! Showing a request raises the window and asks for focus through the
//! platform layer, but arming never trusts that: it starts from the
//! window's real focus events (`UserInput::Focus`), so a window the OS kept
//! in the background stays unarmed until the person brings it forward.
//! While the OS shows its own PIN dialog (a child of this window) the
//! window leaves the top level, so the dialog is not covered.

use egui::{Context, ViewportBuilder, ViewportCommand, WindowLevel};
use websign_ui_model::confirm::ConfirmState;
use websign_ui_model::confirm::view::PinBlock;

use super::window::ConfirmWindow;
use crate::ui::renderer::Backend;
use crate::ui::theme::metrics;

/// The window as first created: visible, since `run` starts with a request.
pub fn native_options(backend: Backend) -> eframe::NativeOptions {
    let [width, height] = metrics::WINDOW_CONFIRM;
    eframe::NativeOptions {
        renderer: backend.eframe(),
        viewport: ViewportBuilder::default()
            .with_title(websign_project::PRODUCT_NAME)
            .with_app_id(websign_project::SLUG)
            .with_inner_size([width, height])
            .with_resizable(false)
            .with_minimize_button(false)
            .with_maximize_button(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_active(true),
        centered: true,
        persist_window: false,
        ..Default::default()
    }
}

/// What was last asked of the OS window, so commands go out on change only.
#[derive(Debug, Default)]
pub struct Shown {
    visible: Option<bool>,
    title: Option<String>,
    on_top: Option<bool>,
}

impl Shown {
    /// Brings the OS window in line with `window`.
    pub fn sync(&mut self, ctx: &Context, window: &ConfirmWindow, native: Option<isize>) {
        let visible = !window.is_idle();
        if self.visible != Some(visible) {
            self.visible = Some(visible);
            ctx.send_viewport_cmd(ViewportCommand::Visible(visible));
            if visible {
                raise(ctx, native);
            }
        }
        if let Some(title) = window
            .title()
            .filter(|title| self.title.as_ref() != Some(title))
        {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = Some(title);
        }
        let view = window.view();
        let os_prompt = view.state == ConfirmState::Signing
            && matches!(view.pin, PinBlock::OsPrompt { now: true, .. });
        let on_top = visible && !os_prompt;
        if self.on_top != Some(on_top) {
            self.on_top = Some(on_top);
            let level = if on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
        }
    }
}

/// Asks for the foreground; when the OS refuses, the person is told the
/// window wants attention (taskbar flash, dock bounce).
fn raise(ctx: &Context, native: Option<isize>) {
    ctx.send_viewport_cmd(ViewportCommand::Focus);
    let focused = native.is_some_and(crate::platform::focus::bring_to_front);
    if !focused {
        ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
            egui::UserAttentionType::Critical,
        ));
    }
}

/// The native handle of the window, in the form `platform::focus` and the
/// OS PIN dialogs take (HWND, NSView, X11 window).
pub fn native_handle(frame: &eframe::Frame) -> Option<isize> {
    use eframe::egui_wgpu::wgpu::rwh::{HasWindowHandle, RawWindowHandle};
    let handle = frame.window_handle().ok()?;
    match handle.as_raw() {
        #[cfg(windows)]
        RawWindowHandle::Win32(win) => Some(win.hwnd.get()),
        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(app) => Some(app.ns_view.as_ptr() as isize),
        RawWindowHandle::Xlib(x) => isize::try_from(x.window).ok(),
        RawWindowHandle::Xcb(x) => isize::try_from(x.window.get()).ok(),
        _ => None,
    }
}
