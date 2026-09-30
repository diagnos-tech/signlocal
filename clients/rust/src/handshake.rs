//! Starting the app and agreeing on a protocol version.

use websign_protocol::messages::{ClientMessage, Hello};
use websign_protocol::types::ClientInfo;
use websign_protocol::{
    AppMessage, ClientEnvelope, ErrorCode, ProtocolRange, RequestId, negotiate,
};

use crate::client::ConnectOptions;
use crate::error::ClientError;
use crate::locate::find_executable;
use crate::session::{Expect, Session, Wait};
use crate::timing::Timing;

/// Spawns the app and returns the session with the negotiated version. Any
/// failure drops the session, which reaps the child.
pub(crate) fn connect(
    options: ConnectOptions,
    timing: Timing,
) -> Result<(Session, u32), ClientError> {
    let executable = options
        .executable
        .clone()
        .or_else(find_executable)
        .ok_or_else(|| ClientError::AppMissing("no websign executable found".into()))?;
    let mut session = Session::start(&executable, timing)?;

    let id = RequestId::new("h").map_err(|error| ClientError::Connection(error.to_string()))?;
    let hello_v = ProtocolRange::CURRENT.max;
    let hello = ClientEnvelope {
        v: hello_v,
        id: id.clone(),
        message: ClientMessage::Hello(Hello {
            client: client_info(options),
            protocols: ProtocolRange::CURRENT,
            browser: None,
        }),
    };
    // A child that cannot even take `hello` has died on start: that is a
    // broken installation, not a protocol failure.
    session
        .send(&hello)
        .map_err(|_| ClientError::AppMissing("the app exited on start".into()))?;

    // An app that refuses our `hello` answers at `hello_v`, so the refusal
    // is readable even when the two version ranges do not overlap.
    let expect = Expect::HelloReply { hello_v };
    let envelope = match session.receive(expect, Some(timing.hello_reply))? {
        Wait::Message(envelope) => envelope,
        Wait::TimedOut => {
            return Err(ClientError::App {
                code: ErrorCode::Timeout,
                message: "the app did not answer hello".into(),
            });
        }
        Wait::Exited => {
            return Err(ClientError::AppMissing(
                "the app exited before answering hello".into(),
            ));
        }
    };
    if envelope.id != id {
        return Err(session.poison("the app answered a request that is not open"));
    }
    match envelope.message {
        AppMessage::Hello(reply) => {
            // Negotiate against the app's advertised range instead of
            // trusting `protocol` blindly: a reply outside our range must
            // never put us on a schema we cannot parse.
            let agreed =
                negotiate(reply.app.protocols, ProtocolRange::CURRENT).map_err(|code| {
                    ClientError::App {
                        code,
                        message: "no protocol version in common with the app".into(),
                    }
                })?;
            if agreed != reply.protocol {
                return Err(session.poison("the app chose a protocol version we do not speak"));
            }
            Ok((session, agreed))
        }
        AppMessage::Error(error) => Err(ClientError::App {
            code: error.code,
            message: error.message,
        }),
        _ => Err(session.poison("the app answered hello with another message")),
    }
}

fn client_info(options: ConnectOptions) -> ClientInfo {
    ClientInfo {
        name: options
            .client_name
            .unwrap_or_else(|| "websign-client".into()),
        version: options
            .client_version
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").into()),
    }
}
