//! The product name of a Linux program: the `Name` of an installed
//! `.desktop` application entry whose `Exec` starts that program
//! ([Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)).
//!
//! The name is self-declared like any product name; the window pairs it with
//! the executable path and the "Unverified program" warning.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Entries larger than this are not application launchers.
const MAX_ENTRY_BYTES: u64 = 64 * 1024;

/// The `Name` of the first application entry, in XDG data-directory order,
/// whose `Exec` program resolves to `executable`.
pub fn product_name(executable: &Path) -> Option<String> {
    let search = std::env::var_os("PATH").unwrap_or_default();
    application_dirs()
        .iter()
        .find_map(|dir| name_in(dir, executable, &search))
}

/// `$XDG_DATA_HOME/applications`, then each `$XDG_DATA_DIRS/applications`,
/// with the specification's defaults.
fn application_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("XDG_DATA_HOME")
        .filter(|dir| Path::new(dir).is_absolute())
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".local/share")));
    let shared = std::env::var_os("XDG_DATA_DIRS")
        .filter(|dirs| !dirs.is_empty())
        .unwrap_or_else(|| OsString::from("/usr/local/share:/usr/share"));
    home.into_iter()
        .chain(std::env::split_paths(&shared).filter(|dir| dir.is_absolute()))
        .map(|dir| dir.join("applications"))
        .collect()
}

fn name_in(dir: &Path, executable: &Path, search: &OsString) -> Option<String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "desktop"))
        .collect();
    // Directory order is arbitrary; sorted, the same entry wins every time.
    files.sort();
    files.iter().find_map(|file| {
        let entry = parse(&read_small(file)?)?;
        let program = resolve(&entry.program, search)?;
        (program == executable).then_some(entry.name)
    })
}

fn read_small(file: &Path) -> Option<String> {
    let size = std::fs::metadata(file).ok()?.len();
    (size <= MAX_ENTRY_BYTES)
        .then(|| std::fs::read_to_string(file).ok())
        .flatten()
}

/// What this module reads from one entry.
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    name: String,
    /// The first word of `Exec`, unquoted.
    program: String,
}

/// The `Name` and `Exec` program of an application entry; `None` for other
/// types, hidden entries, or entries missing either key.
fn parse(text: &str) -> Option<Entry> {
    let (mut name, mut exec, mut application) = (None, None, false);
    let mut in_main_group = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_main_group = line == "[Desktop Entry]";
            continue;
        }
        let Some((key, value)) = line.split_once('=').filter(|_| in_main_group) else {
            continue;
        };
        match key.trim_end() {
            "Name" => name = Some(value.trim_start().to_owned()),
            "Exec" => exec = Some(value.trim_start()),
            "Type" => application = value.trim() == "Application",
            "Hidden" if value.trim() == "true" => return None,
            _ => {}
        }
    }
    let name = name.filter(|name| !name.is_empty())?;
    let program = first_program(exec?)?;
    application.then_some(Entry { name, program })
}

/// The program `Exec` starts: its first word, unquoted, past an `env`
/// prefix with its `VAR=value` assignments.
fn first_program(exec: &str) -> Option<String> {
    let mut words = words(exec).into_iter();
    let mut word = words.next()?;
    if word == "env" || word == "/usr/bin/env" {
        word = words.find(|word| !word.contains('='))?;
    }
    Some(word)
}

/// `Exec` split into words: whitespace separates, and a double-quoted word
/// keeps its spaces with `\"`, `\\`, `` \` `` and `\$` unescaped.
fn words(exec: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut chars = exec.chars().peekable();
    while let Some(&first) = chars.peek() {
        if first.is_whitespace() {
            chars.next();
            continue;
        }
        let mut word = String::new();
        if first == '"' {
            chars.next();
            while let Some(c) = chars.next() {
                match c {
                    '"' => break,
                    '\\' => word.extend(chars.next()),
                    c => word.push(c),
                }
            }
        } else {
            while let Some(c) = chars.next_if(|c| !c.is_whitespace()) {
                word.push(c);
            }
        }
        words.push(word);
    }
    words
}

/// The real path `program` names: itself when absolute, else the first
/// match in `search` (`PATH`), with symlinks resolved like `/proc/*/exe`.
fn resolve(program: &str, search: &OsString) -> Option<PathBuf> {
    let program = Path::new(program);
    if program.is_absolute() {
        return std::fs::canonicalize(program).ok();
    }
    if program.components().count() != 1 {
        return None;
    }
    std::env::split_paths(search)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
        .and_then(|found| std::fs::canonicalize(found).ok())
}

#[cfg(test)]
#[path = "desktop_entry_tests.rs"]
mod tests;
