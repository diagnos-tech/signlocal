//! Direct or store build, decided at run time (`docs/plan.md` D10).
//!
//! The same binary ships in every package; how it was installed is read from
//! the OS (package identity, sandbox entitlement) or from where the binary
//! lives, never from a build flag.

use websign_protocol::types::Channel;

/// How this copy of the app was installed. Diagnostics shows it; the store
/// formats decide [`current`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallFormat {
    /// Unpacked by hand or by `install.sh`/`install.ps1`: the Windows zip,
    /// the macOS `.app` zip, the Linux tar.gz, or a development build.
    Archive,
    /// Linux `.deb` (the binary is listed by dpkg).
    Deb,
    /// Linux `.rpm` (the binary is in `/usr` on an RPM system).
    Rpm,
    /// Linux Flatpak sandbox.
    Flatpak,
    /// Windows MSIX (the process has a package identity).
    Msix,
    /// Mac App Store (the process runs in the App Sandbox).
    MacAppStore,
}

impl InstallFormat {
    /// The distribution channel of this format.
    ///
    /// Flatpak is `Direct`: no Flathub build is planned, so one can only be
    /// a local build; `Store` means the Microsoft and Mac App Stores.
    pub fn channel(self) -> Channel {
        match self {
            InstallFormat::Msix | InstallFormat::MacAppStore => Channel::Store,
            InstallFormat::Archive
            | InstallFormat::Deb
            | InstallFormat::Rpm
            | InstallFormat::Flatpak => Channel::Direct,
        }
    }
}

/// `Store` when running with an MSIX package identity or inside the Mac App
/// Store sandbox; `Direct` otherwise.
pub fn current() -> Channel {
    install_format().channel()
}

/// How this copy was installed; [`InstallFormat::Archive`] when nothing
/// more specific is detected.
pub fn install_format() -> InstallFormat {
    super::os::channel::install_format()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_os_stores_are_the_store_channel() {
        assert_eq!(InstallFormat::Msix.channel(), Channel::Store);
        assert_eq!(InstallFormat::MacAppStore.channel(), Channel::Store);
        for direct in [
            InstallFormat::Archive,
            InstallFormat::Deb,
            InstallFormat::Rpm,
            InstallFormat::Flatpak,
        ] {
            assert_eq!(direct.channel(), Channel::Direct, "{direct:?}");
        }
    }
}
