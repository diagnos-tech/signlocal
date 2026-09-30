//! Linux: a hidden `.desktop` entry that handles `x-scheme-handler/<scheme>`,
//! made the default handler with `xdg-mime`.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

use websign_project::{PRODUCT_NAME, SLUG, URL_SCHEME};

use crate::destination::Outcome;

/// How running `xdg-mime` went.
#[derive(Debug, PartialEq, Eq)]
pub enum Tool {
    Done,
    /// Not installed (minimal systems, some containers).
    Missing,
    Failed(String),
}

fn file_name() -> String {
    format!("{SLUG}-url.desktop")
}

fn mime_type() -> String {
    format!("x-scheme-handler/{URL_SCHEME}")
}

/// `xdg_mime(desktop_file, mime_type)` makes the entry the default handler.
pub fn register(
    applications: &Path,
    executable: &Path,
    dry_run: bool,
    xdg_mime: &dyn Fn(&str, &str) -> Tool,
) -> Outcome {
    let Some(exec) = executable.to_str().and_then(exec_argument) else {
        return Outcome::Failed(format!(
            "{} cannot be written in a desktop entry",
            executable.display()
        ));
    };
    if dry_run {
        return Outcome::DryRun;
    }
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName={PRODUCT_NAME}\nExec={exec} %u\n\
         MimeType={};\nNoDisplay=true\nTerminal=false\n",
        mime_type()
    );
    let path = applications.join(file_name());
    let written = fs::create_dir_all(applications).and_then(|()| fs::write(&path, entry));
    if let Err(error) = written {
        return Outcome::Failed(format!("cannot write {}: {error}", path.display()));
    }
    match xdg_mime(&file_name(), &mime_type()) {
        Tool::Done => Outcome::Written,
        Tool::Missing => Outcome::Skipped(format!(
            "wrote {}, but xdg-mime is not installed to make it the default handler",
            path.display()
        )),
        Tool::Failed(reason) => Outcome::Failed(format!("xdg-mime failed: {reason}")),
    }
}

pub fn unregister(applications: &Path, dry_run: bool) -> Outcome {
    let path = applications.join(file_name());
    if !path.exists() {
        return Outcome::NotPresent;
    }
    if dry_run {
        return Outcome::DryRun;
    }
    match fs::remove_file(&path) {
        Ok(()) => Outcome::Removed,
        Err(error) if error.kind() == ErrorKind::NotFound => Outcome::NotPresent,
        Err(error) => Outcome::Failed(format!("cannot remove {}: {error}", path.display())),
    }
}

/// Runs `xdg-mime default <desktop_file> <mime_type>`.
pub fn xdg_mime(desktop_file: &str, mime_type: &str) -> Tool {
    let status = Command::new("xdg-mime")
        .args(["default", desktop_file, mime_type])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(status) if status.success() => Tool::Done,
        Ok(status) => Tool::Failed(status.to_string()),
        Err(error) if error.kind() == ErrorKind::NotFound => Tool::Missing,
        Err(error) => Tool::Failed(error.to_string()),
    }
}

/// The program as one `Exec` argument (Desktop Entry Specification, "The
/// Exec key"): quoted when it holds reserved characters, `%` doubled, and
/// backslashes doubled once more for the string-value escape. `None` for
/// control characters, which no escape can carry.
fn exec_argument(program: &str) -> Option<String> {
    const RESERVED: &str = " \t\"'\\><~|&;$*?#()`";
    if program.chars().any(char::is_control) {
        return None;
    }
    let quoted = if program.chars().any(|c| RESERVED.contains(c)) {
        let mut inner = String::new();
        for c in program.chars() {
            if matches!(c, '"' | '`' | '$' | '\\') {
                inner.push('\\');
            }
            inner.push(c);
        }
        format!("\"{inner}\"")
    } else {
        program.to_owned()
    };
    Some(quoted.replace('%', "%%").replace('\\', "\\\\"))
}

#[cfg(test)]
mod tests;
