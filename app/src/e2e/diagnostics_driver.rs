//! The diagnostics window's e2e driver: once the scan has finished and the
//! tab stopped changing, save it in both themes and close the window. The
//! suite opens one window per tab (`websign diagnostics --tab <tab>`).

use std::time::{Duration, Instant};

use egui::{Context, FullOutput, RawInput, Ui, ViewportCommand};

use super::E2eConfig;
use super::capture::Capture;
use super::tree::Widgets;

/// The scan runs off the UI thread; nothing is saved before this long.
const MIN_OPEN: Duration = Duration::from_secs(2);
/// And not before the tab has looked the same for this long.
const STABLE: Duration = Duration::from_millis(1500);
/// A tab that never settles (a live device list) is saved anyway.
const MAX_WAIT: Duration = Duration::from_secs(20);

/// The egui plugin.
#[derive(Debug)]
pub struct DiagnosticsDriver {
    capture: Option<Capture>,
    tab: String,
    widgets: Widgets,
    opened: Instant,
    /// What the tab showed last and since when.
    seen: (u64, Instant),
    started: bool,
}

impl DiagnosticsDriver {
    pub fn new(config: E2eConfig) -> DiagnosticsDriver {
        let now = Instant::now();
        DiagnosticsDriver {
            capture: config
                .screenshots
                .map(|dir| Capture::new(dir, "diagnostics")),
            tab: tab_argument(std::env::args()).unwrap_or_else(|| "home".to_owned()),
            widgets: Widgets::default(),
            opened: now,
            seen: (0, now),
            started: false,
        }
    }
}

/// The value of `--tab` (`--tab x` or `--tab=x`).
fn tab_argument(args: impl Iterator<Item = String>) -> Option<String> {
    let mut args = args.skip_while(|arg| arg != "--tab" && !arg.starts_with("--tab="));
    let flag = args.next()?;
    match flag.strip_prefix("--tab=") {
        Some(value) => Some(value.to_owned()),
        None => args.next(),
    }
}

impl egui::Plugin for DiagnosticsDriver {
    fn debug_name(&self) -> &'static str {
        "websign-e2e-diagnostics"
    }

    fn input_hook(&mut self, ctx: &Context, input: &mut RawInput) {
        if let Some(capture) = &mut self.capture {
            capture.input(ctx, input);
        }
    }

    fn output_hook(&mut self, _ctx: &Context, output: &mut FullOutput) {
        self.widgets
            .update(output.platform_output.accesskit_update.as_ref());
        let digest = self.widgets.digest();
        if digest != self.seen.0 {
            self.seen = (digest, Instant::now());
        }
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let Some(capture) = &mut self.capture else {
            return;
        };
        capture.end_pass(&ctx);
        ctx.request_repaint_after(Duration::from_millis(200));
        if capture.is_busy() {
            return;
        }
        if self.started {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        let open = self.opened.elapsed();
        let settled = open >= MIN_OPEN && self.seen.1.elapsed() >= STABLE;
        if settled || open >= MAX_WAIT {
            capture.start(&ctx, &self.tab);
            self.started = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tab_argument;

    fn args(list: &[&str]) -> impl Iterator<Item = String> {
        list.iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn reads_the_tab_flag() {
        assert_eq!(
            tab_argument(args(&["websign", "diagnostics", "--tab", "help"])).as_deref(),
            Some("help")
        );
        assert_eq!(
            tab_argument(args(&["websign", "diagnostics", "--tab=devices"])).as_deref(),
            Some("devices")
        );
        assert_eq!(tab_argument(args(&["websign", "diagnostics"])), None);
    }
}
