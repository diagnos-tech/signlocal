//! The Safari web extension inside `SignLocal.app` (`safari/SPEC.md`): the
//! appex compiled from `safari/`, the WXT safari build as its resources, and
//! a copy of the host binary it starts, each signed inside out.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::project::Project;
use super::stage::{copy, fresh, render_template};
use super::tool::run;
use super::{Context, macos_sign};

/// The WXT build the appex serves, relative to the repository root.
pub const WEB_BUILD: &str = "extension/.output/safari-mv3";

/// Swift folders compiled together into the appex's one module.
const SWIFT_SOURCES: [&str; 2] = ["safari/Relay", "safari/Extension"];

/// `(lipo slice name, swiftc target)`; the version is the app's floor
/// (`LSMinimumSystemVersion` in both Info.plist templates).
const SWIFT_TARGETS: [(&str, &str); 2] = [
    ("arm64", "arm64-apple-macos13.0"),
    ("x86_64", "x86_64-apple-macos13.0"),
];

/// Builds `Contents/PlugIns/<Name>Extension.appex` around `host` (the
/// universal `websign`) and returns its path.
pub fn embed(context: &Context, contents: &Path, host: &Path) -> Result<PathBuf, String> {
    let web = web_build(&context.root)?;
    let executable = appex_executable(&context.project);
    let appex = contents.join("PlugIns").join(format!("{executable}.appex"));
    let inner = appex.join("Contents");
    let helper = inner.join("MacOS").join(&context.project.slug);

    compile(context, &inner.join("MacOS").join(&executable))?;
    copy(host, &helper)?;
    copy_tree(&web, &inner.join("Resources"))?;
    render_template(
        context,
        "macos/Extension-Info.plist.in",
        &inner.join("Info.plist"),
        &[("appex_executable", &executable)],
    )?;
    macos_sign::sign(context, &helper, Some("safari-host.entitlements"))?;
    macos_sign::sign(context, &appex, Some("safari-extension.entitlements"))?;
    Ok(appex)
}

/// The appex's executable and bundle name.
pub fn appex_executable(project: &Project) -> String {
    format!("{}Extension", project.name)
}

/// The WXT safari build, which must exist: the app never ships without it.
fn web_build(root: &Path) -> Result<PathBuf, String> {
    let web = root.join(WEB_BUILD);
    if web.join("manifest.json").is_file() {
        Ok(web)
    } else {
        Err(format!(
            "{WEB_BUILD} is missing: in extension/, run \
             `WEBSIGN_CHANNEL=direct bunx wxt build -b safari` first"
        ))
    }
}

/// Compiles both slices with `swiftc` and joins them with `lipo`.
fn compile(context: &Context, out: &Path) -> Result<(), String> {
    let sources = swift_sources(&context.root)?;
    let work = fresh(context, "safari-build")?;
    let mut slices = Vec::new();
    for (arch, target) in SWIFT_TARGETS {
        let slice = work.join(format!("appex-{arch}"));
        run(
            Command::new("xcrun").args(swiftc_args(target, &slice, &sources)),
            "xcrun swiftc comes with Xcode; the Safari appex is built on macOS",
        )?;
        slices.push(slice);
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    run(
        Command::new("lipo")
            .arg("-create")
            .args(&slices)
            .arg("-output")
            .arg(out),
        "lipo comes with the Xcode command line tools",
    )
}

/// Every `.swift` file of [`SWIFT_SOURCES`], sorted for reproducible builds.
fn swift_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    for folder in SWIFT_SOURCES {
        for (name, is_dir) in crate::fsutil::list_dir(&root.join(folder))? {
            if !is_dir && name.ends_with(".swift") {
                sources.push(root.join(folder).join(name));
            }
        }
    }
    Ok(sources)
}

/// `swiftc` arguments (after `xcrun`) for one slice. An app extension has no
/// `main`: Foundation's `NSExtensionMain` is the entry point, and
/// `-application-extension` refuses APIs extensions may not call.
fn swiftc_args(target: &str, out: &Path, sources: &[PathBuf]) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "swiftc",
        "-target",
        target,
        "-module-name",
        "SignLocalExtension",
        "-swift-version",
        "5",
        "-O",
        "-whole-module-optimization",
        "-parse-as-library",
        "-application-extension",
        "-framework",
        "Foundation",
        "-framework",
        "SafariServices",
        "-Xlinker",
        "-e",
        "-Xlinker",
        "_NSExtensionMain",
        "-o",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.push(out.into());
    args.extend(sources.iter().map(|source| source.into()));
    args
}

/// Copies the folder `from` into `to`, recursively.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("cannot create {}: {e}", to.display()))?;
    for (name, is_dir) in crate::fsutil::list_dir(from)? {
        if is_dir {
            copy_tree(&from.join(&name), &to.join(&name))?;
        } else {
            copy(&from.join(&name), &to.join(&name))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
