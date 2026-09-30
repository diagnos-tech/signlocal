//! The exact text "Copy diagnostics" puts on the clipboard (`docs/ux.md`
//! §8.7). English, stable, and free of personal data by construction: the
//! input types cannot carry names, document numbers, sites, serial numbers,
//! fingerprints or digests.

use super::report_lines as lines;

/// Everything the report may contain.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReportInput {
    pub app_version: String,
    /// `"msix"`, `"direct"`, `"deb"`, `"rpm"`, `"mas"`, `"app-zip"`.
    pub packaging: String,
    pub arch: String,
    pub protocol: u32,
    pub locale: String,
    /// UI scale in percent.
    pub scale_percent: u32,
    /// `"Windows 11 23H2 (10.0.22631)"`.
    pub os: String,
    /// `"wgpu/dx12"`, `"glow"`.
    pub render: String,
    pub browsers: Vec<BrowserLine>,
    pub devices: Vec<DeviceLine>,
    pub modules: Vec<ModuleLine>,
    pub certificates: CertificateCounts,
    /// `"n/a"` outside the macOS store build.
    pub complement: String,
    /// Newest last; only the newest 20 are printed.
    pub recent_errors: Vec<ErrorLine>,
}

/// `chrome 129.0 · extension 1.4.2 · host registered · last ping 2026-09-29T14:02Z`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowserLine {
    pub name: String,
    pub version: Option<String>,
    pub extension_version: Option<String>,
    pub host_registered: bool,
    /// UTC, minute precision, `YYYY-MM-DDTHH:MMZ`.
    pub last_ping: Option<String>,
}

/// `usb 0529:0620 safenet-etoken-5110 · certs 0` or a reader line with ATR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceLine {
    Usb {
        vid_pid: String,
        hint_id: Option<String>,
        certs: u32,
    },
    Reader {
        /// Already anonymized (no serial number).
        name: String,
        /// Any hex spelling (`3BD518…`, `3b d5 …`, `3B:D5:…`); printed as
        /// upper-case bytes joined by `:`.
        atr: Option<String>,
        certs: u32,
    },
}

/// A PKCS#11 module: path with the user folder replaced (`%USERPROFILE%`, `~`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleLine {
    pub path: String,
    /// `Ok((slots, tokens))` or the failure reason.
    pub result: Result<(u32, u32), String>,
    pub user_added: bool,
}

/// Counts only.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertificateCounts {
    pub usable_os: u32,
    pub usable_pkcs11: u32,
    pub deduplicated: u32,
    pub hidden_expired: u32,
    pub hidden_login_only: u32,
    pub hidden_other: u32,
    /// `("icp-brasil-a3", 1)`, printed in the order given (the producer
    /// decides; the ux example lists A3 before A1).
    pub kinds: Vec<(String, u32)>,
    /// `("rsa-2048", 3)`, printed in the order given.
    pub keys: Vec<(String, u32)>,
    pub expiring_within_30_days: u32,
}

/// `2026-09-29T14:05Z sign PinIncorrect pkcs11 CKR_PIN_INCORRECT`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLine {
    pub at: String,
    pub operation: String,
    pub code: String,
    pub source: String,
    pub native: Option<String>,
}

/// Renders the report exactly as `docs/ux.md` §8.7 shows it.
pub fn render(input: &ReportInput) -> String {
    let mut lines: Vec<String> = lines::header(input).into();
    section(
        &mut lines,
        "browsers:",
        input.browsers.iter().map(lines::browser),
    );
    section(
        &mut lines,
        "devices:",
        input.devices.iter().map(lines::device),
    );
    section(
        &mut lines,
        "pkcs11:",
        input.modules.iter().map(lines::module),
    );
    lines.push("certificates:".to_owned());
    lines.extend(lines::certificates(&input.certificates).map(|line| format!("  {line}")));
    lines.push(format!("complement: {}", input.complement));
    section(
        &mut lines,
        "recent errors (last 20):",
        lines::recent_errors(&input.recent_errors),
    );
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

/// A titled section; `  none` when it has no entries.
fn section(lines: &mut Vec<String>, title: &str, entries: impl Iterator<Item = String>) {
    lines.push(title.to_owned());
    let start = lines.len();
    lines.extend(entries.map(|entry| format!("  {entry}")));
    if lines.len() == start {
        lines.push("  none".to_owned());
    }
}

#[cfg(test)]
mod tests;
