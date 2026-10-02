//! Page ↔ extension messages (`window.postMessage`).
//!
//! The SDK never talks to the app. It posts these objects to its own window;
//! the extension's content script accepts only same-window, same-origin
//! messages whose `source` is [`PAGE_SOURCE`], rebuilds them field by field,
//! and the background validates them again before adding the browser-reported
//! [`crate::types::WebContext`] and forwarding them as native messages.
//!
//! Pages cannot send `hello` (the extension owns the connection) or
//! `diagnostics.open` (a site must not pop up app windows), and cannot set
//! `web`: the types below have no such fields.

use serde::{Deserialize, Serialize};

use crate::error::WireError;
use crate::id::RequestId;
use crate::messages::{Cancel, ChooseResult, NeedDigest, SignDigest, SignResult};
use crate::types::{AppInfo, CertificateFilter, FingerprintHex, HashName, SignatureAlgorithmName};
use crate::version::ProtocolRange;

/// `source` of messages the SDK posts.
pub const PAGE_SOURCE: &str = "websign-page";

/// `source` of messages the content script posts.
pub const EXTENSION_SOURCE: &str = "websign-extension";

/// SDK → content script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    remote = "Self",
    tag = "kind",
    rename_all = "camelCase",
    deny_unknown_fields
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum PageToExtension {
    /// Asks the content script to announce itself again (the SDK may load
    /// after the first announcement).
    Discover { source: String },
    /// A request, or a continuation of one.
    Request {
        source: String,
        /// Unique within the page; the extension maps it to a connection-wide id.
        id: RequestId,
        message: Box<PageRequest>,
    },
}

/// What a page may ask.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", tag = "type", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum PageRequest {
    #[serde(rename = "status")]
    Status {},
    #[serde(rename = "choose")]
    Choose {
        #[serde(
            default,
            deserialize_with = "crate::strict::present",
            skip_serializing_if = "Option::is_none"
        )]
        #[cfg_attr(feature = "typescript", ts(optional))]
        filter: Option<CertificateFilter>,
    },
    #[serde(rename = "sign.begin", rename_all = "camelCase")]
    SignBegin {
        hash: HashName,
        #[serde(
            default,
            deserialize_with = "crate::strict::present",
            skip_serializing_if = "Option::is_none"
        )]
        #[cfg_attr(feature = "typescript", ts(optional))]
        algorithms: Option<Vec<SignatureAlgorithmName>>,
        #[serde(
            default,
            deserialize_with = "crate::strict::present",
            skip_serializing_if = "Option::is_none"
        )]
        #[cfg_attr(feature = "typescript", ts(optional))]
        certificate: Option<FingerprintHex>,
    },
    #[serde(rename = "sign.digest")]
    SignDigest(SignDigest),
    #[serde(rename = "cancel")]
    Cancel(Cancel),
}

/// Content script → SDK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    remote = "Self",
    tag = "kind",
    rename_all = "camelCase",
    deny_unknown_fields
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum ExtensionToPage {
    /// Posted at `document_start`, on `load`, and in answer to `discover`.
    Announce {
        source: String,
        extension: ExtensionInfo,
        /// Protocol versions the extension speaks.
        protocols: ProtocolRange,
    },
    /// A reply to (or a `sign.need_digest` for) the page request `id`.
    Message {
        source: String,
        id: RequestId,
        message: Box<PageReply>,
    },
}

/// The extension build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct ExtensionInfo {
    pub version: String,
    /// The browser, as the extension detected it.
    pub browser: crate::types::BrowserName,
}

/// What the page receives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", tag = "type", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum PageReply {
    #[serde(rename = "status")]
    Status(PageStatus),
    #[serde(rename = "choose.result")]
    ChooseResult(ChooseResult),
    #[serde(rename = "sign.need_digest")]
    NeedDigest(NeedDigest),
    #[serde(rename = "sign.result")]
    SignResult(SignResult),
    #[serde(rename = "error")]
    Error(WireError),
}

/// `status()` as the extension answers it: its own facts plus the app's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct PageStatus {
    pub extension: ExtensionInfo,
    /// Absent when the app is missing or did not answer.
    #[serde(
        default,
        deserialize_with = "crate::strict::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "typescript", ts(optional))]
    pub app: Option<AppInfo>,
    /// The app is older than the extension's `MIN_APP_VERSION`.
    pub app_outdated: bool,
    pub remembered: bool,
}

crate::strict::object_serde!(
    PageToExtension,
    PageRequest,
    ExtensionToPage,
    ExtensionInfo,
    PageReply,
    PageStatus
);
