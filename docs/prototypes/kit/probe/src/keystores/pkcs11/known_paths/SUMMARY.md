# probe/src/keystores/pkcs11/known_paths

Where token vendors install their PKCS#11 modules, per operating system; each line has its source in `docs/research/pkcs11-modules.md`.

- `linux.rs` — Linux install locations
- `macos.rs` — macOS install locations
- `mod.rs` — selects the list for the current OS and validates the path templates
- `windows.rs` — Windows install locations (64-bit modules only)
