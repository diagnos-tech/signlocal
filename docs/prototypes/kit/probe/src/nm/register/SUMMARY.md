# probe/src/nm/register

`websign-probe register`: writes the native messaging manifest where each browser looks for it.

- `browsers.rs` — the browsers `--browser` accepts
- `destination.rs` — one place a browser looks for the host, and how to write or remove it
- `destination/` — registry writing and destination tests
- `home.rs` — the user's real home directory (not the sandbox container)
- `linux.rs` — Linux manifest locations
- `linux/` — tests for the Linux locations
- `macos.rs` — macOS manifest locations
- `manifest.rs` — the host manifest JSON
- `mod.rs` — the `register` command
- `windows.rs` — Windows `HKCU` keys pointing at a manifest file
