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
    if facts.connected == 0 || facts.registration_missing_everywhere {
        Light::Red
    } else if facts.with_problems > 0 {
        Light::Yellow
    } else {
        Light::Green
    }
}

/// Devices tab light.
pub fn devices_light(facts: &DevicesFacts) -> Light {
    if facts.pcscd_stopped {
        Light::Red
    } else if facts.devices_without_certificates > 0
        || facts.drivers_failed > 0
        || facts.complement_outdated
    {
        Light::Yellow
    } else {
        Light::Green
    }
}

/// Certificates tab light.
pub fn certificates_light(facts: &CertificatesFacts) -> Light {
    if facts.usable == 0 {
        Light::Red
    } else if facts.expiring_within_30_days > 0 {
        Light::Yellow
    } else {
        Light::Green
    }
}

/// Overall = the worst of the tabs; no tabs is `Gray`.
pub fn overall(lights: &[Light]) -> Light {
    lights.iter().copied().max().unwrap_or(Light::Gray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browsers() {
        let ok = BrowsersFacts {
            connected: 1,
            ..Default::default()
        };
        assert_eq!(browsers_light(&ok), Light::Green);
        assert_eq!(browsers_light(&BrowsersFacts::default()), Light::Red);
        let problem = BrowsersFacts {
            with_problems: 1,
            ..ok.clone()
        };
        assert_eq!(browsers_light(&problem), Light::Yellow);
        let missing = BrowsersFacts {
            registration_missing_everywhere: true,
            ..problem
        };
        assert_eq!(browsers_light(&missing), Light::Red);
    }

    #[test]
    fn devices() {
        assert_eq!(devices_light(&DevicesFacts::default()), Light::Green);
        let outdated = DevicesFacts {
            complement_outdated: true,
            ..Default::default()
        };
        assert_eq!(devices_light(&outdated), Light::Yellow);
        let stopped = DevicesFacts {
            pcscd_stopped: true,
            ..outdated
        };
        assert_eq!(devices_light(&stopped), Light::Red);
    }

    #[test]
    fn certificates_and_overall() {
        let none = CertificatesFacts::default();
        assert_eq!(certificates_light(&none), Light::Red);
        let expiring = CertificatesFacts {
            usable: 2,
            expiring_within_30_days: 1,
        };
        assert_eq!(certificates_light(&expiring), Light::Yellow);
        assert_eq!(
            certificates_light(&CertificatesFacts {
                usable: 2,
                expiring_within_30_days: 0
            }),
            Light::Green
        );
        assert_eq!(overall(&[]), Light::Gray);
        assert_eq!(
            overall(&[Light::Green, Light::Red, Light::Yellow]),
            Light::Red
        );
        assert_eq!(overall(&[Light::Gray, Light::Green]), Light::Green);
    }
}
