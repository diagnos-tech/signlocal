//! Renderer selection with automatic fallback (wgpu, then glow).
//!
//! Virtual machines and RDP sessions often lack a usable Vulkan/Metal/DX12
//! adapter; glow (OpenGL) or a software adapter usually still works. `auto`
//! therefore tries wgpu first and retries with glow when wgpu cannot start.

use std::fmt;

use clap::ValueEnum;

/// User-facing renderer choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Choice {
    /// Try wgpu, fall back to glow when wgpu fails to initialize.
    Auto,
    /// wgpu only (Vulkan, Metal, DX12 or GL through wgpu).
    Wgpu,
    /// glow only (OpenGL).
    Glow,
}

impl Choice {
    /// Backends to attempt, in order.
    pub fn plan(self) -> &'static [Backend] {
        match self {
            Self::Auto => &[Backend::Wgpu, Backend::Glow],
            Self::Wgpu => &[Backend::Wgpu],
            Self::Glow => &[Backend::Glow],
        }
    }
}

/// A concrete backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// wgpu.
    Wgpu,
    /// glow (OpenGL).
    Glow,
}

impl Backend {
    /// The matching eframe renderer.
    pub fn eframe(self) -> eframe::Renderer {
        match self {
            Self::Wgpu => eframe::Renderer::Wgpu,
            Self::Glow => eframe::Renderer::Glow,
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Wgpu => "wgpu",
            Self::Glow => "glow",
        })
    }
}

/// What actually rendered, printed for the CI result table.
#[derive(Debug, Clone)]
pub struct Used {
    /// Backend that produced the frames.
    pub backend: Backend,
    /// Adapter (wgpu) or GL renderer string (glow).
    pub description: String,
}

/// True when `error` means the graphics stack could not start (as opposed to
/// windowing or app errors), so another renderer may still work.
pub fn is_renderer_failure(error: &eframe::Error) -> bool {
    matches!(
        error,
        eframe::Error::Wgpu(_)
            | eframe::Error::Glutin(_)
            | eframe::Error::NoGlutinConfigs(..)
            | eframe::Error::OpenGL(_)
    )
}

/// Runs `attempt` for each backend of the plan until one succeeds.
///
/// A failed attempt is retried with the next backend only when
/// `is_renderer_failure` says the error came from the renderer; other errors
/// (no display server, event loop failure) would fail again, and winit cannot
/// recreate an event loop that failed to start. Fallbacks are reported through
/// `on_fallback`. Returns the last error when all attempts fail.
pub fn run_with_fallback<T, E: fmt::Display>(
    choice: Choice,
    mut attempt: impl FnMut(Backend) -> Result<T, E>,
    is_renderer_failure: impl Fn(&E) -> bool,
    mut on_fallback: impl FnMut(Backend, &E),
) -> Result<(Backend, T), E> {
    let plan = choice.plan();
    let mut last = None;
    for (position, &backend) in plan.iter().enumerate() {
        match attempt(backend) {
            Ok(value) => return Ok((backend, value)),
            Err(error) => {
                let retry = position + 1 < plan.len() && is_renderer_failure(&error);
                if retry {
                    on_fallback(backend, &error);
                }
                last = Some(error);
                if !retry {
                    break;
                }
            }
        }
    }
    Err(last.expect("plan is never empty"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_falls_back_to_glow() {
        let mut fell = vec![];
        let result = run_with_fallback(
            Choice::Auto,
            |b| {
                if b == Backend::Wgpu {
                    Err("no adapter")
                } else {
                    Ok(7)
                }
            },
            |_| true,
            |b, _| fell.push(b),
        );
        assert_eq!(result.unwrap(), (Backend::Glow, 7));
        assert_eq!(fell, [Backend::Wgpu]);
    }

    #[test]
    fn non_renderer_errors_stop_the_plan() {
        let mut calls = 0;
        let result: Result<(Backend, ()), _> = run_with_fallback(
            Choice::Auto,
            |_| {
                calls += 1;
                Err("no display")
            },
            |_| false,
            |_, _| panic!("no fallback"),
        );
        assert!(result.is_err());
        assert_eq!(calls, 1);
    }

    #[test]
    fn explicit_choice_does_not_fall_back() {
        let result: Result<(Backend, ()), _> = run_with_fallback(
            Choice::Wgpu,
            |_| Err("boom"),
            |_| true,
            |_, _| panic!("no fallback"),
        );
        assert_eq!(result.unwrap_err(), "boom");
    }
}
