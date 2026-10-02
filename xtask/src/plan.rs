//! A generation plan: the exact files the generators want on disk.
//!
//! `gen` writes the plan; `check generated` compares it with the tree. One
//! description serves both, so the check can never disagree with the writer.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::fsutil::{list_dir, read_text, same_text, write_text};

/// One generated file, relative to the repository root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Files to produce, and the folders that hold nothing but generated files
/// (so a type removed from the protocol also disappears from disk).
#[derive(Debug, Default)]
pub struct Plan {
    pub files: Vec<GeneratedFile>,
    pub owned_dirs: Vec<PathBuf>,
    /// Files inside `owned_dirs` that another generator writes: never stale,
    /// so `gen --only ts` keeps the `project.ts` it does not produce.
    pub foreign: Vec<PathBuf>,
}

impl Plan {
    pub fn add(&mut self, path: impl Into<PathBuf>, content: String) {
        self.files.push(GeneratedFile {
            path: path.into(),
            content,
        });
    }

    pub fn merge(&mut self, other: Plan) {
        self.files.extend(other.files);
        self.owned_dirs.extend(other.owned_dirs);
        self.foreign.extend(other.foreign);
    }

    /// Writes every file and removes stale files from owned folders.
    pub fn apply(&self, root: &Path) -> Result<(), String> {
        for file in &self.files {
            write_text(&root.join(&file.path), &file.content)?;
        }
        for stale in self.stale_files(root)? {
            std::fs::remove_file(root.join(&stale))
                .map_err(|e| format!("cannot remove stale {}: {e}", stale.display()))?;
        }
        Ok(())
    }

    /// Human-readable differences between the plan and the tree; empty when
    /// the committed files are current.
    pub fn differences(&self, root: &Path) -> Result<Vec<String>, String> {
        let mut problems = Vec::new();
        for file in &self.files {
            match read_text(&root.join(&file.path)) {
                Err(_) => problems.push(format!("{}: missing", file.path.display())),
                Ok(current) if !same_text(&current, &file.content) => {
                    problems.push(format!("{}: out of date", file.path.display()));
                }
                Ok(_) => {}
            }
        }
        for stale in self.stale_files(root)? {
            problems.push(format!("{}: no longer generated", stale.display()));
        }
        Ok(problems)
    }

    /// Files inside owned folders that no generator produces any more.
    fn stale_files(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let planned: BTreeSet<&Path> = self
            .files
            .iter()
            .map(|f| f.path.as_path())
            .chain(self.foreign.iter().map(PathBuf::as_path))
            .collect();
        let mut stale = Vec::new();
        for dir in &self.owned_dirs {
            collect_stale(root, dir, &planned, &mut stale)?;
        }
        Ok(stale)
    }
}

fn collect_stale(
    root: &Path,
    dir: &Path,
    planned: &BTreeSet<&Path>,
    stale: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !root.join(dir).is_dir() {
        return Ok(());
    }
    for (name, is_dir) in list_dir(&root.join(dir))? {
        // `/` like the planned paths, so reports read the same on Windows
        // (where `Path::join` would insert `\`); lookups compare components.
        let relative = PathBuf::from(format!("{}/{name}", dir.display()));
        if is_dir {
            collect_stale(root, &relative, planned, stale)?;
        } else if !planned.contains(relative.as_path()) {
            stale.push(relative);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    fn plan_with(path: &str, content: &str) -> Plan {
        let mut plan = Plan {
            owned_dirs: vec![PathBuf::from("out")],
            ..Plan::default()
        };
        plan.add(path, content.to_owned());
        plan
    }

    #[test]
    fn apply_writes_files_and_removes_stale_ones() {
        let root = TempDir::new();
        root.write("out/old.ts", "old");
        plan_with("out/new.ts", "new").apply(root.path()).unwrap();
        assert!(root.path().join("out/new.ts").is_file());
        assert!(!root.path().join("out/old.ts").exists());
    }

    #[test]
    fn differences_report_missing_outdated_and_stale() {
        let root = TempDir::new();
        root.write("out/stale.ts", "x");
        root.write("out/changed.ts", "before");
        let mut plan = plan_with("out/changed.ts", "after");
        plan.add("out/missing.ts", String::new());
        let mut found = plan.differences(root.path()).unwrap();
        found.sort();
        assert_eq!(
            found,
            [
                "out/changed.ts: out of date",
                "out/missing.ts: missing",
                "out/stale.ts: no longer generated"
            ]
        );
    }

    #[test]
    fn foreign_files_in_owned_folders_are_kept() {
        let root = TempDir::new();
        root.write("out/project.ts", "p");
        let mut plan = plan_with("out/a.ts", "a");
        plan.foreign.push(PathBuf::from("out/project.ts"));
        plan.apply(root.path()).unwrap();
        assert!(root.path().join("out/project.ts").is_file());
    }

    #[test]
    fn a_current_tree_has_no_differences() {
        let root = TempDir::new();
        let plan = plan_with("out/a.ts", "a");
        plan.apply(root.path()).unwrap();
        assert!(plan.differences(root.path()).unwrap().is_empty());
    }
}
