//! The app's log: a size-capped file in the temp folder, because browsers
//! discard a native host's stderr. Level from `WEBSIGN_LOG`
//! (`error`…`trace`, default `info`).
//!
//! Privacy rule for every `log::` call in the workspace: steps, API names,
//! error codes, counts, sizes and module file names only — never a name,
//! document number, digest, signature, certificate, PIN, token label, serial
//! number or site (`docs/architecture/security.md` §Logs).

/// Installs the file logger. Never fails: without a writable temp folder the
/// app runs unlogged.
pub fn init() {
    // Implemented by the app track (docs/plan.md, track A1); a no-op until then.
}
