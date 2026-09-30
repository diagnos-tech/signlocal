//! Saving what the window shows, in the light and then the dark theme:
//! pin the theme, let a few frames settle, ask the viewport for a
//! screenshot (`ViewportCommand::Screenshot`), keep the image the next
//! frame brings, then do the same in the other theme, save both and go back
//! to the system's. Files are written only for a whole pair, so a state
//! that leaves the screen halfway never leaves (or removes) a lone picture.
//!
//! A pair takes a dozen frames, which on a slow desktop outlasts a result
//! notice's hold (900 ms for success); [`in_progress`] lets the window keep
//! a notice up until its pair is saved, so every state gets its pictures.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use egui::{
    ColorImage, Context, Event, RawInput, Theme, ThemePreference, UserData, ViewportCommand,
};

/// Frames drawn in a newly pinned theme before its screenshot.
const SETTLE_FRAMES: u32 = 3;
/// A screenshot that never arrives (the window was hidden meanwhile) is
/// given up after this long, so the driver goes on.
const GIVE_UP: Duration = Duration::from_secs(3);

/// Whether a pair is being taken (one window per process).
static IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Whether a light/dark pair is being taken in this process.
pub fn in_progress() -> bool {
    IN_PROGRESS.load(Ordering::Relaxed)
}

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
    /// The light picture, kept until its dark one arrives.
    light: Option<Arc<ColorImage>>,
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
            light: None,
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
        self.light = None;
        ctx.set_theme(ThemePreference::System);
        self.set_step(Step::Idle);
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
                    self.light = None;
                    self.set_step(Step::Idle);
                }
            }
            Step::Settle(0) => {
                self.set_step(Step::Awaiting(Instant::now()));
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::new(self.theme)));
            }
            Step::Settle(n) => self.set_step(Step::Settle(n - 1)),
        }
        if self.is_busy() {
            ctx.request_repaint();
        }
    }

    /// Call with every frame's input: keeps an arrived light screenshot;
    /// with the dark one, saves the pair.
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
        match self.theme {
            Theme::Light => {
                self.light = Some(image);
                self.pin(ctx, Theme::Dark);
            }
            Theme::Dark => {
                if let Some(light) = self.light.take() {
                    self.save("light", &light);
                    self.save("dark", &image);
                }
                ctx.set_theme(ThemePreference::System);
                self.set_step(Step::Idle);
            }
        }
    }

    fn save(&self, theme: &str, image: &ColorImage) {
        if let Err(error) = super::png::write(&self.path(theme), image) {
            log::error!("e2e screenshot not saved: {error}");
        }
    }

    fn pin(&mut self, ctx: &Context, theme: Theme) {
        self.theme = theme;
        ctx.set_theme(theme);
        self.set_step(Step::Settle(SETTLE_FRAMES));
        ctx.request_repaint();
    }

    fn set_step(&mut self, step: Step) {
        self.step = step;
        IN_PROGRESS.store(step != Step::Idle, Ordering::Relaxed);
    }
}
