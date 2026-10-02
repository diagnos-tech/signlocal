# crates/websign-registration/src/detect

- `linux/` — tests of Linux detection over a throwaway root
- `windows/` — tests of Windows detection over an in-memory registry
- `file_version.rs` — the version resource of a Windows executable (Windows only)
- `launch_services.rs` — LaunchServices lookup by bundle ID and the bundle's version (macOS only)
- `linux.rs` — Linux: executables on `PATH` and `/opt`, Snap launchers, Flatpak IDs
- `macos.rs` — macOS: the bundle IDs of each browser's channels
- `platform.rs` — the OS calls detection needs, with inert stand-ins elsewhere
- `windows.rs` — Windows: `StartMenuInternet` clients and `App Paths` in `HKCU` and `HKLM`
