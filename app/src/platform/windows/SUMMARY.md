# app/src/platform/windows

- `authenticode.rs` — An executable's signer: embedded signature, else its signed catalog, from one open handle.
- `catalog.rs` — Catalog lookup (`CryptCATAdmin*`) for Windows' own programs, named "Microsoft Windows".
- `caller.rs` — The parent process (Toolhelp, checked against PID reuse) and its image path.
- `channel.rs` — MSIX or not, from the process's package identity.
- `focus.rs` — `SetForegroundWindow`, else topmost and a taskbar flash.
- `mod.rs` — Windows implementations of the [`crate::platform`] functions.
- `settings.rs` — Reduce motion (`SPI_GETCLIENTAREAANIMATION`) and dark mode (`AppsUseLightTheme`).
- `system_ui.rs` — `CryptUIDlgViewContext`, the `CryptUIWizImport` wizard, and `ShellExecuteW` for URLs.
- `version_info.rs` — `FileDescription`/`ProductName` from an executable's version resource.
- `trust.rs` — `WinVerifyTrust` of a file or catalog entry and the verified signer's subject name.
- `wide.rs` — UTF-16 conversions and owner-window handles for Win32 calls.
