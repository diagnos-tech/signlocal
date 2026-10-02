# crates/websign-registration/src/url_scheme

- `linux/` — tests of the desktop entry with `xdg-mime` faked
- `linux.rs` — Linux: a hidden `.desktop` entry for `x-scheme-handler/websign`, made default with `xdg-mime`
- `macos.rs` — macOS: nothing to write; the `CFBundleURLTypes` fragment for `Info.plist`
- `windows.rs` — Windows: `HKCU\Software\Classes\websign`
