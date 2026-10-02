//! Staging folders and files every package format shares.

use std::fs;
use std::path::{Path, PathBuf};

use super::Context;
use super::template::render;
use crate::fsutil::{read_text, write_text};

/// A clean folder `dist/.stage/<name>`; staging never touches the artifacts.
pub fn fresh(context: &Context, name: &str) -> Result<PathBuf, String> {
    let dir = context.out.join(".stage").join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("cannot clear {}: {e}", dir.display()))?;
    }
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Copies `from` to `to`, creating folders; errors name both paths.
pub fn copy(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| format!("cannot copy {} to {}: {e}", from.display(), to.display()))
}

/// Renders `packaging/<template>` into `to` with the project values plus
/// `extra`.
pub fn render_template(
    context: &Context,
    template: &str,
    to: &Path,
    extra: &[(&str, &str)],
) -> Result<(), String> {
    let mut values = context.project.values(&context.version);
    values.extend_from_slice(extra);
    let source = context.root.join("packaging").join(template);
    let text = render(&read_text(&source)?, &values).map_err(|e| format!("{template}: {e}"))?;
    write_text(to, &text)
}

/// The license texts a binary distribution must carry: ours and those of the
/// fonts and icons compiled into the app.
const LICENSES: [&str; 4] = [
    "LICENSE",
    "app/assets/fonts/OFL-Inter.txt",
    "app/assets/fonts/OFL-JetBrainsMono.txt",
    "app/assets/icons/LICENSE-Phosphor.txt",
];

/// Copies the license texts into `dir`.
pub fn licenses(context: &Context, dir: &Path) -> Result<(), String> {
    for relative in LICENSES {
        let name = Path::new(relative).file_name().unwrap_or_default();
        copy(&context.root.join(relative), &dir.join(name))?;
    }
    Ok(())
}

/// A path as text for templates.
pub fn text(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("{} is not valid UTF-8", path.display()))
}
