//! "Still reading {device}" (`docs/ux.md` §4.8): a listing that runs past
//! [`SLOW_LISTING`](crate::ports::SLOW_LISTING) tells the engine which
//! device it is most likely reading.
//!
//! The key stores report no progress (a PKCS#11 `C_Initialize` or object
//! search simply blocks), so the worker cannot know which module is slow.
//! It names the first plugged-in token or card that `devices.json` knows:
//! that is the device whose driver is being read in practice, and its model
//! name is safe to show.

use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::time::Duration;

use websign_devices::Snapshot;
use websign_devices::hints::DeviceDatabase;
use websign_devices::possible::{LinkedDevices, possible_devices};

use super::EventSender;
use crate::engine::EngineEvent;
use crate::ports::KeyReply;

/// The model name of the first known token or card in `scan`.
pub(super) fn reading_device(scan: &Snapshot, database: &DeviceDatabase) -> Option<String> {
    possible_devices(scan, database, &LinkedDevices::default())
        .into_iter()
        .find_map(|device| device.hint.map(|hint| hint.name))
}

/// Posts [`KeyReply::SlowListing`] once `after` has passed, unless dropped
/// first (the listing ended). A late notice that crosses the listing is
/// dropped by the engine, which knows no listing is running any more.
#[derive(Debug)]
pub(super) struct SlowWatch {
    _running: Sender<()>,
}

impl SlowWatch {
    pub fn start(after: Duration, device: Option<String>, events: EventSender) -> SlowWatch {
        let (running, ended) = channel::<()>();
        std::thread::spawn(move || {
            if ended.recv_timeout(after) == Err(RecvTimeoutError::Timeout) {
                let _ = events.send(EngineEvent::Keys(KeyReply::SlowListing { device }));
            }
        });
        SlowWatch { _running: running }
    }
}

#[cfg(test)]
mod tests {
    use websign_devices::pcsc::{ReaderScan, ServiceState};
    use websign_devices::usb::{UsbDevice, UsbScan};
    use websign_keystores::{KeystoreError, KeystoreHub, Opened};

    use super::*;
    use crate::ports::KeyCommand;
    use crate::runtime::key_worker::spawn_worker;

    fn scan(product: &str) -> Snapshot {
        Snapshot {
            usb: UsbScan {
                devices: vec![UsbDevice {
                    vendor_id: 0x0529,
                    product_id: 0x0620,
                    manufacturer: None,
                    product: Some(product.to_owned()),
                    classes: vec![0x0B],
                    smart_card: true,
                }],
                hidden: 0,
                problem: None,
            },
            readers: ReaderScan {
                readers: Vec::new(),
                problem: None,
                service: ServiceState::Running,
            },
        }
    }

    #[test]
    fn the_device_is_named_by_its_catalog_model_only() {
        let database = DeviceDatabase::embedded().unwrap();
        assert_eq!(
            reading_device(&scan("eToken 0001-ANA-SOUZA"), &database).as_deref(),
            Some("SafeNet eToken 5110")
        );
        let nothing = Snapshot {
            usb: UsbScan {
                devices: Vec::new(),
                hidden: 0,
                problem: None,
            },
            ..scan("")
        };
        assert_eq!(reading_device(&nothing, &database), None);
    }

    #[test]
    fn the_notice_comes_only_after_the_delay() {
        let (events, inbox) = channel();
        let quick = SlowWatch::start(Duration::from_secs(60), None, events.clone());
        drop(quick);
        let _slow = SlowWatch::start(Duration::from_millis(20), Some("Token".into()), events);
        let first = inbox.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(matches!(
            first,
            EngineEvent::Keys(KeyReply::SlowListing { device: Some(name) }) if name == "Token"
        ));
        assert!(inbox.recv_timeout(Duration::from_millis(100)).is_err());
    }

    /// A key source whose listing takes `delay` (a driver loading).
    struct SlowSource {
        delay: Duration,
    }

    impl websign_keystores::Keystore for SlowSource {
        fn name(&self) -> String {
            "pkcs11:libslow.so".to_owned()
        }

        fn list(&mut self) -> Result<Vec<websign_keystores::FoundKey>, KeystoreError> {
            std::thread::sleep(self.delay);
            Ok(Vec::new())
        }

        fn sign(
            &mut self,
            _: &websign_keystores::FoundKey,
            _: &websign_keystores::SignRequest<'_>,
        ) -> Result<websign_keystores::Signature, KeystoreError> {
            Err(KeystoreError::NotFound)
        }
    }

    #[test]
    fn the_worker_announces_a_slow_listing_before_its_result() {
        let (events, inbox) = channel();
        let make_hub = || {
            KeystoreHub::with_sources(Opened {
                keystores: vec![Box::new(SlowSource {
                    delay: Duration::from_millis(300),
                })],
                failures: Vec::new(),
            })
        };
        let commands = spawn_worker(make_hub, Duration::from_millis(20), events);
        commands.send(KeyCommand::List { refresh: false }).unwrap();
        let wait = Duration::from_secs(10);
        assert!(matches!(
            inbox.recv_timeout(wait).unwrap(),
            EngineEvent::Keys(KeyReply::SlowListing { .. })
        ));
        assert!(matches!(
            inbox.recv_timeout(wait).unwrap(),
            EngineEvent::Keys(KeyReply::Listed(_))
        ));

        // Cached: answered at once, without a notice.
        commands.send(KeyCommand::List { refresh: false }).unwrap();
        assert!(matches!(
            inbox.recv_timeout(wait).unwrap(),
            EngineEvent::Keys(KeyReply::Listed(_))
        ));
        assert!(inbox.recv_timeout(Duration::from_millis(100)).is_err());
    }
}
