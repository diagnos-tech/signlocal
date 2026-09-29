//! The JSON messages exchanged with the extension.
//!
//! Every request carries `"v": 1`, an `"id"` the reply echoes, and a `"type"`.
//! Unknown fields are ignored so newer extensions can talk to older hosts;
//! unknown versions and types are refused with a typed error, never guessed.

use serde::Deserialize;
use serde_json::{Map, Value, json};

/// The only protocol version this host speaks.
pub const PROTOCOL_VERSION: u64 = 1;

/// Longest request id echoed back. Ids are opaque to the host; the cap keeps
/// a hostile page from bouncing large strings through the log or the reply.
const MAX_ID_LEN: usize = 128;

/// Stable machine-readable error codes. Never rename one: the SDK maps them
/// to its typed errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    UnsupportedVersion,
    UnknownType,
    DigestLength,
    NotFound,
    Cancelled,
    WrongPin,
    PinRequired,
    PinLocked,
    Unsupported,
    Internal,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::UnsupportedVersion => "unsupported_version",
            Self::UnknownType => "unknown_type",
            Self::DigestLength => "digest_length",
            Self::NotFound => "not_found",
            Self::Cancelled => "cancelled",
            Self::WrongPin => "wrong_pin",
            Self::PinRequired => "pin_required",
            Self::PinLocked => "pin_locked",
            Self::Unsupported => "unsupported",
            Self::Internal => "internal",
        }
    }
}

/// A failure to report to the extension.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}: {message}", code.as_str())]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
}

impl ProtocolError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Facts the extension volunteers on `ping`, so the app can record every
/// extension that connects. Informational only; never trusted for decisions.
#[derive(Debug, Default, Deserialize)]
pub struct ClientInfo {
    pub extension_version: Option<String>,
    pub user_agent: Option<String>,
    /// Why the extension pinged: `startup`, `installed`, `page`.
    pub reason: Option<String>,
}

/// Text fields of a `sign` request, still unvalidated.
#[derive(Debug, Deserialize)]
pub struct SignFields {
    pub fingerprint: String,
    pub hash: String,
    pub algorithm: String,
    /// Base64 of the digest.
    pub digest: String,
}

#[derive(Debug)]
pub enum Request {
    Ping(Option<ClientInfo>),
    List,
    Sign(SignFields),
}

impl Request {
    /// The `"type"` the request arrived with; a fixed set, safe to log.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Ping(_) => "ping",
            Self::List => "list",
            Self::Sign(_) => "sign",
        }
    }
}

/// A request that has passed the envelope checks, or why it did not.
#[derive(Debug)]
pub struct Parsed {
    /// Echoed in the reply; `None` when the request had no usable id.
    pub id: Option<String>,
    pub request: Result<Request, ProtocolError>,
}

/// Validates the envelope (JSON, object, version, id, type) and extracts the
/// typed request.
pub fn parse_request(frame: &[u8]) -> Parsed {
    let fail = |id: Option<String>, code, message: &str| Parsed {
        id,
        request: Err(ProtocolError::new(code, message)),
    };
    let Ok(value) = serde_json::from_slice::<Value>(frame) else {
        return fail(None, ErrorCode::BadRequest, "request is not valid JSON");
    };
    let Some(object) = value.as_object() else {
        return fail(None, ErrorCode::BadRequest, "request is not a JSON object");
    };

    let id = match object.get("id") {
        Some(Value::String(id)) if id.len() <= MAX_ID_LEN => Some(id.clone()),
        Some(_) => return fail(None, ErrorCode::BadRequest, "\"id\" must be a short string"),
        None => None,
    };
    match object.get("v") {
        Some(version) if version.as_u64() == Some(PROTOCOL_VERSION) => {}
        Some(Value::Number(number)) => {
            let message = format!(
                "protocol version {number} is not supported (supported: {PROTOCOL_VERSION})"
            );
            return fail(id, ErrorCode::UnsupportedVersion, &message);
        }
        _ => return fail(id, ErrorCode::BadRequest, "missing or invalid \"v\""),
    }

    let request = match object.get("type").and_then(Value::as_str) {
        Some("ping") => Ok(Request::Ping(
            object
                .get("client")
                .and_then(|client| ClientInfo::deserialize(client).ok()),
        )),
        Some("list") => Ok(Request::List),
        Some("sign") => SignFields::deserialize(&value)
            .map(Request::Sign)
            .map_err(|error| ProtocolError::new(ErrorCode::BadRequest, error.to_string())),
        Some(_) => Err(ProtocolError::new(
            ErrorCode::UnknownType,
            "unknown request type",
        )),
        None => Err(ProtocolError::new(
            ErrorCode::BadRequest,
            "missing \"type\"",
        )),
    };
    Parsed { id, request }
}

/// A success reply: the envelope plus the type-specific fields.
pub fn success(id: Option<&str>, kind: &str, fields: Value) -> Value {
    let mut reply = Map::new();
    reply.insert("v".into(), json!(PROTOCOL_VERSION));
    reply.insert("id".into(), id.map_or(Value::Null, |id| json!(id)));
    reply.insert("ok".into(), json!(true));
    reply.insert("type".into(), json!(kind));
    if let Value::Object(fields) = fields {
        reply.extend(fields);
    }
    Value::Object(reply)
}

/// An error reply.
pub fn failure(id: Option<&str>, error: &ProtocolError) -> Value {
    let mut detail = json!({ "code": error.code.as_str(), "message": error.message });
    if error.code == ErrorCode::UnsupportedVersion {
        detail["supported"] = json!([PROTOCOL_VERSION]);
    }
    json!({
        "v": PROTOCOL_VERSION,
        "id": id,
        "ok": false,
        "error": detail,
    })
}

#[cfg(test)]
mod tests;
