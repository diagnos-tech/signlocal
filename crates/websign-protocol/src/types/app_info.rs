//! What the app says about itself in `hello` and `status`.

use serde::{Deserialize, Serialize};

use crate::version::ProtocolRange;

/// The app build that answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub struct AppInfo {
    /// Semantic version, e.g. `"1.4.0"`.
    pub version: String,
    pub protocols: ProtocolRange,
    pub os: OsName,
    /// `"x86_64"` or `"aarch64"`.
    pub arch: String,
    pub channel: Channel,
}

/// Operating system family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum OsName {
    Windows,
    Macos,
    Linux,
}

/// How this copy of the app was distributed. Decided at run time (package
/// identity, sandbox), never by a build fork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export))]
pub enum Channel {
    /// Zip, tarball, deb, rpm or `.app` from GitHub Releases.
    Direct,
    /// Microsoft Store (MSIX) or Mac App Store.
    Store,
}
