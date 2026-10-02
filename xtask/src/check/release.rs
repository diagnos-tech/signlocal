//! Release inputs: lockstep versions, license and notice files, install
//! scripts in sync with project.toml, and a release binary without the e2e
//! marker.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::{installers, versions};

/// Present in any binary built with `--features e2e`; it must never ship.
const E2E_MARKER: &[u8] = b"WEBSIGN_E2E_BUILD";
/// Files every release carries: each part's license text.
const LICENSES: [&str; 5] = [
    "LICENSE",
    "LICENSE-CC0",
    "sdk/LICENSE",
    "clients/node/LICENSE",
    "clients/rust/LICENSE",
];
/// The unsigned-build notice the README must show (packaging doc §Unsigned
/// builds): one word per platform that warns, so the section cannot be dropped.
const NOTICE_WORDS: [&str; 2] = ["SmartScreen", "Gatekeeper"];

pub fn check(root: &Path, binary: Option<&Path>) -> Result<Vec<String>, String> {
    let tag = std::env::var("GITHUB_REF_TYPE")
        .ok()
        .filter(|kind| kind == "tag")
        .and_then(|_| std::env::var("GITHUB_REF_NAME").ok());
    let mut problems = versions::check(root, tag.as_deref())?;
    for license in LICENSES {
        if !root.join(license).is_file() {
            problems.push(format!("{license}: missing license text"));
        }
    }
    let readme = std::fs::read_to_string(root.join("README.md")).unwrap_or_default();
    for word in NOTICE_WORDS.iter().filter(|word| !readme.contains(**word)) {
        problems.push(format!(
            "README.md: unsigned-build notice does not mention {word}"
        ));
    }
    problems.extend(installers::check(root)?);
    if let Some(binary) = binary {
        problems.extend(binary_problem(binary)?);
    }
    Ok(problems)
}

fn binary_problem(binary: &Path) -> Result<Option<String>, String> {
    let file = File::open(binary)
        .map_err(|e| format!("cannot open release binary {}: {e}", binary.display()))?;
    let found = contains(file, E2E_MARKER)
        .map_err(|e| format!("cannot read release binary {}: {e}", binary.display()))?;
    Ok(found.then(|| {
        format!(
            "{}: contains {}; it was built with the e2e feature",
            binary.display(),
            String::from_utf8_lossy(E2E_MARKER)
        )
    }))
}

/// Streams `reader`, keeping the last `needle.len() - 1` bytes between
/// chunks so a marker split across two reads is still found.
fn contains(mut reader: impl Read, needle: &[u8]) -> std::io::Result<bool> {
    let mut window: Vec<u8> = Vec::new();
    let mut chunk = vec![0u8; 1 << 20];
    loop {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            return Ok(false);
        }
        window.extend_from_slice(&chunk[..read]);
        if window.windows(needle.len()).any(|w| w == needle) {
            return Ok(true);
        }
        let keep = needle.len() - 1;
        window.drain(..window.len().saturating_sub(keep));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Yields one byte per read: the worst case for a split marker.
    struct Trickle<'a>(&'a [u8]);

    impl Read for Trickle<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let Some((first, rest)) = self.0.split_first() else {
                return Ok(0);
            };
            buf[0] = *first;
            self.0 = rest;
            Ok(1)
        }
    }

    #[test]
    fn finds_a_marker_split_across_reads() {
        assert!(contains(Trickle(b"xxWEBSIGN_E2E_BUILDyy"), E2E_MARKER).unwrap());
    }

    #[test]
    fn a_clean_binary_passes() {
        assert!(!contains(Trickle(b"WEBSIGN_E2E_BUIL"), E2E_MARKER).unwrap());
    }
}
