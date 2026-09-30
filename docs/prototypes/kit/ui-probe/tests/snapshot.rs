//! Headless rendering of the sample UI through egui_kittest with wgpu.
//!
//! Adapter per OS (documented expectation, verified by the `ui-spike`
//! workflow): Windows runners get "Microsoft Basic Render Driver" (WARP, DX12)
//! or a Vulkan software driver; macOS arm64 VMs usually expose Metal through
//! "Apple Paravirtual device"; Linux gets lavapipe (llvmpipe, Vulkan CPU) once
//! mesa-vulkan-drivers is installed. The test prints the adapter it got and
//! writes `<tmp>/ui-probe-kittest.png` (override with UI_PROBE_KITTEST_OUT).
//! Without any adapter the test fails: that is the signal, not a bug.

use std::path::PathBuf;

use egui_kittest::Harness;
use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};
use ui_probe::ui::{self, UiState};

#[test]
fn renders_sample_ui_with_wgpu() {
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let info = render_state.adapter.get_info();
    println!(
        "KITTEST adapter=\"{}\" backend={:?} type={:?}",
        info.name, info.backend, info.device_type
    );
    let mut state = UiState::default();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(
            ui_probe::WINDOW_SIZE[0],
            ui_probe::WINDOW_SIZE[1],
        ))
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui| {
            ui::install_fonts(ui.ctx());
            ui::draw(ui, &mut state, "kittest / wgpu");
        });
    harness.run();
    let image = harness
        .render()
        .expect("wgpu render failed: no usable adapter");
    let out = std::env::var_os("UI_PROBE_KITTEST_OUT").map_or_else(
        || std::env::temp_dir().join("ui-probe-kittest.png"),
        PathBuf::from,
    );
    std::fs::create_dir_all(out.parent().expect("has parent")).expect("mkdir");
    image.save(&out).expect("save png");
    println!(
        "KITTEST png={} size={}x{}",
        out.display(),
        image.width(),
        image.height()
    );
}
