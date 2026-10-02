# crates/websign-registration/src/destination

- `file.rs` — manifest files: written only where the browser's folder exists, removed with any host copy
- `registry.rs` — Windows registry entries that point browsers at a host manifest
- `registry_tests.rs` — the Windows plan applied end to end against an in-memory registry
- `tests.rs` — install/uninstall tests over temporary folders (promoted from the kit)
