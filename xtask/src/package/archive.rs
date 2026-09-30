//! Zip and tar.gz creation through the system tools, so xtask needs no
//! archive dependency: `zip` (Linux, macOS, Git for Windows) with `tar -a`
//! (bsdtar, shipped with Windows 10+) as the fallback.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use super::tool::run;

/// Names of the entries directly inside `dir`, sorted.
fn entries(dir: &Path) -> Result<Vec<String>, String> {
    let list = crate::fsutil::list_dir(dir)?;
    Ok(list.into_iter().map(|(name, _)| name).collect())
}

fn absolute(path: &Path) -> Result<std::path::PathBuf, String> {
    std::path::absolute(path).map_err(|e| format!("cannot resolve {}: {e}", path.display()))
}

/// Zips every entry of `dir` at the top level of `out`. Symlinks are stored
/// as links (macOS bundles need it) and extra file attributes are dropped so
/// archives do not depend on the builder's user.
pub fn zip(dir: &Path, out: &Path) -> Result<(), String> {
    let out = absolute(out)?;
    let _ = fs::remove_file(&out);
    let names = entries(dir)?;
    let attempt = Command::new("zip")
        .args(["-q", "-r", "-y", "-X"])
        .arg(&out)
        .args(&names)
        .current_dir(dir)
        .status();
    match attempt {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("`zip` failed ({status})")),
        Err(error) if error.kind() == ErrorKind::NotFound => run(
            Command::new("tar")
                .args(["-a", "-c", "-f"])
                .arg(&out)
                .args(&names)
                .current_dir(dir),
            "install zip or a tar that writes zip files (bsdtar)",
        ),
        Err(error) => Err(format!("cannot run `zip`: {error}")),
    }
}

/// Packs the folder `dir` (as its own top-level entry) into `out`.
pub fn tar_gz(dir: &Path, out: &Path) -> Result<(), String> {
    let out = absolute(out)?;
    let name = dir
        .file_name()
        .ok_or_else(|| format!("{} has no name", dir.display()))?;
    let parent = dir
        .parent()
        .ok_or_else(|| format!("{} has no parent", dir.display()))?;
    run(
        Command::new("tar")
            .arg("-czf")
            .arg(&out)
            .arg("-C")
            .arg(parent)
            .arg(name),
        "install tar",
    )
}
