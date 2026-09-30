//! `status` and `choose`: questions that never sign anything.

use serde::{Deserialize, Serialize};

use crate::types::{AppInfo, Certificate, CertificateFilter, WebContext};

/// Asks for the app's state. Never opens a window.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct Status {
    /// Native messaging only (see [`WebContext`]).
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub web: Option<WebContext>,
}

/// The app's state as far as this caller may know it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct StatusReply {
    pub app: AppInfo,
    /// Whether the person ticked "Remember this site/app" for this caller, so
    /// `choose` will answer without a window.
    pub remembered: bool,
}

/// `certificates()`: the certificate the person picks, never the list.
///
/// A remembered caller gets the certificates it already used, without a
/// window; anyone else gets the confirmation window in "choose" mode.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct Choose {
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub web: Option<WebContext>,
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub filter: Option<CertificateFilter>,
}

/// The chosen certificate (window) or the remembered ones (no window).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct ChooseResult {
    /// Never empty: no certificate ends the request with an error instead.
    pub certificates: Vec<Certificate>,
}

crate::strict::object_serde!(Status, StatusReply, Choose, ChooseResult);
