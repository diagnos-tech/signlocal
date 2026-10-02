//! The files the repository owns, as git sees them.
//!
//! Asking git makes `.gitignore` the single definition of "ignored", so the
//! checker never disagrees with what a commit would contain.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

/// The frozen Phase-0 kit follows the format but is not checked.
const UNCHECKED_PREFIX: &str = "docs/prototypes/kit";
/// Tool folders that are never part of the tree, even if not gitignored.
const SKIPPED_NAMES: [&str; 6] = [".git", "node_modules", "target", "dist", ".output", ".wxt"];

/// Folder (repository-relative, `""` is the root) → its entries; a folder
/// entry ends with `/`.
pub type Tree = BTreeMap<String, BTreeSet<String>>;

/// Tracked and untracked-but-not-ignored files, grouped by folder.
pub fn tree(root: &Path) -> Result<Tree, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .output()
        .map_err(|e| format!("cannot run git to list the repository files: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let listing = String::from_utf8_lossy(&output.stdout);
    let files = listing.split('\0').filter(|path| !path.is_empty());
    // `--cached` also lists files deleted from the working tree.
    Ok(group(
        files.filter(|path| root.join(path).symlink_metadata().is_ok()),
    ))
}

/// Groups repository-relative file paths into folders and entries.
pub fn group<'a>(files: impl Iterator<Item = &'a str>) -> Tree {
    let mut tree = Tree::new();
    for file in files {
        let parts: Vec<&str> = file.split('/').collect();
        if parts.iter().any(|part| SKIPPED_NAMES.contains(part)) {
            continue;
        }
        let mut folder = String::new();
        for (index, part) in parts.iter().enumerate() {
            let last = index + 1 == parts.len();
            let entry = if last {
                (*part).to_owned()
            } else {
                format!("{part}/")
            };
            tree.entry(folder.clone()).or_default().insert(entry);
            if !last {
                folder = if folder.is_empty() {
                    (*part).to_owned()
                } else {
                    format!("{folder}/{part}")
                };
            }
        }
    }
    tree
}

/// Whether `folder` is exempt from the `SUMMARY.md` rules: the frozen kit,
/// `generated/` folders, whose summary the generator writes, and the TypeDoc
/// output under `site/api/`, which TypeDoc rewrites from scratch (its parent
/// `site/SUMMARY.md` still lists `api/`).
pub fn is_unchecked(folder: &str) -> bool {
    let kit = folder == UNCHECKED_PREFIX || folder.starts_with("docs/prototypes/kit/");
    let api_docs = folder == "site/api" || folder.starts_with("site/api/");
    kit || api_docs || folder.rsplit('/').next() == Some("generated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_files_into_folders_with_slash_marked_subfolders() {
        let tree = group(["README.md", "a/x.rs", "a/b/y.rs"].into_iter());
        assert_eq!(
            tree[""],
            BTreeSet::from(["README.md".to_owned(), "a/".to_owned()])
        );
        assert_eq!(
            tree["a"],
            BTreeSet::from(["x.rs".to_owned(), "b/".to_owned()])
        );
        assert_eq!(tree["a/b"], BTreeSet::from(["y.rs".to_owned()]));
    }

    #[test]
    fn skips_tool_folders() {
        let tree = group(["a/node_modules/x.js", "a/ok.rs"].into_iter());
        assert_eq!(tree["a"], BTreeSet::from(["ok.rs".to_owned()]));
    }

    #[test]
    fn the_kit_and_generated_folders_are_unchecked() {
        assert!(is_unchecked("docs/prototypes/kit/app"));
        assert!(!is_unchecked("docs/prototypes/kitchen"));
        assert!(is_unchecked("sdk/src/generated"));
        assert!(is_unchecked("site/api/functions"));
        assert!(!is_unchecked("site/apis"));
        assert!(!is_unchecked("sdk/src"));
    }
}
