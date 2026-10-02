//! Turning the model's decisions into events for the engine.
//!
//! Only `Sign` carries something the model never sees: the PIN. It is
//! copied once into a `SecretString` (wiped when the engine drops it after
//! `C_Login`) and the field's buffer is wiped at once.

use secrecy::SecretString;
use websign_host::EngineEvent;
use websign_host::runtime::EventSender;
use websign_ui_model::confirm::port::RequestKey;
use websign_ui_model::confirm::{Intent, UiEvent};

use super::session::Session;

/// Sends `intent` for request `key`. `pin_field` says whether our PIN field
/// is on screen (only then does the PIN go with `Sign`).
pub fn send(
    events: &EventSender,
    key: RequestKey,
    intent: Intent,
    session: &mut Session,
    pin_field: bool,
) {
    let event = match intent {
        Intent::Selected(fingerprint) => {
            session.clear_pin();
            session.details_open = false;
            session.technical_open = false;
            UiEvent::Selected { key, fingerprint }
        }
        Intent::Continue(fingerprint) => UiEvent::Continue { key, fingerprint },
        Intent::Sign {
            fingerprint,
            via,
            remember,
        } => {
            let pin = pin_field.then(|| SecretString::from(session.pin.as_str().to_owned()));
            session.clear_pin();
            UiEvent::Sign {
                key,
                fingerprint,
                via,
                pin,
                remember,
            }
        }
        Intent::Choose {
            fingerprint,
            remember,
        } => UiEvent::Choose {
            key,
            fingerprint,
            remember,
        },
        Intent::Cancel(code) => UiEvent::Cancel { key, code },
        Intent::Rescan => UiEvent::Rescan { key },
        Intent::OpenDiagnostics(tab) => UiEvent::OpenDiagnostics { tab },
        Intent::ViewCertificate(fingerprint) => UiEvent::ViewCertificate { key, fingerprint },
    };
    if events.send(EngineEvent::Ui(event)).is_err() {
        log::warn!("the engine is gone; a window decision was dropped");
    }
}
