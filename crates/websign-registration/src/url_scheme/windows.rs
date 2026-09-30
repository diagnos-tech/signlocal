//! `HKCU\Software\Classes\<scheme>`: the per-user URL protocol handler.

use std::path::Path;

use websign_project::{PRODUCT_NAME, URL_SCHEME};

use crate::destination::Outcome;
use crate::registry::{Hive, Registry};

fn class_key() -> String {
    format!(r"Software\Classes\{URL_SCHEME}")
}

pub fn register(registry: &dyn Registry, executable: &Path, dry_run: bool) -> Outcome {
    if dry_run {
        return Outcome::DryRun;
    }
    let key = class_key();
    let exe = executable.to_string_lossy();
    let values = [
        (key.clone(), "", format!("URL:{PRODUCT_NAME}")),
        (key.clone(), "URL Protocol", String::new()),
        (format!(r"{key}\DefaultIcon"), "", format!("\"{exe}\",0")),
        (
            format!(r"{key}\shell\open\command"),
            "",
            format!("\"{exe}\" \"%1\""),
        ),
    ];
    for (subkey, name, value) in values {
        if let Err(error) = registry.set_string(&subkey, name, &value) {
            return Outcome::Failed(format!("cannot write HKCU\\{subkey}: {error}"));
        }
    }
    Outcome::Written
}

pub fn unregister(registry: &dyn Registry, dry_run: bool) -> Outcome {
    let key = class_key();
    if !registry.key_exists(Hive::CurrentUser, &key) {
        return Outcome::NotPresent;
    }
    if dry_run {
        return Outcome::DryRun;
    }
    match registry.remove_tree(&key) {
        Ok(()) => Outcome::Removed,
        Err(error) => Outcome::Failed(format!("cannot remove HKCU\\{key}: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemoryRegistry;

    #[test]
    fn register_writes_the_protocol_keys_and_unregister_is_idempotent() {
        let registry = MemoryRegistry::new();
        let exe = Path::new(r"C:\Programs\WebeSign\websign.exe");
        assert_eq!(register(&registry, exe, false), Outcome::Written);
        let key = class_key();
        let read = |subkey: &str, name: &str| {
            registry
                .get_string(Hive::CurrentUser, subkey, name)
                .unwrap()
        };
        assert_eq!(read(&key, "URL Protocol").as_deref(), Some(""));
        assert_eq!(
            read(&format!(r"{key}\shell\open\command"), "").as_deref(),
            Some(r#""C:\Programs\WebeSign\websign.exe" "%1""#)
        );

        assert_eq!(unregister(&registry, true), Outcome::DryRun);
        assert_eq!(unregister(&registry, false), Outcome::Removed);
        assert!(
            registry
                .current_user_keys()
                .iter()
                .all(|k| !k.contains("classes\\"))
        );
        assert_eq!(unregister(&registry, false), Outcome::NotPresent);
    }

    #[test]
    fn dry_run_writes_nothing() {
        let registry = MemoryRegistry::new();
        assert_eq!(
            register(&registry, Path::new(r"C:\w.exe"), true),
            Outcome::DryRun
        );
        assert!(registry.current_user_keys().is_empty());
    }
}
