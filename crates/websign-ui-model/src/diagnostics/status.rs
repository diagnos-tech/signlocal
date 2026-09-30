//! Traffic lights per tab and overall (`docs/ux.md` §8.1).

/// A light: color + icon + text, never color alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Light {
    /// Not applicable / no data (ranks lowest).
    Gray,
    Green,
    Yellow,
    Red,
}

/// Facts about browsers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowsersFacts {
    /// Installed browsers with an extension connection on record.
    pub connected: u32,
    /// Installed browsers with no extension, an outdated one, or a broken
    /// registration.
    pub with_problems: u32,
    /// The app's registration is missing in every browser.
    pub registration_missing_everywhere: bool,
}

/// Facts about devices.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DevicesFacts {
    pub devices_without_certificates: u32,
    pub drivers_failed: u32,
    pub complement_outdated: bool,
    /// Linux only.
    pub pcscd_stopped: bool,
}

/// Facts about certificates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertificatesFacts {
    pub usable: u32,
    pub expiring_within_30_days: u32,
}

/// Browsers tab light.
pub fn browsers_light(facts: &BrowsersFacts) -> Light {
    let _ = facts;
    todo!("SPEC.md §3.1")
}

/// Devices tab light.
pub fn devices_light(facts: &DevicesFacts) -> Light {
    let _ = facts;
    todo!("SPEC.md §3.1")
}

/// Certificates tab light.
pub fn certificates_light(facts: &CertificatesFacts) -> Light {
    let _ = facts;
    todo!("SPEC.md §3.1")
}

/// Overall = the worst of the tabs.
pub fn overall(lights: &[Light]) -> Light {
    let _ = lights;
    todo!("SPEC.md §3.1")
}
