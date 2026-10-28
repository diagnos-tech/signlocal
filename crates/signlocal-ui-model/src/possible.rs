//! "Possible certificates" as the window shows them (`docs/ux.md` §6.2).

/// A device that looks like it should hold a certificate but brought none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PossibleCard {
    /// "SafeNet eToken 5110"; `None` for an unknown card.
    pub name: Option<String>,
    /// For an unknown card: the reader it sits in.
    pub reader: Option<String>,
    pub card: bool,
    /// The vendor middleware to install, when known.
    pub driver: Option<String>,
    /// Download link for this OS, when known.
    pub download: Option<String>,
    /// macOS store build only: the Add-on is needed too (`docs/ux.md` §7).
    pub needs_complement: bool,
}
