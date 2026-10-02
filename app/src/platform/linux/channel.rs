//! How the Linux build was installed, read from where the binary lives.

use std::path::Path;

use crate::platform::channel::InstallFormat;

/// Folders RPM keeps its database in (older and newer layouts).
const RPM_DATABASES: [&str; 2] = ["/var/lib/rpm", "/usr/lib/sysimage/rpm"];

pub fn install_format() -> InstallFormat {
    // Written by Flatpak into every sandbox; nothing else creates it.
    if Path::new("/.flatpak-info").exists() {
        return InstallFormat::Flatpak;
    }
    let Ok(executable) = std::env::current_exe() else {
        return InstallFormat::Archive;
    };
    let dpkg_list =
        std::fs::read_to_string(format!("/var/lib/dpkg/info/{}.list", websign_project::SLUG)).ok();
    let rpm_system = RPM_DATABASES.iter().any(|dir| Path::new(dir).is_dir());
    classify(&executable, dpkg_list.as_deref(), rpm_system)
}

/// `dpkg_list` is our package's dpkg file list, when dpkg has one.
/// The `.rpm` puts the binary in `/usr/bin`; `install.sh` never writes into
/// `/usr` outside `/usr/local`, so a binary there on an RPM system came from
/// the package.
fn classify(executable: &Path, dpkg_list: Option<&str>, rpm_system: bool) -> InstallFormat {
    let listed =
        dpkg_list.is_some_and(|list| list.lines().any(|line| Path::new(line) == executable));
    if listed {
        InstallFormat::Deb
    } else if rpm_system && executable.starts_with("/usr") && !executable.starts_with("/usr/local")
    {
        InstallFormat::Rpm
    } else {
        InstallFormat::Archive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BINARY: &str = "/usr/bin/websign";

    #[test]
    fn a_binary_dpkg_lists_is_the_deb() {
        let list = "/.\n/usr\n/usr/bin\n/usr/bin/websign\n";
        assert_eq!(
            classify(Path::new(BINARY), Some(list), true),
            InstallFormat::Deb
        );
        assert_eq!(
            classify(Path::new("/home/u/.local/bin/websign"), Some(list), false),
            InstallFormat::Archive
        );
    }

    #[test]
    fn a_binary_in_usr_on_an_rpm_system_is_the_rpm() {
        assert_eq!(classify(Path::new(BINARY), None, true), InstallFormat::Rpm);
        assert_eq!(
            classify(Path::new("/usr/local/bin/websign"), None, true),
            InstallFormat::Archive
        );
        assert_eq!(
            classify(Path::new(BINARY), None, false),
            InstallFormat::Archive
        );
    }
}
