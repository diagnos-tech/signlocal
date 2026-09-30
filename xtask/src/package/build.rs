//! Finding and building the `websign` binary for a target.

use std::path::PathBuf;
use std::process::Command;

use super::Context;
use super::tool::run;

/// `websign-app`'s binary name.
const BINARY: &str = "websign";

fn target_dir(context: &Context) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| context.root.join("target"), PathBuf::from)
}

/// The release binary for `triple`, built first unless `--no-build`.
pub fn binary(context: &Context, triple: &str) -> Result<PathBuf, String> {
    if !context.no_build {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
        run(
            Command::new(cargo)
                .args(["build", "--release", "--locked", "-p", "websign-app"])
                .args(["--target", triple])
                .current_dir(&context.root),
            "install Rust (rustup)",
        )?;
    }
    let file = if triple.contains("windows") {
        format!("{BINARY}.exe")
    } else {
        BINARY.to_owned()
    };
    let path = target_dir(context).join(triple).join("release").join(file);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "{} does not exist; build it with `cargo build --release -p websign-app --target {triple}`",
            path.display()
        ))
    }
}
