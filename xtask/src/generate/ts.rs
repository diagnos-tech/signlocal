//! Protocol types for TypeScript, exported by ts-rs from `websign-protocol`.
//!
//! The Rust types are the single definition of the wire; three packages get
//! the same files so none of them can drift from it.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::text::{summary, ts_header};
use crate::fsutil::{list_dir, read_text};
use crate::plan::Plan;

/// (package, has `generated/project.ts` from the `project` generator)
const PACKAGES: [(&str, bool); 3] = [("sdk", false), ("extension", true), ("clients/node", true)];

pub fn plan(root: &Path) -> Result<Plan, String> {
    let scratch = ScratchDir::new()?;
    export_bindings(root, scratch.path())?;
    let mut types = Vec::new();
    for (name, is_dir) in list_dir(scratch.path())? {
        if !is_dir && name.ends_with(".ts") && name != "index.ts" {
            let content = read_text(&scratch.path().join(&name))?;
            types.push((name.trim_end_matches(".ts").to_owned(), content));
        }
    }
    if types.is_empty() {
        return Err(
            "ts-rs exported no types; is the `typescript` feature of websign-protocol intact?"
                .into(),
        );
    }
    Ok(package_plan(&types))
}

/// Pure part: the files of every package for the exported `types`
/// (`(type name, file content)`, sorted by name).
fn package_plan(types: &[(String, String)]) -> Plan {
    let mut plan = Plan::default();
    for (package, has_project) in PACKAGES {
        let dir = format!("{package}/src/generated");
        for (name, content) in types {
            plan.add(format!("{dir}/{name}.ts"), content.clone());
        }
        plan.add(format!("{dir}/index.ts"), barrel(types, has_project));
        plan.add(format!("{dir}/SUMMARY.md"), summary_for(&dir, has_project));
        plan.owned_dirs.push(PathBuf::from(&dir));
        if has_project {
            plan.foreign
                .push(PathBuf::from(format!("{dir}/project.ts")));
        }
    }
    plan
}

/// Specifiers carry `.js` so the barrel is valid Node ESM, which (unlike a
/// bundler) does not guess extensions.
fn barrel(types: &[(String, String)], has_project: bool) -> String {
    let mut out = ts_header("crates/websign-protocol");
    for (name, _) in types {
        out.push_str(&format!("export type {{ {name} }} from \"./{name}.js\";\n"));
    }
    if has_project {
        out.push_str("export * from \"./project.js\";\n");
    }
    out
}

fn summary_for(dir: &str, has_project: bool) -> String {
    let description = if has_project {
        "protocol types from crates/websign-protocol and `project.ts` from project.toml (`index.ts` re-exports them)"
    } else {
        "protocol types generated from crates/websign-protocol (`index.ts` re-exports them)"
    };
    summary(dir, &[("*.ts", description)])
}

/// Runs the ts-rs export tests, which write one file per type into `out`.
///
/// `TS_RS_IMPORT_EXTENSION=js` makes the imports between types `./X.js`,
/// which Node ESM (and a consumer's `.d.ts` resolution) needs; bundlers
/// accept it too.
fn export_bindings(root: &Path, out: &Path) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .current_dir(root)
        .env("TS_RS_EXPORT_DIR", out)
        .env("TS_RS_IMPORT_EXTENSION", "js")
        .args([
            "test",
            "--quiet",
            "-p",
            "websign-protocol",
            "--features",
            "typescript",
        ])
        .arg("export_bindings")
        .output()
        .map_err(|e| format!("cannot run cargo to export the TypeScript types: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "exporting the TypeScript types failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    ))
}

/// A temporary folder removed on drop, so a failed run leaves nothing behind.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new() -> Result<ScratchDir, String> {
        let path = std::env::temp_dir().join(format!("xtask-ts-{}", std::process::id()));
        // A crashed run with a recycled process id may have left files here.
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("cannot create {}: {e}", path.display()))?;
        Ok(ScratchDir(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn types() -> Vec<(String, String)> {
        vec![("A".into(), "a".into()), ("B".into(), "b".into())]
    }

    #[test]
    fn every_package_gets_the_types_barrel_and_summary() {
        let plan = package_plan(&types());
        assert_eq!(plan.files.len(), 3 * 4);
        assert_eq!(plan.owned_dirs.len(), 3);
        assert_eq!(plan.foreign.len(), 2);
    }

    #[test]
    fn only_packages_with_project_constants_export_them() {
        assert!(barrel(&types(), true).ends_with("export * from \"./project.js\";\n"));
        assert!(!barrel(&types(), false).contains("project"));
        assert!(barrel(&types(), false).contains("export type { B } from \"./B.js\";"));
    }
}
