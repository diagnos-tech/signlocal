//! The lines of the report about this copy of the app and the OS
//! (`docs/ux.md` §8.7): packaging and operating system.

use crate::platform::channel::InstallFormat;

/// The report's name for how the app was installed.
pub fn packaging_name(format: InstallFormat) -> &'static str {
    match format {
        InstallFormat::Archive if cfg!(target_os = "macos") => "app-zip",
        InstallFormat::Archive => "direct",
        InstallFormat::Deb => "deb",
        InstallFormat::Rpm => "rpm",
        InstallFormat::Flatpak => "flatpak",
        InstallFormat::Msix => "msix",
        InstallFormat::MacAppStore => "mas",
    }
}

/// `"Ubuntu 24.04.1 LTS (6.8.0-45-generic)"`, `"macOS 15.1"`,
/// `"Windows 11 (build 22631)"`:
/// the release people recognize, then the kernel or build when cheap to
/// read. Never the computer's name.
pub fn os_description() -> String {
    let name = release_name();
    match kernel_release() {
        Some(kernel) => format!("{name} ({kernel})"),
        None => name,
    }
}

#[cfg(target_os = "linux")]
fn release_name() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| os_release_field(&text, "PRETTY_NAME"))
        .unwrap_or_else(|| "Linux".to_owned())
}

#[cfg(target_os = "macos")]
fn release_name() -> String {
    // `sw_vers` is part of every macOS install and answers instantly.
    let version = std::process::Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    match version {
        Some(version) => format!("macOS {version}"),
        None => "macOS".to_owned(),
    }
}

#[cfg(windows)]
fn release_name() -> String {
    crate::platform::os_version::windows_release().unwrap_or_else(|| "Windows".to_owned())
}

#[cfg(target_os = "linux")]
fn kernel_release() -> Option<String> {
    let text = std::fs::read_to_string("/proc/sys/kernel/osrelease").ok()?;
    Some(text.trim().to_owned()).filter(|text| !text.is_empty())
}

#[cfg(not(target_os = "linux"))]
fn kernel_release() -> Option<String> {
    None
}

/// The value of `key` in an `os-release` file, unquoted.
#[cfg(any(target_os = "linux", test))]
fn os_release_field(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let value = line.strip_prefix(key)?.strip_prefix('=')?;
        let value = value.trim().trim_matches(['"', '\'']);
        (!value.is_empty()).then(|| value.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_release_values_are_unquoted() {
        let text = "NAME=\"Ubuntu\"\nPRETTY_NAME=\"Ubuntu 24.04.1 LTS\"\nID=ubuntu\n";
        assert_eq!(
            os_release_field(text, "PRETTY_NAME").as_deref(),
            Some("Ubuntu 24.04.1 LTS")
        );
        assert_eq!(os_release_field(text, "VERSION"), None);
    }

    #[test]
    fn packaging_names_match_the_report() {
        assert_eq!(packaging_name(InstallFormat::Msix), "msix");
        assert_eq!(packaging_name(InstallFormat::Deb), "deb");
        assert_eq!(packaging_name(InstallFormat::MacAppStore), "mas");
    }
}
