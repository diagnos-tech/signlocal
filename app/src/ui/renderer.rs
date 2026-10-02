//! Which renderer draws the windows (evidence: `docs/prototypes/5-ui-screenshots.md`).
//!
//! * **wgpu** is the renderer everywhere. It also covers RDP and GPU-less VMs:
//!   on Windows it falls back to DX12 WARP ("Microsoft Basic Render Driver"),
//!   on Linux to Vulkan llvmpipe, on macOS it uses Metal.
//! * **glow** is a fallback on macOS and Linux only: on Windows it fails
//!   ("egui_glow requires opengl 2.0+"). The fallback is a **re-exec** of the
//!   same process arguments with `WEBSIGN_RENDERER=glow`, never an in-process
//!   retry (a second event loop is only proven to work on X11).
//!
//! `WEBSIGN_RENDERER=wgpu|glow` forces one. The backend that drew is reported
//! in diagnostics (`render: wgpu/dx12`, `render: glow`).
//!
//! **Startup order.** A re-exec is only safe before stdin is read, but a
//! host process opens its first window after reading a request. So the
//! choice is made without re-executing, from facts available at start: the
//! forced variable, else a "glow needed" marker in the app's data folder,
//! else wgpu. The marker is written the first time wgpu fails for a renderer
//! reason on macOS or Linux ([`remember_glow_needed`]):
//!
//! * the diagnostics window (nothing read from stdin yet) records it and
//!   re-executes with glow at once;
//! * a host process records it and fails the requests on screen with
//!   `Internal` (the window could not be shown); every later launch on this
//!   machine starts with glow directly.

use std::process::Command;

/// A concrete backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Wgpu,
    Glow,
}

impl Backend {
    /// The eframe renderer to start.
    pub fn eframe(self) -> eframe::Renderer {
        match self {
            Backend::Wgpu => eframe::Renderer::Wgpu,
            Backend::Glow => eframe::Renderer::Glow,
        }
    }

    /// The value of [`RENDERER_ENV`] that selects this backend.
    pub fn name(self) -> &'static str {
        match self {
            Backend::Wgpu => "wgpu",
            Backend::Glow => "glow",
        }
    }
}

/// Environment variable that forces a backend (and marks a re-exec).
pub const RENDERER_ENV: &str = "WEBSIGN_RENDERER";

/// The backend for this process: the forced one, else glow when this
/// machine needed it before (macOS and Linux), else wgpu. Reads a file and
/// the environment only, so it may run at any point of a host's life.
pub fn choose() -> Backend {
    forced().unwrap_or_else(|| {
        let remembered = cfg!(any(target_os = "macos", target_os = "linux"))
            && glow_marker().is_some_and(|marker| marker.exists());
        if remembered {
            Backend::Glow
        } else {
            Backend::Wgpu
        }
    })
}

/// Records that wgpu failed here, so later launches start with glow.
/// Best effort: without a data folder every launch tries wgpu first.
pub fn remember_glow_needed() {
    let Some(marker) = glow_marker() else {
        return;
    };
    let written = marker
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&marker, b"wgpu failed on this machine\n"));
    match written {
        Ok(()) => log::warn!("wgpu failed: later launches start with glow"),
        Err(error) => log::warn!("could not record the glow fallback: {:?}", error.kind()),
    }
}

/// The marker file: `renderer-glow` in the app's data folder.
fn glow_marker() -> Option<std::path::PathBuf> {
    websign_host::store::data_dir().map(|dir| dir.join("renderer-glow"))
}

/// Whether a failed wgpu start may be retried by re-executing with glow:
/// macOS and Linux, and only when glow was not already forced.
pub fn may_fall_back_to_glow() -> bool {
    fallback_allowed(
        cfg!(any(target_os = "macos", target_os = "linux")),
        forced(),
    )
}

/// Whether `error` came from the graphics stack, as opposed to the window
/// system or the app: only then can another renderer succeed. A missing
/// display (e.g. no X11 libraries) fails the same way under glow.
pub fn is_renderer_failure(error: &eframe::Error) -> bool {
    matches!(
        error,
        eframe::Error::Wgpu(_)
            | eframe::Error::Glutin(_)
            | eframe::Error::NoGlutinConfigs(..)
            | eframe::Error::OpenGL(_)
    )
}

/// This process again, same arguments, forced to glow. The caller spawns it
/// and relays its exit code; standard streams are inherited.
///
/// Only safe before this process has read anything from stdin: the child
/// cannot see bytes the parent already consumed. A host process therefore
/// never re-executes; it relies on [`remember_glow_needed`] (see the module
/// documentation).
pub fn glow_reexec() -> std::io::Result<Command> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args(std::env::args_os().skip(1))
        .env(RENDERER_ENV, Backend::Glow.name());
    Ok(command)
}

/// What drew the window, for diagnostics: `wgpu/dx12`, `wgpu/vulkan`,
/// `wgpu/metal`, `glow`.
pub fn describe(creation: &eframe::CreationContext<'_>) -> String {
    match &creation.wgpu_render_state {
        Some(state) => format!("wgpu/{}", state.adapter.get_info().backend.to_str()),
        None if creation.gl.is_some() => Backend::Glow.name().to_owned(),
        None => "unknown".to_owned(),
    }
}

fn forced() -> Option<Backend> {
    parse(std::env::var(RENDERER_ENV).ok().as_deref())
}

/// The backend named by the variable's value; anything else is ignored with
/// a warning, so a typo cannot keep the window from opening.
fn parse(value: Option<&str>) -> Option<Backend> {
    let value = value?.trim();
    [Backend::Wgpu, Backend::Glow]
        .into_iter()
        .find(|backend| value.eq_ignore_ascii_case(backend.name()))
        .or_else(|| {
            log::warn!("ignoring {RENDERER_ENV}={value:?}: expected wgpu or glow");
            None
        })
}

fn fallback_allowed(glow_works_here: bool, forced: Option<Backend>) -> bool {
    glow_works_here && forced != Some(Backend::Glow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_forced_backend() {
        assert_eq!(parse(Some("glow")), Some(Backend::Glow));
        assert_eq!(parse(Some(" WGPU ")), Some(Backend::Wgpu));
        assert_eq!(parse(Some("vulkan")), None);
        assert_eq!(parse(None), None);
    }

    #[test]
    fn falls_back_only_where_glow_works_and_only_once() {
        assert!(fallback_allowed(true, None));
        assert!(fallback_allowed(true, Some(Backend::Wgpu)));
        assert!(!fallback_allowed(true, Some(Backend::Glow)));
        assert!(!fallback_allowed(false, None));
    }

    #[test]
    fn maps_to_eframe_renderers() {
        assert_eq!(Backend::Wgpu.eframe(), eframe::Renderer::Wgpu);
        assert_eq!(Backend::Glow.eframe(), eframe::Renderer::Glow);
    }

    #[test]
    fn reexec_forces_glow() {
        let command = glow_reexec().expect("current exe");
        let forced = command
            .get_envs()
            .find(|(key, _)| *key == RENDERER_ENV)
            .and_then(|(_, value)| value);
        assert_eq!(forced, Some(std::ffi::OsStr::new("glow")));
    }
}
