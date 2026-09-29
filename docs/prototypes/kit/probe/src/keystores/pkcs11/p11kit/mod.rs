//! Modules registered with p11-kit: the `*.module` files that packages drop
//! in a well-known directory so every PKCS#11 client finds the same drivers.
//!
//! A file looks like
//!
//! ```text
//! module: opensc-pkcs11.so
//! disable-in: p11-kit-proxy
//! ```
//!
//! Only `module:`, `enable-in:`, `disable-in:`, `trust-policy:` and `remote:`
//! matter here; every other key is ignored.

mod module_file;

use std::path::{Path, PathBuf};

use module_file::ModuleFile;

/// Directories with `.module` files, lowest precedence first: a file in a
/// later directory replaces a same-named file in an earlier one.
pub fn config_dirs() -> Vec<PathBuf> {
    // p11-kit has no Windows port to register modules with.
    if cfg!(windows) {
        return Vec::new();
    }
    let mut dirs = vec![
        PathBuf::from("/usr/share/p11-kit/modules"),
        PathBuf::from("/etc/pkcs11/modules"),
    ];
    if cfg!(target_os = "macos") {
        dirs.insert(0, PathBuf::from("/opt/homebrew/share/p11-kit/modules"));
        dirs.insert(1, PathBuf::from("/usr/local/share/p11-kit/modules"));
        dirs.push(PathBuf::from("/opt/homebrew/etc/pkcs11/modules"));
        dirs.push(PathBuf::from("/usr/local/etc/pkcs11/modules"));
    }
    let user = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".config")));
    dirs.extend(user.map(|dir| dir.join("pkcs11/modules")));
    dirs
}

/// Where a relative `module:` is looked up, best guess first. p11-kit uses
/// the directory compiled into it, which is `$libdir/pkcs11` on every distro
/// but under a different `$libdir` on each.
pub fn module_dirs() -> Vec<PathBuf> {
    let multiarch = match std::env::consts::ARCH {
        "x86_64" => Some("x86_64-linux-gnu"),
        "aarch64" => Some("aarch64-linux-gnu"),
        "arm" => Some("arm-linux-gnueabihf"),
        "riscv64" => Some("riscv64-linux-gnu"),
        _ => None,
    };
    let mut dirs: Vec<PathBuf> = multiarch
        .map(|triple| PathBuf::from(format!("/usr/lib/{triple}/pkcs11")))
        .into_iter()
        .collect();
    dirs.extend(
        [
            "/usr/lib64/pkcs11",
            "/usr/lib/pkcs11",
            "/usr/local/lib/pkcs11",
        ]
        .map(PathBuf::from),
    );
    if cfg!(target_os = "macos") {
        dirs.extend(["/opt/homebrew/lib/pkcs11"].map(PathBuf::from));
    }
    dirs
}

/// Module files registered in `config_dirs`, with relative `module:` values
/// resolved against `module_dirs`, that apply to `programs` and can sign.
/// A missing library is still returned (the caller reports it).
pub fn registered_modules(
    config_dirs: &[PathBuf],
    module_dirs: &[PathBuf],
    programs: &[String],
) -> Vec<PathBuf> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for dir in config_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            let Some(name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
            else {
                continue;
            };
            if !name.ends_with(".module") {
                continue;
            }
            files.retain(|(known, _)| *known != name);
            files.push((name, path));
        }
    }
    files.sort();
    files
        .into_iter()
        .filter_map(|(_, path)| {
            let file = ModuleFile::parse(&std::fs::read_to_string(path).ok()?);
            if !file.is_signing_candidate() || !file.applies_to(programs) {
                return None;
            }
            Some(resolve(file.module.as_deref()?, module_dirs))
        })
        .collect()
}

fn resolve(module: &str, module_dirs: &[PathBuf]) -> PathBuf {
    let path = Path::new(module);
    if path.is_absolute() {
        return path.to_owned();
    }
    module_dirs
        .iter()
        .map(|dir| dir.join(path))
        .find(|candidate| candidate.exists())
        .or_else(|| module_dirs.first().map(|dir| dir.join(path)))
        .unwrap_or_else(|| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn programs(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("websign-p11kit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn later_directories_override_same_named_files() {
        let root = scratch_dir("override");
        let (system, user, libs) = (root.join("system"), root.join("user"), root.join("libs"));
        for dir in [&system, &user, &libs] {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(system.join("vendor.module"), "module: old.so\n").unwrap();
        std::fs::write(system.join("other.module"), "module: /abs/other.so\n").unwrap();
        std::fs::write(user.join("vendor.module"), "module: new.so\n").unwrap();
        std::fs::write(user.join("notes.txt"), "module: ignored.so\n").unwrap();
        std::fs::write(libs.join("new.so"), b"").unwrap();

        let found = registered_modules(
            &[system, user],
            std::slice::from_ref(&libs),
            &programs(&["websign"]),
        );
        assert_eq!(found, [PathBuf::from("/abs/other.so"), libs.join("new.so")]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn excluded_and_unreadable_registrations_are_skipped() {
        let root = scratch_dir("skip");
        std::fs::write(
            root.join("trust.module"),
            "module: p11-kit-trust.so\ntrust-policy: yes\n",
        )
        .unwrap();
        std::fs::write(
            root.join("mine.module"),
            "module: /abs/mine.so\ndisable-in: websign\n",
        )
        .unwrap();
        std::fs::write(root.join("empty.module"), "").unwrap();
        let found = registered_modules(
            &[root.clone(), root.join("does-not-exist")],
            &[],
            &programs(&["websign"]),
        );
        assert!(found.is_empty(), "{found:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn relative_modules_fall_back_to_the_first_module_dir_when_missing() {
        let dirs = [
            PathBuf::from("/nonexistent/a"),
            PathBuf::from("/nonexistent/b"),
        ];
        assert_eq!(resolve("x.so", &dirs), PathBuf::from("/nonexistent/a/x.so"));
    }
}
