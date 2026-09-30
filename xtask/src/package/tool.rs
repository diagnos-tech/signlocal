//! Running external tools (nfpm, zip, tar, lipo, codesign, cargo) with errors
//! that say what to install instead of a bare "No such file".

use std::io::ErrorKind;
use std::process::Command;

/// Runs `command` to completion; `hint` says how to get the tool.
pub fn run(command: &mut Command, hint: &str) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().into_owned();
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("`{program}` failed ({status})")),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Err(format!("`{program}` not found: {hint}"))
        }
        Err(error) => Err(format!("cannot run `{program}`: {error}")),
    }
}
