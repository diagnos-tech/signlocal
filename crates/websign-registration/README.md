# websign-registration

Makes browsers find the app: native messaging manifests for Chrome, Edge,
Brave, Chromium, Vivaldi, Opera (and Arc on macOS) and Firefox on every OS
(per user, and system-wide for Linux packages), the `websign:` URL scheme,
extension pre-registration, installed-browser detection and read-back status
for diagnostics.

- Manifest writing per OS is promoted from the Phase-0 kit (proven in CI,
  inside the macOS sandbox and from an MSIX package).
- Writes only into folders a browser already created; never reads profiles.
- Every Windows registry access goes through the `registry::Registry` trait,
  so the Windows logic is tested on Linux and macOS against
  `registry::MemoryRegistry`; writes can only target `HKCU`.
- Contract: [`SPEC.md`](SPEC.md).
- License: GPL-3.0-or-later.

## Build and test

```sh
cargo test -p websign-registration
cargo clippy -p websign-registration --all-targets -- -D warnings
cargo clippy -p websign-registration --all-targets --target x86_64-pc-windows-msvc -- -D warnings
cargo clippy -p websign-registration --all-targets --target aarch64-apple-darwin -- -D warnings
```

Tests write only into temporary folders (a throwaway home, data folder or
root) and an in-memory registry; they never touch the real user's browsers.
