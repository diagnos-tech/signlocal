# app/src/ui/confirm/snapshots/linux

Reviewed baselines, `<state>-<theme>.png`, 480 × 600, rendered headless (wgpu, Vulkan llvmpipe). The scenes are
scripted in `../../tests/scenes.rs`; Ana's card goes through the token driver here (`fixtures::ana_card`), so no state shows an OS PIN hint.

- `*.png` — one image per state and theme: loading, loading-slow, empty, continue-new-site, preparing, arming, ready, pin-field, pin-error, pin-locked, signing, error-driver, error-unsupported, success, site-cancelled, timeout, choose, desktop-unverified, queue-idn-expiring
