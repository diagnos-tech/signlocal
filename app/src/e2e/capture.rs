//! Saving what the window shows, in the light and then the dark theme:
//! pin the theme, let a few frames settle, ask the viewport for a
//! screenshot (`ViewportCommand::Screenshot`), save the image the next
//! frame brings, then do the same in the other theme and go back to the
//! system's.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Context, Event, RawInput, Theme, ThemePreference, UserData, ViewportCommand};

/// Frames drawn in a newly pinned theme before its screenshot.
const SETTLE_FRAMES: u32 = 3;
/// A screenshot that never arrives (the window was hidden meanwhile) is
/// given up after this long, so the driver goes on.
const GIVE_UP: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Idle,
    Settle(u32),
    Awaiting(Instant),
}

/// One window's screenshots.
#[derive(Debug)]
pub struct Capture {
    dir: PathBuf,
    window: &'static str,
    state: String,
    theme: Theme,
    step: Step,
}

impl Capture {
    /// Screenshots of `window` (`confirm`, `diagnostics`) go to `dir`.
    pub fn new(dir: PathBuf, window: &'static str) -> Capture {
        Capture {
            dir,
            window,
            state: String::new(),
            theme: Theme::Light,
            step: Step::Idle,
        }
    }

    /// Whether a light/dark pair is being taken.
    pub fn is_busy(&self) -> bool {
        self.step != Step::Idle
    }

    fn is_awaiting(&self) -> bool {
        matches!(self.step, Step::Awaiting(_))
    }

    /// Starts the pair for `state`.
    pub fn start(&mut self, ctx: &Context, state: &str) {
        self.state = state.to_owned();
        self.pin(ctx, Theme::Light);
    }

    /// Drops a pair whose state left the screen before both pictures were
    /// taken, so no file shows another state than its name says. Returns
    /// the dropped state.
    pub fn abandon_unless(&mut self, ctx: &Context, shown: Option<&str>) -> Option<String> {
        if !self.is_busy() || shown == Some(self.state.as_str()) {
            return None;
        }
        let _ = std::fs::remove_file(self.path("light"));
        ctx.set_theme(ThemePreference::System);
        self.step = Step::Idle;
        Some(std::mem::take(&mut self.state))
    }

    fn path(&self, theme: &str) -> PathBuf {
        self.dir
            .join(format!("{}-{}-{theme}.png", self.window, self.state))
    }

    /// Call at the end of every pass: asks for the screenshot once settled.
    pub fn end_pass(&mut self, ctx: &Context) {
        match self.step {
            Step::Idle => {}
            Step::Awaiting(since) => {
                if since.elapsed() > GIVE_UP {
                    log::warn!("e2e screenshot never arrived; skipped");
                    ctx.set_theme(ThemePreference::System);
                    self.step = Step::Idle;
                }
            }
            Step::Settle(0) => {
                self.step = Step::Awaiting(Instant::now());
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::new(self.theme)));
            }
            Step::Settle(n) => self.step = Step::Settle(n - 1),
        }
        if self.is_busy() {
            ctx.request_repaint();
        }
    }

    /// Call with every frame's input: saves an arrived screenshot.
    pub fn input(&mut self, ctx: &Context, input: &RawInput) {
        if !self.is_awaiting() {
            return;
        }
        let Some(image) = input.events.iter().find_map(|event| match event {
            Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        }) else {
            return;
        };
        let theme = match self.theme {
            Theme::Light => "light",
            Theme::Dark => "dark",
        };
        let path = self.path(theme);
        if let Err(error) = super::png::write(&path, &image) {
            log::error!("e2e screenshot not saved: {error}");
        }
        match self.theme {
            Theme::Light => self.pin(ctx, Theme::Dark),
            Theme::Dark => {
                ctx.set_theme(ThemePreference::System);
                self.step = Step::Idle;
            }
        }
    }

    fn pin(&mut self, ctx: &Context, theme: Theme) {
        self.theme = theme;
        ctx.set_theme(theme);
        self.step = Step::Settle(SETTLE_FRAMES);
        ctx.request_repaint();
    }
}
