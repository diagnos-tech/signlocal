# Proof 5: screenshots of the real UI on every OS

## Question

Can the product's e2e tests render the real egui/eframe window and capture a PNG on Windows, macOS and the main Linux distributions, on CI machines with no GPU? Which approach should be used per OS, and does the wgpu-to-glow renderer fallback work?

## Method

Spike workflow `ui-spike` ([`.github/workflows/ui-spike.yml`](../../.github/workflows/ui-spike.yml)), run 36657465602 (branch `claude/focused-hopper-2o8lka`, commit 557298a). Each job builds `kit/ui-probe` in release mode and runs [`ci/run-spike.sh`](kit/ui-probe/ci/run-spike.sh), which tries four approaches and records each without failing the job:

- `window --renderer auto|wgpu|glow`: a real window, screenshot taken in-app with `ViewportCommand::Screenshot`, PNG written from `Event::Screenshot`. `auto` tries wgpu and retries in the same process with glow.
- `kittest (wgpu)`: headless `egui_kittest` render (no window, no display server), PNG from the wgpu adapter.

Targets: `windows-latest`, `macos-latest`, `ubuntu-24.04` (Xvfb + mesa), and the containers `debian:12`, `fedora:42`, `rockylinux:9`, `archlinux:latest` on an Ubuntu runner (Xvfb + mesa). Logs of all seven jobs were read, and the artifacts `ui-spike-<os>` were downloaded to check PNG sizes (all PNGs are 480x360; the ubuntu-24.04 artifact download failed with an expired-signature error, so it relies on the log; its artifact is 131 KB, like macOS, which is consistent with four PNGs).

## Results

| OS | window auto | window wgpu | window glow | kittest headless |
|---|---|---|---|---|
| Windows (Server, `windows-latest`) | OK, wgpu DX12 Microsoft Basic Render Driver (Cpu) | OK, same adapter | FAIL: `egui_glow requires opengl 2.0+` | OK, DX12 Microsoft Basic Render Driver |
| macOS (`macos-latest`) | OK, wgpu Metal Apple Paravirtual device | OK, same adapter | OK, glow Apple Software Renderer | OK, Metal Apple Paravirtual device |
| Ubuntu 24.04 | OK, wgpu Vulkan llvmpipe (LLVM 20.1.2) | OK, same adapter | OK, glow llvmpipe (LLVM 20.1.2) | OK |
| Debian 12 (container) | FAIL: xlib | FAIL: xlib | FAIL: xlib | OK, Vulkan llvmpipe (LLVM 15.0.6) |
| Fedora 42 (container) | FAIL: xlib | FAIL: xlib | FAIL: xlib | OK, Vulkan llvmpipe (LLVM 20.1.8) |
| Rocky Linux 9 (container) | FAIL: xlib | FAIL: xlib | FAIL: xlib | OK, Vulkan llvmpipe (LLVM 21.1.8) |
| Arch Linux (container) | FAIL: xlib | FAIL: xlib | FAIL: xlib | OK, Vulkan llvmpipe (LLVM 22.1.8) |

"xlib" = `winit EventLoopError: os error ... Failed to load one of xlib's shared libraries`. It happens before any renderer is chosen, so it is not a graphics failure: the containers lack the X11 client libraries that winit loads with `dlopen` (`libX11`, `libXcursor`, `libXi`, `libXrandr`). The Ubuntu runner image has them preinstalled, which is why the same script works there. Our package lists installed `libxkbcommon-x11`, `libx11-xcb1` and `libxcb-xfixes0` but not these. This is a diagnosis from the error text; it was not re-run with the extra libraries.

Notable points:

- Kittest passed on all 7 systems, always with a software adapter (llvmpipe, WARP, Apple paravirtual). It needs no display server.
- In every job where the window opened, `auto` picked wgpu, so the wgpu-to-glow fallback was never exercised on CI. It stays untested.
- glow alone fails on Windows (the runner only offers the GDI OpenGL 1.1 driver) and works on macOS and Linux. So on Windows glow cannot be the fallback; wgpu with the DX12 WARP adapter is what works.
- `auto` and `wgpu` produce byte-identical PNGs where both work (macOS 45847 bytes, Windows 45348 bytes); the kittest PNG is 37993 bytes on Windows and macOS and 37950 on Linux (font/rasterizer differences), so pixel comparisons must be tolerant and per platform.

## Decision

For the product e2e screenshots:

1. **Default for all OS: `egui_kittest` headless (wgpu).** It passed on all 7 systems, including bare containers, with no display server and no X libraries. Use it for the screenshot/snapshot suite and keep baselines per platform (Linux, macOS, Windows) with a tolerance.
2. **Window path (in-app `ViewportCommand::Screenshot`) for the few tests that must prove the real window** (native decorations, IME, file dialogs, viewport behavior):
   - Windows and macOS: use `--renderer auto` (resolves to wgpu: DX12 WARP and Metal). No extra setup on the GitHub runners.
   - Linux: run under Xvfb with mesa software drivers; `auto` resolves to wgpu Vulkan llvmpipe. On Ubuntu 24.04 this worked. On other distros the X11 client libraries must be installed too (see below).
3. **Fallback wgpu to glow: do not rely on it off Linux/macOS.** It is unproven on CI (never triggered), and on Windows glow does not work at all on CI-class machines. A failed window start also cannot be retried by creating a second winit event loop, so if the fallback is kept it should be a re-exec of the process with `--renderer glow`, only on macOS/Linux. On Windows the only software path is WARP through wgpu, which is already what `auto` picks.

### Packages that worked

Kittest (window path not required), from `ui-spike.yml`:

| Distro | Packages |
|---|---|
| Ubuntu 24.04 (window + kittest) | `xvfb xauth mesa-vulkan-drivers libgl1-mesa-dri libegl1 libxkbcommon-x11-0 libx11-xcb1 libxcb-xfixes0` |
| Debian 12 | `xvfb xauth mesa-vulkan-drivers libgl1-mesa-dri libegl1 libxkbcommon-x11-0 libx11-xcb1 libxcb-xfixes0 libwayland-client0 libxkbcommon0` (plus build tools) |
| Fedora 42 | `xorg-x11-server-Xvfb xorg-x11-xauth mesa-vulkan-drivers mesa-dri-drivers mesa-libEGL libxkbcommon-x11 libxcb libwayland-client` |
| Rocky Linux 9 | `epel-release` first, then the Fedora list (with `--allowerasing`) |
| Arch Linux | `xorg-server-xvfb xorg-xauth vulkan-swrast mesa libxkbcommon-x11 libxcb wayland` |

For the window path on the containers, add the X11 client libraries (untested hypothesis, next step: rerun the spike): Debian `libx11-6 libxcursor1 libxi6 libxrandr2`; Fedora/Rocky `libX11 libXcursor libXi libXrandr`; Arch `libx11 libxcursor libxi libxrandr`.

## Still missing

- Rerun the container jobs with the X11 libraries to confirm the window path there.
- Exercise the fallback for real (force a wgpu failure, e.g. `WGPU_BACKEND` pointing at an unavailable backend, or no Vulkan ICD) and test a re-exec.
- Wayland sessions and real GPUs/RDP sessions (only software adapters were tested).
