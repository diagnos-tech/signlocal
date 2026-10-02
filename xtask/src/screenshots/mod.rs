//! `cargo xtask screenshots --from <dir> --os <name>`: copies e2e PNGs into
//! `docs/screenshots/<os>/` with stable names and rewrites that folder's
//! `SUMMARY.md` and index page (`docs/architecture/testing.md` §Screenshots).

mod index;
mod naming;

use std::path::{Path, PathBuf};

use crate::fsutil::{list_dir, write_text};
use crate::root::repo_root;
use index::{render_index, render_summary};
use naming::{Shot, valid_name};

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// `cargo xtask screenshots`.
#[derive(Debug, clap::Args)]
pub struct ScreenshotsArgs {
    /// Folder the e2e run wrote its PNGs to (searched recursively; the
    /// files are named `<window>-<state>-<theme>.png`).
    #[arg(long)]
    pub from: std::path::PathBuf,
    /// `windows`, `macos`, `ubuntu-24.04`, `fedora-42`, …
    #[arg(long)]
    pub os: String,
}

/// Copies and indexes.
pub fn run(args: &ScreenshotsArgs) -> Result<(), String> {
    let count = publish(&repo_root()?, &args.from, &args.os)?;
    println!("screenshots: {count} images for {}", args.os);
    Ok(())
}

/// Replaces `docs/screenshots/<os>/` with the PNGs found under `from`.
/// Old images go first so the index never lists a state that no longer exists.
fn publish(root: &Path, from: &Path, os: &str) -> Result<usize, String> {
    if !valid_name(os) {
        return Err(format!(
            "--os `{os}` must use lowercase letters, digits, `.`, `-` or `_`"
        ));
    }
    let mut sources = Vec::new();
    find_pngs(from, &mut sources)?;
    if sources.is_empty() {
        return Err(format!(
            "no PNG files under {}; run the e2e suite with WEBSIGN_E2E_SCREENSHOTS pointing there",
            from.display()
        ));
    }
    let shots = validate(&sources)?;

    let target = root.join("docs/screenshots").join(os);
    if target.is_dir() {
        for (name, is_dir) in list_dir(&target)? {
            if !is_dir && name.ends_with(".png") {
                std::fs::remove_file(target.join(&name))
                    .map_err(|e| format!("cannot remove old {name}: {e}"))?;
            }
        }
    }
    std::fs::create_dir_all(&target)
        .map_err(|e| format!("cannot create {}: {e}", target.display()))?;
    for (shot, source) in shots.iter().zip(&sources) {
        std::fs::copy(source, target.join(&shot.file)).map_err(|e| {
            format!(
                "cannot copy {} to {}: {e}",
                source.display(),
                target.display()
            )
        })?;
    }
    write_text(&target.join("index.md"), &render_index(os, &shots))?;
    write_text(&target.join("SUMMARY.md"), &render_summary(os, &shots))?;
    write_root_summary(root)?;
    Ok(shots.len())
}

/// Names must be safe and unique, and every file must really be a PNG.
fn validate(sources: &[PathBuf]) -> Result<Vec<Shot>, String> {
    let mut shots: Vec<Shot> = Vec::new();
    for source in sources {
        let file = source
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !valid_name(&file) {
            return Err(format!(
                "{}: file names must use lowercase letters, digits, `.`, `-` or `_`",
                source.display()
            ));
        }
        if shots.iter().any(|shot| shot.file == file) {
            return Err(format!(
                "{}: another folder has a file named {file}",
                source.display()
            ));
        }
        let head =
            std::fs::read(source).map_err(|e| format!("cannot read {}: {e}", source.display()))?;
        if !head.starts_with(PNG_SIGNATURE) {
            return Err(format!("{}: not a PNG image", source.display()));
        }
        shots.push(Shot::parse(&file));
    }
    Ok(shots)
}

fn find_pngs(folder: &Path, found: &mut Vec<PathBuf>) -> Result<(), String> {
    for (name, is_dir) in list_dir(folder)? {
        let path = folder.join(&name);
        if is_dir {
            find_pngs(&path, found)?;
        } else if name.ends_with(".png") {
            found.push(path);
        }
    }
    Ok(())
}

/// `docs/screenshots/SUMMARY.md` lists one row per OS folder.
fn write_root_summary(root: &Path) -> Result<(), String> {
    let folder = root.join("docs/screenshots");
    let mut text = String::from(
        "# docs/screenshots\n\nWindow screenshots per operating system, collected from e2e runs by `cargo xtask screenshots`.\n\n",
    );
    for (name, is_dir) in list_dir(&folder)? {
        if is_dir {
            text.push_str(&format!("- `{name}/` — {}\n", folder_description(&name)));
        }
    }
    write_text(&folder.join("SUMMARY.md"), &text)
}

/// Folders named after an OS hold window screenshots; `popup/` (written by
/// the extension's `bun run screenshots`) holds popup states and locales.
fn folder_description(name: &str) -> String {
    match name {
        "popup" => {
            "every extension popup state, light and dark, and the missing state in 7 locales"
                .to_owned()
        }
        os => format!("every window state on {os}, light and dark"),
    }
}

#[cfg(test)]
mod tests;
