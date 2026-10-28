//! Where token vendors install their PKCS#11 module, per operating system.
//!
//! Most of these modules register nowhere (no p11-kit file, no OS driver), so
//! the only way to find them is to know the path. Each entry is one product;
//! its paths are alternative install locations, and only the first one that
//! exists is loaded. The source of every path is in
//! `docs/research/pkcs11-modules.md`.
//!
//! A path may hold `%VAR%` (Windows environment variables) or a single `*`
//! inside one directory name (vendors that put the install date in it).

mod linux;
mod macos;
mod windows;

/// One product's module and where it may be installed.
#[derive(Debug, Clone, Copy)]
pub struct KnownModule {
    pub vendor: &'static str,
    pub product: &'static str,
    pub paths: &'static [&'static str],
}

/// Shorthand that keeps one product per line in the tables.
const fn module(
    vendor: &'static str,
    product: &'static str,
    paths: &'static [&'static str],
) -> KnownModule {
    KnownModule {
        vendor,
        product,
        paths,
    }
}

/// The list for the operating system this binary runs on.
pub fn for_this_os() -> &'static [KnownModule] {
    if cfg!(windows) {
        windows::MODULES
    } else if cfg!(target_os = "macos") {
        macos::MODULES
    } else {
        linux::MODULES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTS: [(&str, &[KnownModule]); 3] = [
        ("linux", linux::MODULES),
        ("macos", macos::MODULES),
        ("windows", windows::MODULES),
    ];

    fn all() -> impl Iterator<Item = (&'static str, &'static KnownModule)> {
        LISTS
            .into_iter()
            .flat_map(|(os, list)| list.iter().map(move |entry| (os, entry)))
    }

    #[test]
    fn every_entry_has_a_name_and_at_least_one_path() {
        for (os, entry) in all() {
            assert!(
                !entry.vendor.is_empty() && !entry.product.is_empty(),
                "{os}: {entry:?}"
            );
            assert!(
                !entry.paths.is_empty(),
                "{os}: {} has no paths",
                entry.product
            );
        }
    }

    #[test]
    fn paths_are_absolute_for_their_operating_system() {
        for (os, entry) in all() {
            for path in entry.paths {
                let absolute = match os {
                    "windows" => path.starts_with('%') || path.as_bytes().get(1) == Some(&b':'),
                    _ => path.starts_with('/'),
                };
                assert!(absolute, "{os}: {path} is not absolute");
            }
        }
    }

    #[test]
    fn no_path_is_listed_twice_per_operating_system() {
        for (os, list) in LISTS {
            let mut seen = std::collections::HashSet::new();
            for path in list.iter().flat_map(|entry| entry.paths) {
                assert!(seen.insert(*path), "{os}: {path} appears twice");
            }
        }
    }

    #[test]
    fn a_path_has_at_most_one_wildcard() {
        for (os, entry) in all() {
            for path in entry.paths {
                assert!(path.matches('*').count() <= 1, "{os}: {path}");
            }
        }
    }

    #[test]
    fn the_test_token_module_is_always_known() {
        for (os, list) in LISTS {
            assert!(
                list.iter().any(|entry| entry.product == "SoftHSM2"),
                "{os} lacks SoftHSM2"
            );
        }
    }
}
