//! Secure keyboard entry while our PIN field has the focus
//! (`docs/architecture/security.md` T6, `docs/ux.md` §4.6).
//!
//! On macOS this is Carbon's `EnableSecureEventInput`, which stops other
//! processes (keyloggers, event taps) from reading keystrokes. Windows and
//! Linux have no per-app equivalent, so there it does nothing.
//!
//! egui is immediate mode: a field that is not drawn says nothing, so the
//! field only asks, every pass it is focused ([`request`]), and a plugin
//! decides at the end of the pass. Anything that stops the asking turns it
//! off at the end of that same pass: focus moving elsewhere, the field
//! leaving the screen, the window losing focus or hiding (both end the
//! window's focus, which runs a pass). The plugin lives in the egui
//! `Context`, so closing the window or a panic unwinding past it drops the
//! guard, which turns it off too.

mod latch;

use egui::{Context, Ui, ViewportId};

#[cfg(target_os = "macos")]
use super::os::secure_input::Guard;

/// Nothing to turn on outside macOS.
#[cfg(not(target_os = "macos"))]
#[derive(Debug)]
struct Guard;

#[cfg(not(target_os = "macos"))]
impl Guard {
    fn enable() -> Option<Self> {
        Some(Self)
    }
}

/// Keeps secure keyboard entry on through the end of the current pass.
/// Call it every pass in which the PIN field has the keyboard focus and its
/// window is focused.
pub fn request(ctx: &Context) {
    let viewport = ctx.viewport_id();
    ctx.plugin_or_default::<SecureInput>().lock().requested_in = Some(viewport);
}

/// Whether secure keyboard entry is on for `ctx` (outside macOS: whether it
/// would be).
pub fn is_on(ctx: &Context) -> bool {
    ctx.plugin_opt::<SecureInput>()
        .is_some_and(|plugin| plugin.lock().latch.is_held())
}

/// The egui plugin that owns the guard.
#[derive(Debug, Default)]
struct SecureInput {
    /// The viewport whose pass asked, cleared at the end of that pass.
    requested_in: Option<ViewportId>,
    /// The viewport that holds the guard.
    holder: Option<ViewportId>,
    latch: latch::Latch<Guard>,
}

impl egui::Plugin for SecureInput {
    fn debug_name(&self) -> &'static str {
        "secure_input"
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let viewport = ui.ctx().viewport_id();
        if self.requested_in == Some(viewport) {
            self.requested_in = None;
            self.holder = Some(viewport);
            self.latch.set(true, Guard::enable);
        } else if self.holder == Some(viewport) {
            self.holder = None;
            self.latch.set(false, Guard::enable);
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{Context, RawInput};

    use super::{is_on, request};

    fn pass(ctx: &Context, ask: bool) {
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            if ask {
                request(ui.ctx());
            }
        });
    }

    #[test]
    fn on_while_asked_and_off_the_first_pass_without_asking() {
        let ctx = Context::default();
        assert!(!is_on(&ctx));
        pass(&ctx, true);
        pass(&ctx, true);
        assert!(is_on(&ctx));
        pass(&ctx, false);
        assert!(!is_on(&ctx));
        pass(&ctx, true);
        assert!(is_on(&ctx));
    }
}
