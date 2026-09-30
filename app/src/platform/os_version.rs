//! The Windows release people recognize, for the report's `os:` line
//! (`docs/ux.md` §8.7): "Windows 11 (build 22631)".
//!
//! Read with `RtlGetVersion`, which, unlike `GetVersionEx`, is not capped
//! at Windows 8 by the executable's compatibility manifest. The marketing
//! version ("23H2") lives only in the registry and is left out: the build
//! number already identifies it.

/// This Windows' release; `None` elsewhere or when the call fails.
pub fn windows_release() -> Option<String> {
    #[cfg(windows)]
    {
        super::windows::os_version::numbers()
            .map(|(major, minor, build)| release_name(major, minor, build))
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// First build of Windows 11, which still reports itself as 10.0.
const WINDOWS_11_BUILD: u32 = 22_000;

/// The name of Windows `major.minor` build `build`.
fn release_name(major: u32, minor: u32, build: u32) -> String {
    let name = match (major, minor) {
        (10, 0) if build >= WINDOWS_11_BUILD => "Windows 11".to_owned(),
        (10, 0) => "Windows 10".to_owned(),
        (6, 3) => "Windows 8.1".to_owned(),
        (6, 2) => "Windows 8".to_owned(),
        (6, 1) => "Windows 7".to_owned(),
        _ => format!("Windows {major}.{minor}"),
    };
    format!("{name} (build {build})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_11_is_told_apart_by_its_build() {
        assert_eq!(release_name(10, 0, 22631), "Windows 11 (build 22631)");
        assert_eq!(release_name(10, 0, 22000), "Windows 11 (build 22000)");
        assert_eq!(release_name(10, 0, 19045), "Windows 10 (build 19045)");
    }

    #[test]
    fn older_and_unknown_versions_keep_their_numbers() {
        assert_eq!(release_name(6, 1, 7601), "Windows 7 (build 7601)");
        assert_eq!(release_name(11, 2, 30000), "Windows 11.2 (build 30000)");
    }

    #[cfg(not(windows))]
    #[test]
    fn other_systems_have_no_windows_release() {
        assert_eq!(windows_release(), None);
    }
}
