//! Loading a PKCS#11 module once per process.
//!
//! A vendor library is loaded and initialized the first time it is needed and
//! then kept for the life of the process: initialization can take a second
//! (it talks to the smart card service), and unloading a library that
//! started threads is a classic way to crash a process. `C_Finalize` is
//! never called for the same reason, and because other code in the process
//! (p11-kit's proxy, for one) may be using the same library.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use cryptoki::context::{CInitializeArgs, CInitializeFlags, Pkcs11};
use cryptoki::error::{Error, RvError};

use super::ckr;
use super::file_id::FileId;

static LOADED: Mutex<Vec<(FileId, Pkcs11)>> = Mutex::new(Vec::new());

/// The initialized module at `path`, loading it if this is the first request.
///
/// The error text says what went wrong in terms a person can act on.
pub fn load(path: &Path) -> Result<Pkcs11, String> {
    let identity = FileId::of(path);
    let mut loaded = LOADED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(known) = identity
        .as_ref()
        .and_then(|id| loaded.iter().find(|(known, _)| known == id))
    {
        return Ok(known.1.clone());
    }

    let pkcs11 = Pkcs11::new(path).map_err(|error| load_error(path, &error.to_string()))?;
    match pkcs11.initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK)) {
        // Another user of the same library (p11-kit's proxy, a middleware
        // component) initialized it first; it is ready to use.
        Ok(()) | Err(Error::Pkcs11(RvError::CryptokiAlreadyInitialized, _)) => {}
        Err(error) => {
            // Never unload a library that failed half-way through startup.
            std::mem::forget(pkcs11);
            return Err(initialize_error(path, &error));
        }
    }
    if let Some(identity) = identity {
        loaded.push((identity, pkcs11.clone()));
    }
    Ok(pkcs11)
}

fn load_error(path: &Path, reason: &str) -> String {
    let lowercase = reason.to_lowercase();
    let wrong_architecture = [
        "wrong elf class",
        "os error 193",
        "not a valid win32",
        "incompatible architecture",
        "mach-o",
    ]
    .iter()
    .any(|marker| lowercase.contains(marker));
    let hint = if wrong_architecture {
        " (the module was built for a different architecture than this program)"
    } else {
        ""
    };
    format!("cannot load {}: {reason}{hint}", path.display())
}

fn initialize_error(path: &Path, error: &Error) -> String {
    let reason = match error {
        Error::Pkcs11(rv, _) => {
            let (code, name) = ckr::describe(*rv);
            format!("{name} ({code:#x})")
        }
        other => other.to_string(),
    };
    format!(
        "{} loaded but C_Initialize failed: {reason}",
        path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_not_a_library_reports_the_path_and_the_reason() {
        let error = load(Path::new("/nonexistent/libnothing.so")).unwrap_err();
        assert!(
            error.starts_with("cannot load /nonexistent/libnothing.so: "),
            "{error}"
        );
    }

    #[test]
    fn wrong_architecture_gets_a_hint() {
        for reason in [
            "/x.so: wrong ELF class: ELFCLASS32",
            "LoadLibraryExW failed (os error 193)",
            "mach-o, but wrong architecture",
        ] {
            assert!(
                load_error(Path::new("/x.so"), reason).contains("different architecture"),
                "{reason}"
            );
        }
        assert!(!load_error(Path::new("/x.so"), "no such file").contains("architecture"));
    }

    #[test]
    fn initialize_failures_show_the_return_code() {
        let error = Error::Pkcs11(
            RvError::GeneralError,
            cryptoki::context::Function::Initialize,
        );
        assert_eq!(
            initialize_error(Path::new("/x.so"), &error),
            "/x.so loaded but C_Initialize failed: CKR_GENERAL_ERROR (0x5)"
        );
    }
}
