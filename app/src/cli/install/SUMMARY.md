# app/src/cli/install

- `menu_entry.rs` — The Linux app-menu entry, written only when none exists and removed only when ours.
- `purge.rs` — `uninstall --purge`: the data folder and the log folder, only when their paths end in the app's name.
- `steps.rs` — Manifests, URL scheme and extension pre-registration as report steps.
