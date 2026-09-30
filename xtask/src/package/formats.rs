//! The artifact kinds and which target OS each applies to.

use super::target::{Os, Target};

/// One kind of release file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Deb,
    Rpm,
    #[value(name = "tar.gz")]
    TarGz,
    Zip,
    App,
    /// Both browser zips; independent of the target, built by `bun run zip`
    /// in `extension/`.
    Extension,
}

impl Format {
    fn os(self) -> Option<Os> {
        match self {
            Format::Deb | Format::Rpm | Format::TarGz => Some(Os::Linux),
            Format::Zip => Some(Os::Windows),
            Format::App => Some(Os::Darwin),
            Format::Extension => None,
        }
    }
}

/// The formats to produce: the requested ones, or everything the OS ships.
/// Asking for a format the target cannot produce is an error, not a skip, so
/// a CI typo cannot silently drop an artifact.
pub fn plan(target: &Target, requested: &[Format]) -> Result<Vec<Format>, String> {
    if requested.is_empty() {
        return Ok(match target.os {
            Os::Linux => vec![Format::Deb, Format::Rpm, Format::TarGz],
            Os::Windows => vec![Format::Zip],
            Os::Darwin => vec![Format::App],
        });
    }
    for format in requested {
        if format.os().is_some_and(|os| os != target.os) {
            return Err(format!(
                "{format:?} cannot be produced for {}",
                target.triple
            ));
        }
    }
    Ok(requested.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::target::parse;

    #[test]
    fn each_os_ships_its_own_formats_by_default() {
        let linux = parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(
            plan(&linux, &[]).unwrap(),
            [Format::Deb, Format::Rpm, Format::TarGz]
        );
        let windows = parse("aarch64-pc-windows-msvc").unwrap();
        assert_eq!(plan(&windows, &[]).unwrap(), [Format::Zip]);
    }

    #[test]
    fn a_format_of_another_os_is_refused() {
        let windows = parse("x86_64-pc-windows-msvc").unwrap();
        assert!(plan(&windows, &[Format::Deb]).is_err());
        assert!(plan(&windows, &[Format::Extension]).is_ok());
    }
}
