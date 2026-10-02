//! `websign sign` and `websign choose` run one whole connection inside this
//! process: an in-memory client speaks the protocol to the same engine,
//! window and key store worker that serve `websign connect`, so the command
//! line gets exactly the web's behavior (queue, consent, verification code,
//! self-check) with no second implementation to drift.

mod frames;

use std::sync::{Arc, Mutex};

use websign_core::present::caller::DesktopCaller;
use websign_host::session::Transport;
use websign_protocol::messages::{ClientMessage, Hello};
use websign_protocol::types::ClientInfo;
use websign_protocol::{AppMessage, ErrorCode, ProtocolRange};

use super::output::error;
use frames::{FrameReader, LocalClient, envelope};

/// The id of the one request; `hello` uses [`HELLO_ID`].
const REQUEST_ID: &str = "1";
const HELLO_ID: &str = "0";

/// Sends `request` for the program that started us and returns the final
/// message. `digest` answers every `sign.need_digest`: the command line
/// knows its digest up front.
pub fn request(request: ClientMessage, digest: Option<Vec<u8>>) -> AppMessage {
    let Some(caller) = crate::platform::caller::parent_caller() else {
        return error(
            ErrorCode::Internal,
            "the calling program could not be identified",
        );
    };
    run(caller, request, digest)
}

/// [`request`] for a known caller.
fn run(caller: DesktopCaller, request: ClientMessage, digest: Option<Vec<u8>>) -> AppMessage {
    let hello = ClientMessage::Hello(Hello {
        client: ClientInfo {
            name: "websign-cli".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        protocols: ProtocolRange::CURRENT,
        browser: None,
    });
    let (to_app, reader) = FrameReader::channel();
    for (id, message) in [(HELLO_ID, hello), (REQUEST_ID, request)] {
        match envelope(id, message) {
            Some(frame) => {
                let _ = to_app.send(frame);
            }
            None => return error(ErrorCode::Internal, "the request could not be encoded"),
        }
    }
    let answer = Arc::new(Mutex::new(None));
    let client = LocalClient::new(to_app, digest, Arc::clone(&answer), REQUEST_ID);
    let status =
        crate::host_process::serve(Transport::Desktop { caller }, reader, Box::new(client));
    let answer = answer.lock().ok().and_then(|mut slot| slot.take());
    answer.unwrap_or_else(|| {
        log::error!("the engine ended with status {status} and no answer");
        error(ErrorCode::Internal, "the app ended without an answer")
    })
}
