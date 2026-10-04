//! The Linux app-menu entry that opens diagnostics (`websign` with no
//! arguments).
//!
//! Packages and `install.sh` ship their own `websign.desktop` with icons;
//! this entry exists for a binary unpacked by hand. So it is written only
//! when no entry exists yet, marked as ours, and removed only when it
//! carries the mark — never a file a package owns.

use std::path::{Path, PathBuf};

use websign_registration::Outcome;

use crate::cli::report::{Kind, Step};

/// Identifies the entries this command wrote.
const MARK: &str = "X-SignLocal-Written-By=websign install";
/// Where packages put theirs.
const SYSTEM_APPLICATIONS: &str = "/usr/share/applications";

/// Writes (`install`) or removes (`uninstall`) the entry; nothing off Linux.
pub fn apply(install: bool, dry_run: bool) -> Option<Step> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let file_name = format!("{}.desktop", websign_project::SLUG);
    let Some(dir) = applications_dir() else {
        return Some(step(
            String::new(),
            Outcome::Skipped("no home folder".to_owned()),
        ));
    };
    let file = dir.join(&file_name);
    let outcome = if install {
        let system = Path::new(SYSTEM_APPLICATIONS).join(&file_name);
        write(&file, &system, dry_run)
    } else {
        remove(&file, dry_run)
    };
    Some(step(file.display().to_string(), outcome))
}

fn step(location: String, outcome: Outcome) -> Step {
    Step {
        kind: Kind::MenuEntry,
        target: "app menu".to_owned(),
        location,
        outcome,
    }
}

fn write(file: &Path, system: &Path, dry_run: bool) -> Outcome {
    if file.exists() || system.exists() {
        return Outcome::Skipped("an entry already exists".to_owned());
    }
    let exe = match websign_registration::host_binary() {
        Ok(exe) => exe,
        Err(error) => return Outcome::Failed(error.to_string()),
    };
    if dry_run {
        return Outcome::DryRun;
    }
    let Some(text) = contents(&exe) else {
        return Outcome::Failed("the app's path cannot be written in a menu entry".to_owned());
    };
    let result = file
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(file, text));
    match result {
        Ok(()) => Outcome::Written,
        Err(error) => Outcome::Failed(format!("{:?}", error.kind())),
    }
}

fn remove(file: &Path, dry_run: bool) -> Outcome {
    let ours = std::fs::read_to_string(file).is_ok_and(|text| text.contains(MARK));
    if !ours {
        return Outcome::NotPresent;
    }
    if dry_run {
        return Outcome::DryRun;
    }
    match std::fs::remove_file(file) {
        Ok(()) => Outcome::Removed,
        Err(error) => Outcome::Failed(format!("{:?}", error.kind())),
    }
}

/// The entry, or `None` for a path no entry can hold (not UTF-8, or with
/// control characters, which would end the line).
pub fn contents(exe: &Path) -> Option<String> {
    let exe = exec_argument(exe.to_str()?)?;
    Some(format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nExec={exe}\nIcon={}\nTerminal=false\nCategories=Utility;Security;\n{MARK}\n",
        websign_project::PRODUCT_NAME,
        websign_project::TAGLINE,
        websign_project::SLUG,
    ))
}

/// `path` as a quoted `Exec` argument (Desktop Entry spec, "The Exec
/// key"): inside the quotes `"`, `` ` ``, `$` and `\` take a backslash;
/// then the string-value escaping doubles every backslash again; `%`
/// (field codes) is written `%%`.
fn exec_argument(path: &str) -> Option<String> {
    if path.chars().any(char::is_control) {
        return None;
    }
    let mut quoted = String::with_capacity(path.len() + 2);
    for c in path.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    Some(format!(
        "\"{}\"",
        quoted.replace('\\', "\\\\").replace('%', "%%")
    ))
}

/// `$XDG_DATA_HOME/applications`, by default `~/.local/share/applications`.
fn applications_dir() -> Option<PathBuf> {
    let absolute = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    absolute("XDG_DATA_HOME")
        .or_else(|| absolute("HOME").map(|home| home.join(".local/share")))
        .map(|dir| dir.join("applications"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_writes_a_marked_entry_once_and_uninstall_removes_only_ours() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("websign.desktop");
        let no_system = dir.path().join("none.desktop");
        assert_eq!(write(&file, &no_system, true), Outcome::DryRun);
        assert_eq!(write(&file, &no_system, false), Outcome::Written);
        assert!(std::fs::read_to_string(&file).unwrap().contains(MARK));
        assert!(matches!(
            write(&file, &no_system, false),
            Outcome::Skipped(_)
        ));
        assert_eq!(remove(&file, false), Outcome::Removed);
        std::fs::write(&file, "[Desktop Entry]\nName=packaged\n").unwrap();
        assert_eq!(remove(&file, false), Outcome::NotPresent);
        assert!(file.exists());
    }

    #[test]
    fn exec_is_quoted() {
        let text = contents(Path::new("/opt/my apps/websign")).unwrap();
        assert!(text.contains("Exec=\"/opt/my apps/websign\"\n"), "{text}");
        assert_eq!(
            exec_argument(r#"/o/100% "a" $b `c` \d"#).unwrap(),
            r#""/o/100%% \\"a\\" \\$b \\`c\\` \\\\d""#
        );
        assert_eq!(exec_argument("/o/a\nb"), None);
    }
}
