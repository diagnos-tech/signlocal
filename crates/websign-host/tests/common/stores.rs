//! In-memory stores with the semantics of `SPEC.md` §7, shareable with the
//! test so it can seed and inspect them.

use std::cell::RefCell;
use std::rc::Rc;

use websign_host::store::{
    ConnectionRecord, ConnectionStore, ConsentRecord, ConsentStore, ErrorRecord, ErrorStore,
    Settings, SettingsStore, StoreError, Stores, UsageStore,
};

#[derive(Default)]
pub struct StoreState {
    pub consent: Vec<ConsentRecord>,
    pub connections: Vec<ConnectionRecord>,
    pub usage: Vec<String>,
    pub errors: Vec<ErrorRecord>,
    pub settings: Settings,
    /// Every operation fails with an I/O error.
    pub failing: bool,
}

pub type State = Rc<RefCell<StoreState>>;

fn check(state: &State) -> Result<(), StoreError> {
    if state.borrow().failing {
        Err(StoreError::Io {
            path: "fake".to_owned(),
            detail: "disk on fire".to_owned(),
        })
    } else {
        Ok(())
    }
}

pub struct MemStores {
    consent: Consent,
    connections: Connections,
    usage: Usage,
    errors: Errors,
    settings: SettingsMem,
}

impl MemStores {
    pub fn new(state: &State) -> MemStores {
        MemStores {
            consent: Consent(state.clone()),
            connections: Connections(state.clone()),
            usage: Usage(state.clone()),
            errors: Errors(state.clone()),
            settings: SettingsMem(state.clone()),
        }
    }
}

impl Stores for MemStores {
    fn consent(&mut self) -> &mut dyn ConsentStore {
        &mut self.consent
    }
    fn connections(&mut self) -> &mut dyn ConnectionStore {
        &mut self.connections
    }
    fn usage(&mut self) -> &mut dyn UsageStore {
        &mut self.usage
    }
    fn errors(&mut self) -> &mut dyn ErrorStore {
        &mut self.errors
    }
    fn settings(&mut self) -> &mut dyn SettingsStore {
        &mut self.settings
    }
}

struct Consent(State);

impl ConsentStore for Consent {
    fn get(&mut self, key: &str) -> Result<Option<ConsentRecord>, StoreError> {
        check(&self.0)?;
        Ok(self
            .0
            .borrow()
            .consent
            .iter()
            .find(|r| r.key == key)
            .cloned())
    }

    fn list(&mut self) -> Result<Vec<ConsentRecord>, StoreError> {
        check(&self.0)?;
        Ok(self.0.borrow().consent.clone())
    }

    fn remember(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError> {
        check(&self.0)?;
        let mut state = self.0.borrow_mut();
        match state.consent.iter_mut().find(|r| r.key == key) {
            Some(record) => {
                record.certificates.retain(|f| f != fingerprint);
                record.certificates.insert(0, fingerprint.to_owned());
                record.last_used_at = now;
            }
            None => state.consent.push(ConsentRecord {
                key: key.to_owned(),
                remembered_at: now,
                last_used_at: now,
                certificates: vec![fingerprint.to_owned()],
            }),
        }
        Ok(())
    }

    fn record_use(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError> {
        check(&self.0)?;
        let mut state = self.0.borrow_mut();
        if let Some(record) = state.consent.iter_mut().find(|r| r.key == key) {
            record.certificates.retain(|f| f != fingerprint);
            record.certificates.insert(0, fingerprint.to_owned());
            record.last_used_at = now;
        }
        Ok(())
    }

    fn revoke(&mut self, key: &str) -> Result<(), StoreError> {
        check(&self.0)?;
        self.0.borrow_mut().consent.retain(|r| r.key != key);
        Ok(())
    }
}

struct Connections(State);

impl ConnectionStore for Connections {
    fn record(&mut self, record: ConnectionRecord) -> Result<(), StoreError> {
        check(&self.0)?;
        let mut state = self.0.borrow_mut();
        state.connections.retain(|c| c.browser != record.browser);
        state.connections.push(record);
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<ConnectionRecord>, StoreError> {
        check(&self.0)?;
        Ok(self.0.borrow().connections.clone())
    }
}

struct Usage(State);

impl UsageStore for Usage {
    fn recent(&mut self) -> Result<Vec<String>, StoreError> {
        check(&self.0)?;
        Ok(self.0.borrow().usage.clone())
    }

    fn record(&mut self, fingerprint: &str, _now: i64) -> Result<(), StoreError> {
        check(&self.0)?;
        let mut state = self.0.borrow_mut();
        state.usage.retain(|f| f != fingerprint);
        state.usage.insert(0, fingerprint.to_owned());
        Ok(())
    }
}

struct Errors(State);

impl ErrorStore for Errors {
    fn record(&mut self, record: ErrorRecord) -> Result<(), StoreError> {
        check(&self.0)?;
        self.0.borrow_mut().errors.push(record);
        Ok(())
    }

    fn list(&mut self) -> Result<Vec<ErrorRecord>, StoreError> {
        check(&self.0)?;
        Ok(self.0.borrow().errors.clone())
    }
}

struct SettingsMem(State);

impl SettingsStore for SettingsMem {
    fn get(&mut self) -> Result<Settings, StoreError> {
        check(&self.0)?;
        Ok(self.0.borrow().settings.clone())
    }

    fn update(&mut self, change: &mut dyn FnMut(&mut Settings)) -> Result<(), StoreError> {
        check(&self.0)?;
        change(&mut self.0.borrow_mut().settings);
        Ok(())
    }
}
