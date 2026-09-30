//! `codesign` for the macOS bundle and what it nests, always inside out:
//! `--deep` would re-sign nested code without its entitlements and strip the
//! Safari appex's sandbox.

use std::process::Command;

use super::Context;
use super::tool::run;

/// Signing identity. Ad hoc (`-`) gives the code an identity for the URL
/// scheme, the keychain and Safari's "Allow Unsigned Extensions", but is not
/// notarization (unsigned-build notice, docs/install.md).
/// TODO(gustavo): Developer ID identity, `--options runtime --timestamp`,
/// then notarization.
const IDENTITY: &str = "-";

/// Signs `path` (a binary or a bundle), with
/// `packaging/macos/<entitlements>` when given.
pub fn sign(
    context: &Context,
    path: &std::path::Path,
    entitlements: Option<&str>,
) -> Result<(), String> {
    let mut command = Command::new("codesign");
    command.args(["--force", "--sign", IDENTITY]);
    if let Some(file) = entitlements {
        command
            .arg("--entitlements")
            .arg(context.root.join("packaging/macos").join(file));
    }
    run(
        command.arg(path),
        "codesign comes with the Xcode command line tools",
    )
}
