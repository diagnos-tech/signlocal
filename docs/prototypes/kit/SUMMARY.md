# kit

The risk-proof kit: a Cargo workspace and helper scripts whose surviving parts (probe-core, keystore adapters) seed the real app, so quality rules apply here too.

- `.cargo/` — Cargo configuration shared by the workspace
- `Cargo.toml` — workspace manifest: members, shared package metadata, dependency versions, and lint levels
- `Cargo.lock` — pinned dependency versions of the workspace
- `extension/` — minimal Manifest V3 test extension that bridges localhost pages to the native host
- `linux/` — Linux proof scripts: SoftHSM2 token setup and the full CI run
- `macos/` — macOS proof scripts: Keychain CI run and the App Sandbox experiment
- `msix/` — Windows MSIX packaging and test scripts for the probe package
- `nm-e2e/` — Node harness that checks the whole browser path in a real Chromium
- `probe/` — the `websign-probe` binary: lists and signs through every key path, and runs as a native messaging host
- `probe-core/` — pure, platform-independent, heavily tested logic shared by every key source
- `ui-probe/` — spike that proves the app window can be rendered and screenshotted on every CI target
- `windows/` — Windows proof scripts: test certificate creation and the CI run
