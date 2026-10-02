//! Target triples the release supports and what each means for packaging.

/// Operating system of a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Windows,
    Darwin,
}

/// CPU architecture of a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
}

impl Arch {
    /// Debian and nfpm spelling.
    pub fn deb(self) -> &'static str {
        match self {
            Arch::X64 => "amd64",
            Arch::Arm64 => "arm64",
        }
    }

    /// RPM spelling.
    pub fn rpm(self) -> &'static str {
        match self {
            Arch::X64 => "x86_64",
            Arch::Arm64 => "aarch64",
        }
    }

    /// Spelling in tar.gz and zip names.
    pub fn short(self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }
}

/// A parsed target triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub triple: String,
    pub os: Os,
    /// `None` for `universal-apple-darwin`.
    pub arch: Option<Arch>,
}

impl Target {
    /// The architecture of a target that always has one.
    pub fn arch(&self) -> Result<Arch, String> {
        self.arch
            .ok_or_else(|| format!("{} has no single architecture", self.triple))
    }
}

/// The two slices of the universal macOS binary.
pub const MACOS_SLICES: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];

/// Parses one of the supported triples.
pub fn parse(triple: &str) -> Result<Target, String> {
    let (os, arch) = match triple {
        "x86_64-unknown-linux-gnu" => (Os::Linux, Some(Arch::X64)),
        "aarch64-unknown-linux-gnu" => (Os::Linux, Some(Arch::Arm64)),
        "x86_64-pc-windows-msvc" => (Os::Windows, Some(Arch::X64)),
        "aarch64-pc-windows-msvc" => (Os::Windows, Some(Arch::Arm64)),
        "x86_64-apple-darwin" | "aarch64-apple-darwin" | "universal-apple-darwin" => {
            (Os::Darwin, None)
        }
        other => return Err(format!("unsupported target `{other}`")),
    };
    Ok(Target {
        triple: triple.to_owned(),
        os,
        arch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_macos_triple_packs_the_universal_app() {
        for triple in MACOS_SLICES.into_iter().chain(["universal-apple-darwin"]) {
            let target = parse(triple).unwrap();
            assert_eq!((target.os, target.arch), (Os::Darwin, None));
        }
    }

    #[test]
    fn architectures_are_spelled_per_format() {
        let target = parse("aarch64-unknown-linux-gnu").unwrap();
        let arch = target.arch().unwrap();
        assert_eq!(
            (arch.deb(), arch.rpm(), arch.short()),
            ("arm64", "aarch64", "arm64")
        );
    }

    #[test]
    fn unknown_triples_are_refused() {
        assert!(parse("riscv64gc-unknown-linux-gnu").is_err());
    }
}
