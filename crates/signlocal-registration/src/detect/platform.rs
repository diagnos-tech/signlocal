//! The OS calls detection needs, with inert stand-ins elsewhere so the
//! per-OS logic compiles (and is tested) everywhere.

use std::path::Path;

/// The version resource of a Windows executable (`1.2.3.4`).
pub fn file_version(path: &Path) -> Option<String> {
    #[cfg(windows)]
    return super::file_version::read(path);
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

/// Whether LaunchServices knows an app with `bundle_id`, and its
/// `CFBundleShortVersionString`.
pub fn find_bundle(bundle_id: &str) -> Option<Option<String>> {
    #[cfg(target_os = "macos")]
    return super::launch_services::find(bundle_id);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bundle_id;
        None
    }
}

/// Installed Flatpak application IDs (user and system installations); empty
/// when Flatpak is absent.
pub fn flatpak_apps() -> Vec<String> {
    let Ok(output) = std::process::Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}
