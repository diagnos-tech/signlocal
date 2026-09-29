//! Opening, listing and de-duplicating the key sources, once per process.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};

use probe_core::{CertInfo, Deduped, Fingerprint, dedup_by_fingerprint};

use crate::keystores::{self, FoundKey, Keystore, Opened, Options};
use crate::nm::protocol::{ErrorCode, ProtocolError};

/// One certificate and the ways to reach its key.
pub(super) struct Group {
    pub(super) info: CertInfo,
    /// Index into [`Loaded::keys`] of the preferred path (the OS first).
    pub(super) primary: usize,
    pub(super) paths: usize,
}

pub(super) struct Key {
    /// Index into [`Loaded::keystores`].
    pub(super) store: usize,
    pub(super) found: FoundKey,
}

pub(super) struct Loaded {
    pub(super) keystores: Vec<Box<dyn Keystore>>,
    pub(super) keys: Vec<Key>,
    pub(super) groups: Vec<Group>,
    pub(super) warnings: Vec<String>,
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
        *state = match catch_unwind(AssertUnwindSafe(|| load(options))) {
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

/// Opens every source, lists it and merges duplicates (the OS wins).
fn load(options: &Options) -> Loaded {
    Loaded::index(keystores::open_all(options))
}

impl Loaded {
    /// Lists every opened source and groups the keys by certificate.
    pub(super) fn index(opened: Opened) -> Loaded {
        let mut warnings: Vec<String> = opened
            .failures
            .iter()
            .map(|failure| format!("{}: {}", failure.source, failure.error))
            .collect();
        let mut keystores = opened.keystores;
        let mut keys = Vec::new();
        for (store, keystore) in keystores.iter_mut().enumerate() {
            match keystore.list() {
                Ok(found) => keys.extend(found.into_iter().map(|found| Key { store, found })),
                Err(error) => warnings.push(format!("{}: {error}", keystore.name())),
            }
        }

        let parsed: Vec<(usize, CertInfo)> = keys
            .iter()
            .enumerate()
            .filter_map(|(index, key)| Some((index, CertInfo::from_der(&key.found.cert_der).ok()?)))
            .collect();
        let groups = dedup_by_fingerprint(parsed, |(index, _)| {
            let found = &keys[*index].found;
            (Fingerprint::of(&found.cert_der), found.kind)
        })
        .into_iter()
        .map(
            |Deduped {
                 primary,
                 alternates,
             }| Group {
                paths: 1 + alternates.len(),
                primary: primary.0,
                info: primary.1,
            },
        )
        .collect();
        Loaded {
            keystores,
            keys,
            groups,
            warnings,
        }
    }
}

pub(super) fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_owned())
}
