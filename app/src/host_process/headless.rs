//! A windowless confirmation for e2e builds only (feature `e2e`, never in
//! releases; `tests/release_build.rs` checks the default binary has none of
//! this). With `WEBSIGN_E2E_HEADLESS=1` the engine gets this port instead of
//! the window: it decides as a person would with the preselected
//! certificate, so client libraries can be tested end to end on machines
//! without a display (CI containers, SoftHSM2).
//!
//! `WEBSIGN_E2E_CONFIRM=cancel` cancels instead; `WEBSIGN_E2E_PIN` is the
//! PIN typed for PKCS#11 keys (`crate::e2e` documents both).

use std::collections::BTreeMap;

use secrecy::SecretString;
use websign_host::EngineEvent;
use websign_host::ports::ConfirmUi;
use websign_host::runtime::EventSender;
use websign_protocol::ErrorCode;
use websign_ui_model::certs::build_cert_list;
use websign_ui_model::confirm::port::{Failure, Mode, RequestKey};
use websign_ui_model::confirm::{UiCommand, UiEvent};

/// Turns the headless port on.
const HEADLESS_ENV: &str = "WEBSIGN_E2E_HEADLESS";

/// The scripted port, when the environment asks for it.
pub fn from_env(events: EventSender) -> Option<Headless> {
    std::env::var_os(HEADLESS_ENV).filter(|v| v == "1")?;
    log::warn!("{}: headless confirmation active", crate::e2e::MARKER);
    Some(Headless {
        events,
        cancel: std::env::var("WEBSIGN_E2E_CONFIRM").is_ok_and(|v| v == "cancel"),
        pin: std::env::var("WEBSIGN_E2E_PIN").ok(),
        modes: BTreeMap::new(),
    })
}

/// Answers every window command at once, like a person who always agrees.
#[derive(Debug)]
pub struct Headless {
    events: EventSender,
    cancel: bool,
    pin: Option<String>,
    /// Requests shown and not yet answered with a certificate.
    modes: BTreeMap<RequestKey, Mode>,
}

impl ConfirmUi for Headless {
    fn command(&mut self, command: UiCommand) {
        let event = match command {
            UiCommand::Open(request) => {
                self.modes.insert(request.key, request.mode);
                None
            }
            UiCommand::Certificates {
                key,
                candidates,
                context,
                ..
            } => {
                let selected = build_cert_list(&candidates, &context).selected;
                self.modes
                    .remove(&key)
                    .map(|mode| match (self.cancel, selected, mode) {
                        (true, _, _) => cancel(key, ErrorCode::UserCancelled),
                        (false, None, _) => cancel(key, ErrorCode::NoCertificates),
                        (false, Some(fingerprint), Mode::Choose) => UiEvent::Choose {
                            key,
                            fingerprint,
                            remember: false,
                        },
                        (false, Some(fingerprint), Mode::Sign { .. }) => {
                            UiEvent::Continue { key, fingerprint }
                        }
                    })
            }
            UiCommand::DigestReady {
                key, fingerprint, ..
            } => Some(UiEvent::Sign {
                key,
                fingerprint,
                via: 0,
                pin: self.pin.clone().map(SecretString::from),
                remember: false,
            }),
            UiCommand::Failed { key, failure } => Some(cancel(key, failure_code(&failure))),
            _ => None,
        };
        if let Some(event) = event {
            let _ = self.events.send(EngineEvent::Ui(event));
        }
    }

    fn parent_window(&self) -> Option<isize> {
        None
    }
}

fn cancel(key: RequestKey, code: ErrorCode) -> UiEvent {
    UiEvent::Cancel { key, code }
}

/// A failure a person would read and give up on, as the caller's code.
fn failure_code(failure: &Failure) -> ErrorCode {
    match failure {
        Failure::PinIncorrect { .. } => ErrorCode::PinIncorrect,
        Failure::PinLocked { .. } => ErrorCode::PinLocked,
        Failure::TokenRemoved => ErrorCode::TokenRemoved,
        Failure::DriverFailure { .. } => ErrorCode::DriverFailure,
        Failure::UnsupportedAlgorithm { .. } => ErrorCode::UnsupportedAlgorithm,
        Failure::CertificateUnavailable => ErrorCode::CertificateUnavailable,
        Failure::Internal { .. } => ErrorCode::Internal,
    }
}
