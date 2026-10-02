//! Which PKCS#11 module files to try: the ones asked for, the ones registered
//! with p11-kit and the ones at vendors' well-known install paths.
//!
//! The same library often shows up more than once (a p11-kit registration
//! that points at a vendor path, a symlink in `/usr/lib`), so candidates are
//! merged by file identity: loading one module twice would list its
//! certificates twice and initialize it twice.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::file_id::FileId;
use super::known_paths::{self, KnownModule};
use super::{p11kit, path_patterns};
use crate::Options;
use websign_project as config;

/// A module file to load.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    /// Whether a missing or unloadable file is worth telling the user about:
    /// yes when they (or a package, through p11-kit) named it, no when it is
    /// only a guess from the list of well-known paths.
    pub reportable: bool,
    /// The vendor product this path belongs to, when it came from the list of
    /// well-known paths; it makes failure messages recognizable.
    pub known: Option<&'static KnownModule>,
}

/// Every module to load, without duplicates. `options` decides which sources
/// are consulted; explicitly requested modules are always included.
pub fn discover(options: &Options) -> Vec<Candidate> {
    let mut found = Found::default();
    for path in &options.extra_modules {
        found.add(path.clone(), true, None);
    }
    if !options.no_p11_kit {
        let registered = p11kit::registered_modules(
            &p11kit::config_dirs(),
            &p11kit::module_dirs(),
            &program_names(),
        );
        for path in registered {
            found.add(path, true, None);
        }
    }
    if !options.no_known_modules {
        for module in known_paths::for_this_os() {
            if let Some(path) = installed(module) {
                found.add(path, false, Some(module));
            }
        }
    }
    found.candidates
}

/// The names p11-kit's `enable-in` / `disable-in` may know this program by.
fn program_names() -> Vec<String> {
    let mut names = vec![config::SLUG.to_owned()];
    let executable = std::env::current_exe().ok();
    let stem = executable
        .as_deref()
        .and_then(Path::file_stem)
        .and_then(|stem| stem.to_str());
    names.extend(stem.map(str::to_owned).filter(|stem| stem != config::SLUG));
    names
}

/// Candidates in the order they were added, keeping one per file.
#[derive(Default)]
struct Found {
    candidates: Vec<Candidate>,
    by_file: HashMap<FileId, usize>,
}

impl Found {
    fn add(&mut self, path: PathBuf, reportable: bool, known: Option<&'static KnownModule>) {
        let id = FileId::of(&path);
        // A file that does not exist has no identity; only its path can repeat.
        let existing = match &id {
            Some(id) => self.by_file.get(id).copied(),
            None => self
                .candidates
                .iter()
                .position(|candidate| candidate.path == path),
        };
        match existing {
            // One reportable mention is enough to keep the merged entry so.
            Some(index) => {
                let candidate = &mut self.candidates[index];
                candidate.reportable |= reportable;
                candidate.known = candidate.known.or(known);
            }
            None => {
                if let Some(id) = id {
                    self.by_file.insert(id, self.candidates.len());
                }
                self.candidates.push(Candidate {
                    path,
                    reportable,
                    known,
                });
            }
        }
    }
}

/// The first path of `module` that exists on this machine.
fn installed(module: &KnownModule) -> Option<PathBuf> {
    module
        .paths
        .iter()
        .flat_map(|template| path_patterns::expand(template))
        .find(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("websign-discovery-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[cfg(unix)]
    #[test]
    fn the_same_file_reached_twice_is_one_candidate() {
        let root = scratch_dir("dedup");
        let library = root.join("libmodule.so");
        std::fs::write(&library, b"x").unwrap();
        let link = root.join("libmodule-link.so");
        std::os::unix::fs::symlink(&library, &link).unwrap();

        let mut found = Found::default();
        found.add(library.clone(), false, None);
        found.add(link, true, None);
        found.add(root.join("missing.so"), true, None);
        found.add(root.join("missing.so"), false, None);

        assert_eq!(found.candidates.len(), 2);
        assert_eq!(found.candidates[0].path, library);
        assert!(
            found.candidates[0].reportable,
            "a reportable mention makes the merged entry reportable"
        );
        assert_eq!(found.candidates[1].path, root.join("missing.so"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn requested_modules_come_first_and_are_reportable() {
        let options = Options {
            extra_modules: vec![PathBuf::from("/nonexistent/libmine.so")],
            no_known_modules: true,
            no_p11_kit: true,
            ..Options::default()
        };
        let found = discover(&options);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, PathBuf::from("/nonexistent/libmine.so"));
        assert!(found[0].reportable && found[0].known.is_none());
    }
}
