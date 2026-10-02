//! The machine's key sources, opened and listed once per process.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};

use probe_core::{CertInfo, Fingerprint};

use crate::keystores::inventory::{Entry, Inventory};
use crate::keystores::{Opened, Options};
use crate::nm::protocol::{ErrorCode, ProtocolError};

/// One certificate the host offers.
pub(super) struct Group {
    /// Index into [`Inventory::entries`] of the preferred path (the OS first).
    entry: usize,
    /// How many key sources expose the certificate.
    paths: usize,
}

/// Every source, what it listed, and the certificates that can be offered
/// (the ones whose DER could be read).
pub(super) struct Loaded {
    pub(super) inventory: Inventory,
    groups: Vec<Group>,
}

pub(super) enum State {
    Unloaded,
    Loaded(Loaded),
    Failed(ProtocolError),
}

/// Opens and lists every key source the first time it is needed.
pub(super) fn ensure_loaded<'a>(
    state: &'a mut State,
    options: &Options,
) -> Result<&'a mut Loaded, ProtocolError> {
    if matches!(state, State::Unloaded) {
        let opened = catch_unwind(AssertUnwindSafe(|| {
            Loaded::index(crate::keystores::open_all(options))
        }));
        *state = match opened {
            Ok(loaded) => State::Loaded(loaded),
            Err(payload) => State::Failed(ProtocolError::new(
                ErrorCode::Internal,
                format!("key sources failed to load: {}", panic_message(&*payload)),
            )),
        };
    }
    match state {
        State::Loaded(loaded) => Ok(loaded),
        State::Failed(error) => Err(error.clone()),
        State::Unloaded => unreachable!("loaded above"),
    }
}

impl Loaded {
    /// Lists every opened source and groups the keys by certificate.
    pub(super) fn index(opened: Opened) -> Loaded {
        let inventory = Inventory::list(opened);
        let groups = inventory
            .groups()
            .into_iter()
            .filter(|group| inventory.entries[group.primary].info.is_ok())
            .map(|group| Group {
                entry: group.primary,
                paths: 1 + group.alternates.len(),
            })
            .collect();
        Loaded { inventory, groups }
    }

    /// Each offered certificate, its preferred entry and its number of paths.
    pub(super) fn certificates(&self) -> impl Iterator<Item = (&CertInfo, &Entry, usize)> {
        self.groups.iter().filter_map(|group| {
            let entry = &self.inventory.entries[group.entry];
            Some((entry.info.as_ref().ok()?, entry, group.paths))
        })
    }

    /// Index into [`Inventory::entries`] of the preferred path to `fingerprint`.
    pub(super) fn find(&self, fingerprint: &Fingerprint) -> Option<usize> {
        self.groups.iter().map(|group| group.entry).find(|&entry| {
            self.inventory.entries[entry]
                .info
                .as_ref()
                .is_ok_and(|info| info.fingerprint == *fingerprint)
        })
    }
}

pub(super) fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_owned())
}
