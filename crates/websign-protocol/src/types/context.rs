//! Who is asking: facts the transport attaches to requests.

use serde::{Deserialize, Serialize};

/// Where a web request came from, as the **browser** reported it to the
/// extension (`MessageSender`), never as the page claims.
///
/// Required on `status`, `choose` and `sign.begin` arriving over native
/// messaging; refused on `websign connect`, where the app identifies the
/// calling program itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct WebContext {
    /// Serialized origin of the frame that called the SDK
    /// (`"https://app.example.com"`, default port omitted).
    pub origin: String,
    /// Serialized origin of the tab's top-level document. Equal to `origin`
    /// unless the SDK runs inside an iframe.
    pub top_origin: String,
}

/// The calling program, as a client library describes itself in `hello`.
/// Informational (logs, diagnostics); the app never trusts it for decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct ClientInfo {
    /// `"websign-extension"`, `"@websign/desktop"`, `"websign-client"`, or a
    /// program's own name; at most 64 bytes.
    pub name: String,
    /// The client's version; at most 64 bytes.
    pub version: String,
}

/// The browser hosting the extension. Sent once per connection in `hello`,
/// so the window can say "via Chrome" and diagnostics can list connections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct BrowserInfo {
    pub name: BrowserName,
    /// Major.minor as the browser reports it; at most 64 bytes.
    pub version: String,
    /// Why the extension connected (diagnostics only).
    pub reason: HelloReason,
}

/// Browsers the extension recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum BrowserName {
    Chrome,
    Chromium,
    Edge,
    Brave,
    Opera,
    Vivaldi,
    Firefox,
    Safari,
    Other,
}

/// What made the extension open the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum HelloReason {
    /// `runtime.onStartup`: the browser started.
    Startup,
    /// `runtime.onInstalled`: install or update.
    Installed,
    /// A page request.
    Page,
    /// The toolbar popup opened.
    Popup,
}

crate::strict::object_serde!(WebContext, ClientInfo, BrowserInfo);
