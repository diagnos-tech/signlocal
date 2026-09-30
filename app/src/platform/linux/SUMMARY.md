# app/src/platform/linux

- `caller.rs` — The parent process from `/proc/<ppid>/exe`, named by a matching `.desktop` entry.
- `channel.rs` — Install format (deb, rpm, Flatpak, archive) from where the binary lives.
- `command.rs` — Running desktop helpers (`gsettings`, `gdbus`, `xdg-open`) with a deadline, or detached.
- `desktop_entry.rs` — Finding the `.desktop` application entry whose `Exec` starts a given executable.
- `desktop_entry_tests.rs` — Tests of the `.desktop` parsing, `Exec` unquoting and path resolution.
- `focus.rs` — No-op: the window manager or compositor decides focus.
- `mod.rs` — Linux implementations of the [`crate::platform`] functions.
- `private_file.rs` — Short-lived `0600` files in a private `$XDG_RUNTIME_DIR` folder, swept when old.
- `settings.rs` — Reduce motion (GNOME, KDE) and dark mode (XDG portal, GNOME) queries.
- `system_ui.rs` — Certificate viewer (`gcr-viewer`, else `xdg-open`) and URLs; no `.pfx` import.
