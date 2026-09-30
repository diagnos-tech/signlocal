//! `SHA256SUMS`: the hash of every release file, in the format
//! `sha256sum -c` reads and `install.sh` / `install.ps1` verify against.

use std::fs::File;
use std::path::{Path, PathBuf};

use super::sha256::hex_digest;
use super::stage::copy;
use crate::fsutil::{list_dir, write_text};

const FILE_NAME: &str = "SHA256SUMS";
/// Published next to the artifacts so one checksum file covers them too.
const SCRIPTS: [&str; 2] = ["install.sh", "install.ps1"];

/// One `<hash>  <name>` line per regular file in `out`, sorted by name.
fn lines(out: &Path) -> Result<String, String> {
    let mut text = String::new();
    for (name, is_dir) in list_dir(out)? {
        if is_dir || name == FILE_NAME || name.starts_with('.') {
            continue;
        }
        let path = out.join(&name);
        let mut file =
            File::open(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let hash =
            hex_digest(&mut file).map_err(|e| format!("cannot hash {}: {e}", path.display()))?;
        text.push_str(&format!("{hash}  {name}\n"));
    }
    Ok(text)
}

/// Copies the install scripts into `out` and writes `SHA256SUMS`.
pub fn write(root: &Path, out: &Path) -> Result<PathBuf, String> {
    for script in SCRIPTS {
        copy(
            &root.join("scripts/install").join(script),
            &out.join(script),
        )?;
    }
    let file = out.join(FILE_NAME);
    write_text(&file, &lines(out)?)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn lines_use_the_two_space_format_and_skip_hidden_and_itself() {
        let dir = TempDir::new();
        dir.write("b.zip", "abc");
        dir.write("a.deb", "");
        dir.write(".stage/x", "ignored");
        dir.write("SHA256SUMS", "old");
        let text = lines(dir.path()).unwrap();
        assert_eq!(
            text,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  a.deb\n\
             ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  b.zip\n"
        );
    }
}
