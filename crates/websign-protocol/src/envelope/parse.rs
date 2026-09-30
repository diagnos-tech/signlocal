//! The strict, order-defined frame parser behind `parse_client_message` and
//! `parse_app_message` (`SPEC.md` §5).

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use super::ParseError;
use super::describe::{echoable, failure};
use super::json;
use crate::error::ErrorCode;
use crate::id::RequestId;
use crate::version::ProtocolRange;

/// Which side sent the frame; the two catalogs differ in `type` values and
/// in what a version-less first message may claim.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Sender {
    Client,
    App,
}

impl Sender {
    fn types(self) -> &'static [&'static str] {
        match self {
            Sender::Client => &[
                "hello",
                "status",
                "choose",
                "sign.begin",
                "sign.digest",
                "cancel",
                "diagnostics.open",
            ],
            Sender::App => &[
                "hello",
                "status",
                "choose.result",
                "sign.need_digest",
                "sign.result",
                "done",
                "error",
            ],
        }
    }
}

/// A frame whose envelope passed every check; the body is not parsed yet.
pub(super) struct Checked {
    pub v: u32,
    pub id: RequestId,
    /// The message object: `type` and the body fields, without `v` and `id`.
    pub body: Value,
}

pub(super) fn invalid(id: Option<&RequestId>, message: impl Into<String>) -> ParseError {
    ParseError {
        id: id.cloned(),
        code: ErrorCode::InvalidRequest,
        message: message.into(),
    }
}

/// Checks steps 1 to 5 of `SPEC.md` §5.
pub(super) fn check_envelope(
    frame: &[u8],
    negotiated: Option<u32>,
    sender: Sender,
) -> Result<Checked, ParseError> {
    let Some(Value::Object(mut object)) = json::read(frame) else {
        return Err(invalid(
            None,
            "frame is not a JSON object without repeated keys",
        ));
    };

    let id = object
        .remove("id")
        .and_then(|value| serde_json::from_value::<RequestId>(value).ok())
        .ok_or_else(|| invalid(None, "id is missing or not a valid request id"))?;

    let v = object
        .remove("v")
        .and_then(|value| value.as_u64())
        .and_then(|number| u32::try_from(number).ok())
        .filter(|&number| number >= 1)
        .ok_or_else(|| invalid(Some(&id), "v is missing or not a positive integer"))?;

    let kind = match object.get("type") {
        Some(Value::String(kind)) if sender.types().contains(&kind.as_str()) => kind.clone(),
        Some(Value::String(kind)) => {
            let message = match echoable(kind) {
                Some(name) => format!("unknown message type {name}"),
                None => "unknown message type".to_owned(),
            };
            return Err(invalid(Some(&id), message));
        }
        _ => return Err(invalid(Some(&id), "type is missing or not a string")),
    };

    check_version(&object, &kind, v, negotiated, sender, &id)?;
    Ok(Checked {
        v,
        id,
        body: Value::Object(object),
    })
}

fn check_version(
    object: &Map<String, Value>,
    kind: &str,
    v: u32,
    negotiated: Option<u32>,
    sender: Sender,
    id: &RequestId,
) -> Result<(), ParseError> {
    match negotiated {
        Some(n) if v != n => Err(invalid(
            Some(id),
            format!("message version {v} does not match the negotiated version {n}"),
        )),
        Some(_) => Ok(()),
        None if kind != "hello" => Err(invalid(Some(id), "hello must be the first message")),
        None => match sender {
            Sender::Client => {
                let offered = object
                    .get("protocols")
                    .and_then(|value| serde_json::from_value::<ProtocolRange>(value.clone()).ok());
                match offered {
                    Some(range) if !(range.min..=range.max).contains(&v) => Err(invalid(
                        Some(id),
                        "message version is outside hello.protocols",
                    )),
                    _ => Ok(()),
                }
            }
            Sender::App => match object.get("protocol").and_then(Value::as_u64) {
                Some(protocol) if protocol != u64::from(v) => Err(invalid(
                    Some(id),
                    "message version does not match hello.protocol",
                )),
                _ => Ok(()),
            },
        },
    }
}

/// Step 6: deserializes the body strictly, naming the offending field on
/// failure without echoing any value.
pub(super) fn parse_body<T: DeserializeOwned>(checked: &Checked) -> Result<T, ParseError> {
    T::deserialize(&checked.body).map_err(|error| {
        invalid(
            Some(&checked.id),
            failure::<T>(&error.to_string(), &checked.body),
        )
    })
}
