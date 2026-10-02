//! `hello`: the first message of every connection.

use serde::{Deserialize, Serialize};

use crate::types::{AppInfo, BrowserInfo, ClientInfo};
use crate::version::ProtocolRange;

/// Opens a connection. Any other first message is refused with
/// `InvalidRequest` and the connection is closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct Hello {
    pub client: ClientInfo,
    /// Protocol versions the client speaks.
    pub protocols: ProtocolRange,
    /// Required over native messaging, refused over `websign connect`.
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub browser: Option<BrowserInfo>,
}

/// The app's answer: who it is and the version both sides will speak.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct HelloReply {
    pub app: AppInfo,
    /// The negotiated version; every later message follows its schema.
    pub protocol: u32,
}

crate::strict::object_serde!(Hello, HelloReply);
