//! Finding the `websign` executable.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use websign_project::{PRODUCT_NAME, SLUG};

/// Name the caller can set to bypass the search (tests, unusual installs).
const OVERRIDE_VARIABLE: &str = "WEBSIGN_EXECUTABLE";

/// The app executable: `WEBSIGN_EXECUTABLE` when set, else `websign` on
/// `PATH`, else the install locations of `docs/architecture/
/// packaging-and-release.md` §Install locations for this OS. `None` when
/// none exists.
///
/// ```
/// if websign_client::find_executable().is_none() {
///     println!("Install WebeSign first.");
/// }
/// ```
pub fn find_executable() -> Option<PathBuf> {
    let inputs = SearchInputs {
        override_path: env::var_os(OVERRIDE_VARIABLE),
        path: env::var_os("PATH"),
        home: env::home_dir(),
        local_app_data: env::var_os("LOCALAPPDATA").map(PathBuf::from),
    };
    search(&inputs, Platform::current())
}

/// Everything the search reads from the environment, so it can be tested
/// without touching the process's own variables.
#[derive(Debug, Default)]
pub(crate) struct SearchInputs {
    pub override_path: Option<OsString>,
    pub path: Option<OsString>,
    pub home: Option<PathBuf>,
    pub local_app_data: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    pub(crate) fn current() -> Platform {
        if cfg!(windows) {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }

    fn executable_name(self) -> String {
        match self {
            Platform::Windows => format!("{SLUG}.exe"),
            Platform::MacOs | Platform::Linux => SLUG.to_owned(),
        }
    }
}

pub(crate) fn search(inputs: &SearchInputs, os: Platform) -> Option<PathBuf> {
    let overridden = inputs
        .override_path
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| exists(path, os));
    overridden
        .or_else(|| on_path(inputs.path.as_ref(), os))
        .or_else(|| {
            install_locations(inputs, os)
                .into_iter()
                .find(|path| exists(path, os))
        })
}

fn on_path(path: Option<&OsString>, os: Platform) -> Option<PathBuf> {
    let name = os.executable_name();
    env::split_paths(path?)
        // A relative or empty entry means "the current directory": a
        // program must not be found (and run) from wherever the caller
        // happens to have been started.
        .filter(|dir| dir.is_absolute())
        .map(|dir| dir.join(&name))
        .find(|candidate| exists(candidate, os))
}

/// The documented install locations, in search order.
pub(crate) fn install_locations(inputs: &SearchInputs, os: Platform) -> Vec<PathBuf> {
    let home = inputs.home.as_deref();
    let name = os.executable_name();
    let mut found = Vec::new();
    match os {
        Platform::Windows => {
            if let Some(base) = inputs.local_app_data.as_deref() {
                found.push(base.join("Programs").join(PRODUCT_NAME).join(&name));
                // The Microsoft Store alias.
                found.push(base.join("Microsoft").join("WindowsApps").join(&name));
            }
        }
        Platform::MacOs => {
            let bundle = format!("{PRODUCT_NAME}.app");
            let inside = |base: &Path| base.join(&bundle).join("Contents/MacOS").join(&name);
            found.push(inside(Path::new("/Applications")));
            if let Some(home) = home {
                found.push(inside(&home.join("Applications")));
                found.push(home.join(".local/bin").join(&name));
            }
        }
        Platform::Linux => {
            found.push(Path::new("/usr/bin").join(&name));
            if let Some(home) = home {
                found.push(home.join(".local/bin").join(&name));
            }
        }
    }
    found
}

/// Whether `path` names something that can be run.
///
/// On Windows the Store alias is a reparse point that `is_file` reports as
/// missing; looking at the link itself (not its target) still finds it.
fn exists(path: &Path, os: Platform) -> bool {
    match os {
        Platform::Windows => fs::symlink_metadata(path).is_ok_and(|meta| !meta.is_dir()),
        Platform::MacOs | Platform::Linux => path.is_file(),
    }
}

#[cfg(test)]
mod tests;
