//! The universal `.app`: both slices joined by `lipo`, the Safari appex
//! embedded, ad-hoc signed inside out, zipped.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::stage::{fresh, licenses, render_template};
use super::target::MACOS_SLICES;
use super::tool::run;
use super::{Context, archive, build, macos_sign, names, safari};

/// Builds `WebeSign.app` into `websign-<v>-macos-universal.zip`, with the
/// Safari appex in `Contents/PlugIns` (`safari/SPEC.md`). Signatures are ad
/// hoc for now (`macos_sign.rs`).
pub fn app(context: &Context) -> Result<PathBuf, String> {
    let project = &context.project;
    let stage = fresh(context, "macos")?;
    let bundle = stage.join(format!("{}.app", project.name));
    let contents = bundle.join("Contents");
    let executable = contents.join("MacOS").join(&project.slug);

    let slices = MACOS_SLICES
        .iter()
        .map(|triple| build::binary(context, triple))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(parent) = executable.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    run(
        Command::new("lipo")
            .arg("-create")
            .args(&slices)
            .arg("-output")
            .arg(&executable),
        "lipo comes with the Xcode command line tools; universal apps are built on macOS",
    )?;

    write_info_plist(context, &contents.join("Info.plist"))?;
    licenses(context, &contents.join("Resources/licenses"))?;
    safari::embed(context, &contents, &executable)?;
    macos_sign::sign(context, &bundle, None)?;

    let out = context
        .out
        .join(names::macos_zip(&project.slug, &context.version));
    archive::zip(&stage, &out)?;
    Ok(out)
}

/// Renders `Info.plist`. The URL-scheme entry comes from the app's
/// registration code, the one place that declares the scheme.
fn write_info_plist(context: &Context, to: &Path) -> Result<(), String> {
    let url_types = websign_registration::url_scheme::info_plist_url_types();
    render_template(
        context,
        "macos/Info.plist.in",
        to,
        &[("url_types", &url_types)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::project;
    use crate::testutil::TempDir;

    #[test]
    fn info_plist_declares_the_bundle_and_the_url_scheme() {
        let root = crate::root::repo_root().unwrap();
        let (project, version) = project::load(&root).unwrap();
        let out = TempDir::new();
        let plist = out.path().join("Info.plist");
        let expected = [
            format!("<string>{}</string>", project.bundle_id),
            format!("<string>{}</string>", project.url_scheme),
        ];
        let context = Context {
            root,
            out: out.path().to_path_buf(),
            project,
            version,
            no_build: true,
        };
        write_info_plist(&context, &plist).unwrap();
        let text = std::fs::read_to_string(plist).unwrap();
        assert!(text.contains("<key>CFBundleURLSchemes</key>") && !text.contains("{{"));
        assert!(expected.iter().all(|line| text.contains(line)), "{text}");
    }
}
