# crates/websign-registration/src/status

- `evaluate/` — tests of the manifest judgement
- `files/` — status tests over a throwaway home
- `registry/` — Windows status tests over an in-memory registry
- `evaluate.rs` — judges one manifest against what this app would have written
- `files.rs` — status from manifest files (Linux and macOS)
- `registry.rs` — status on Windows: the first `HKCU` key the browser finds decides
