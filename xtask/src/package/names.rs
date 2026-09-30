//! The exact artifact file names (`packaging-and-release.md` §Artifacts).
//! `install.sh`, `install.ps1` and the docs build the same names by hand, so
//! any change here must change them too.

use super::target::Arch;

pub fn deb(slug: &str, version: &str, arch: Arch) -> String {
    format!("{slug}_{version}_{}.deb", arch.deb())
}

pub fn rpm(slug: &str, version: &str, arch: Arch) -> String {
    format!("{slug}-{version}-1.{}.rpm", arch.rpm())
}

pub fn tarball(slug: &str, version: &str, arch: Arch) -> String {
    format!("{slug}-{version}-linux-{}.tar.gz", arch.short())
}

pub fn windows_zip(slug: &str, version: &str, arch: Arch) -> String {
    format!("{slug}-{version}-windows-{}.zip", arch.short())
}

pub fn macos_zip(slug: &str, version: &str) -> String {
    format!("{slug}-{version}-macos-universal.zip")
}

pub fn extension_zip(slug: &str, version: &str, browser: &str) -> String {
    format!("{slug}-extension-{version}-{browser}.zip")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_release_table() {
        assert_eq!(
            deb("websign", "0.1.0", Arch::X64),
            "websign_0.1.0_amd64.deb"
        );
        assert_eq!(
            rpm("websign", "0.1.0", Arch::Arm64),
            "websign-0.1.0-1.aarch64.rpm"
        );
        assert_eq!(
            tarball("websign", "0.1.0", Arch::X64),
            "websign-0.1.0-linux-x64.tar.gz"
        );
        assert_eq!(
            windows_zip("websign", "0.1.0", Arch::Arm64),
            "websign-0.1.0-windows-arm64.zip"
        );
        assert_eq!(
            macos_zip("websign", "0.1.0"),
            "websign-0.1.0-macos-universal.zip"
        );
        assert_eq!(
            extension_zip("websign", "0.1.0", "chromium"),
            "websign-extension-0.1.0-chromium.zip"
        );
    }
}
