//! The desktop's "open file" dialog through its command-line front end:
//! `kdialog` on KDE, `zenity` elsewhere (GNOME, Xfce, Cinnamon, MATE), the
//! other one as a fallback.
//!
//! The XDG portal's `FileChooser` would also reach sandboxed (Flatpak)
//! builds, but it answers with a D-Bus signal that `gdbus` cannot wait for
//! in one call; without a helper the window falls back to a path field.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::platform::file_picker::{FileRequest, Picked};

/// A dialog program and the arguments that ask it for one file.
#[derive(Debug, PartialEq, Eq)]
struct Invocation {
    program: &'static str,
    args: Vec<String>,
}

pub fn choose_file(
    request: FileRequest,
    _parent: Option<isize>,
    done: impl FnOnce(Picked) + Send + 'static,
) {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let invocations = invocations(&request, &desktop);
    let worker = std::thread::Builder::new()
        .name("file-picker".into())
        .spawn(move || done(run_first(&invocations)));
    if let Err(error) = worker {
        log::warn!("file picker: could not start: {error}");
    }
}

/// Runs the first dialog that exists; waits for the person as long as
/// they take.
fn run_first(invocations: &[Invocation]) -> Picked {
    for invocation in invocations {
        let output = Command::new(invocation.program)
            .args(&invocation.args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        let Ok(output) = output else {
            continue;
        };
        match output.status.code() {
            Some(0) => return chosen(&output.stdout),
            // Both programs exit with 1 when the person cancels.
            Some(1) => return Picked::Cancelled,
            // No display, broken install: try the other one.
            _ => log::info!("file picker: {} failed", invocation.program),
        }
    }
    Picked::Unavailable
}

fn chosen(stdout: &[u8]) -> Picked {
    use std::os::unix::ffi::OsStrExt;
    let line = stdout.strip_suffix(b"\n").unwrap_or(stdout);
    if line.is_empty() {
        return Picked::Cancelled;
    }
    Picked::Chosen(PathBuf::from(std::ffi::OsStr::from_bytes(line)))
}

/// The dialogs to try, the desktop's own first.
fn invocations(request: &FileRequest, desktop: &str) -> Vec<Invocation> {
    let patterns: Vec<String> = request
        .extensions
        .iter()
        .flat_map(|extension| {
            let exact = format!("*.{extension}");
            // Shared objects are often versioned: `libeTPkcs11.so.10`.
            let versioned = (*extension == "so").then(|| format!("*.{extension}.*"));
            std::iter::once(exact).chain(versioned)
        })
        .collect();
    let patterns = patterns.join(" ");
    let zenity = Invocation {
        program: "zenity",
        args: vec![
            "--file-selection".into(),
            format!("--title={}", request.title),
            format!("--file-filter={} | {patterns}", request.type_name),
        ],
    };
    let kdialog = Invocation {
        program: "kdialog",
        args: vec![
            "--title".into(),
            request.title.clone(),
            "--getopenfilename".into(),
            std::env::var("HOME").unwrap_or_else(|_| "/".into()),
            format!("{patterns}|{}", request.type_name),
        ],
    };
    if desktop
        .split(':')
        .any(|name| name.eq_ignore_ascii_case("KDE"))
    {
        vec![kdialog, zenity]
    } else {
        vec![zenity, kdialog]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> FileRequest {
        FileRequest {
            title: "Choose the token driver".into(),
            type_name: "Token driver".into(),
            extensions: vec!["so"],
        }
    }

    #[test]
    fn zenity_filters_shared_objects_including_versioned_ones() {
        let first = &invocations(&request(), "GNOME")[0];
        assert_eq!(first.program, "zenity");
        assert!(
            first
                .args
                .contains(&"--file-filter=Token driver | *.so *.so.*".to_owned())
        );
    }

    #[test]
    fn kde_gets_kdialog_first() {
        let all = invocations(&request(), "ubuntu:KDE");
        assert_eq!(all[0].program, "kdialog");
        assert_eq!(
            all[0].args.last().map(String::as_str),
            Some("*.so *.so.*|Token driver")
        );
        assert_eq!(all[1].program, "zenity");
    }

    #[test]
    fn missing_programs_mean_no_dialog() {
        let none = [Invocation {
            program: "websign-no-such-dialog",
            args: Vec::new(),
        }];
        assert_eq!(run_first(&none), Picked::Unavailable);
    }

    #[test]
    fn the_printed_path_is_the_choice() {
        assert_eq!(
            chosen(b"/usr/lib/libeTPkcs11.so\n"),
            Picked::Chosen(PathBuf::from("/usr/lib/libeTPkcs11.so"))
        );
        assert_eq!(chosen(b"\n"), Picked::Cancelled);
    }
}
