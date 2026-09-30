//! Direct or store build, decided at run time (`docs/plan.md` D10).

use websign_protocol::types::Channel;

/// `Store` when running with an MSIX package identity or inside the Mac App
/// Store sandbox; `Direct` otherwise.
pub fn current() -> Channel {
    todo!("packaging-and-release.md §Channels")
}
