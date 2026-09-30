//! `.deb` and `.rpm` through nfpm, run as an external tool.

use std::path::PathBuf;
use std::process::Command;

use super::formats::Format;
use super::stage::{fresh, render_template, text};
use super::target::Target;
use super::tool::run;
use super::{Context, build, linux, names};

/// The nfpm release CI installs from the pinned release tarball (a
/// `go install` build reports its version as "dev" and would only warn).
/// A different version only warns: the config uses stable keys, but the
/// packages were verified with this one.
pub const NFPM_VERSION: &str = "2.43.0";

/// Warns when the installed nfpm is not the pinned version.
fn check_version() {
    let output = Command::new("nfpm").arg("--version").output();
    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        if !text.contains(NFPM_VERSION) {
            eprintln!(
                "xtask: warning: nfpm {NFPM_VERSION} is the tested version; found `{}`",
                text.trim()
            );
        }
    }
}

/// Builds one package for `target`.
pub fn package(context: &Context, target: &Target, format: Format) -> Result<PathBuf, String> {
    let arch = target.arch()?;
    let slug = &context.project.slug;
    let (packager, file) = match format {
        Format::Deb => ("deb", names::deb(slug, &context.version, arch)),
        _ => ("rpm", names::rpm(slug, &context.version, arch)),
    };
    let stage = fresh(context, &format!("{}-{packager}", target.triple))?;
    let share = stage.join("share");
    linux::write_share(context, &share)?;
    let binary = build::binary(context, &target.triple)?;
    let config = stage.join("nfpm.yaml");
    let packaging = context.root.join("packaging");
    render_template(
        context,
        "linux/nfpm.yaml.in",
        &config,
        &[
            ("arch", arch.deb()),
            ("binary", text(&binary)?),
            ("share", text(&share)?),
            ("packaging", text(&packaging)?),
        ],
    )?;
    check_version();
    let out = context.out.join(file);
    run(
        Command::new("nfpm")
            .args(["package", "--packager", packager, "--config"])
            .arg(&config)
            .arg("--target")
            .arg(&out),
        &format!("install nfpm {NFPM_VERSION} (https://nfpm.goreleaser.com/install/)"),
    )?;
    Ok(out)
}
