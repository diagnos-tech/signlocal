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

/// A concrete backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Wgpu,
    Glow,
}

/// Environment variable that forces a backend (and marks a re-exec).
pub const RENDERER_ENV: &str = "WEBSIGN_RENDERER";

/// The backend for this process: the forced one, else wgpu.
pub fn choose() -> Backend {
    todo!("testing.md §Renderer; ui track")
}

/// Whether a failed wgpu start may be retried by re-executing with glow:
/// macOS and Linux, and only when glow was not already forced.
pub fn may_fall_back_to_glow() -> bool {
    todo!("testing.md §Renderer; ui track")
}
