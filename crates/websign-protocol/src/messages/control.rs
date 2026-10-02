//! Messages that steer a connection rather than sign.

use serde::{Deserialize, Serialize};

/// Withdraws the open request with the same id. The request ends with an
/// `Aborted` error; `cancel` itself gets no reply. Unknown ids are ignored
/// (the request may have just finished).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct Cancel {}

/// Opens the diagnostics window in a separate process, so it outlives this
/// connection. Only the extension popup and desktop clients send it; the
/// extension never forwards it from a page.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct OpenDiagnostics {
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub tab: Option<DiagnosticsTab>,
}

/// A tab of the diagnostics window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum DiagnosticsTab {
    Browsers,
    Devices,
    Certificates,
    Help,
}

/// Success without data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct Done {}

crate::strict::object_serde!(Cancel, OpenDiagnostics, Done);
