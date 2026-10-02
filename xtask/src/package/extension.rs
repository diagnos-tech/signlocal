//! The browser extension zips, renamed to the release names. They are built
//! by `bun run zip` in `extension/` (WXT), which writes
//! `<package>-<version>-<browser>.zip`; the Firefox sources zip is ignored.

use std::path::{Path, PathBuf};

use super::stage::copy;
use super::{Context, names};
use crate::fsutil::list_dir;

/// WXT browser suffix and the name the release uses for it.
const BROWSERS: [(&str, &str); 2] = [("chrome", "chromium"), ("firefox", "firefox")];

/// The one WXT output ending in `-<suffix>.zip`.
fn find(output: &Path, suffix: &str) -> Result<PathBuf, String> {
    let ending = format!("-{suffix}.zip");
    let mut found: Vec<String> = list_dir(output)
        .map_err(|e| format!("{e}; run `bun run zip` in extension/"))?
        .into_iter()
        .filter(|(name, is_dir)| !is_dir && name.ends_with(&ending))
        .map(|(name, _)| name)
        .collect();
    match (found.pop(), found.is_empty()) {
        (Some(name), true) => Ok(output.join(name)),
        (None, _) => Err(format!(
            "no *{ending} in {}; run `bun run zip` in extension/",
            output.display()
        )),
        (Some(_), false) => Err(format!("several *{ending} in {}", output.display())),
    }
}

/// Copies both browser zips into the output folder; returns the last one.
pub fn zips(context: &Context) -> Result<PathBuf, String> {
    let output = context.root.join("extension/.output");
    let mut last = output.clone();
    for (suffix, browser) in BROWSERS {
        let source = find(&output, suffix)?;
        last = context.out.join(names::extension_zip(
            &context.project.slug,
            &context.version,
            browser,
        ));
        copy(&source, &last)?;
    }
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn picks_the_browser_zip_and_not_the_sources_zip() {
        let dir = TempDir::new();
        dir.write("websign-extension-0.1.0-firefox.zip", "");
        dir.write("websign-extension-0.1.0-sources.zip", "");
        let path = find(dir.path(), "firefox").unwrap();
        assert!(path.ends_with("websign-extension-0.1.0-firefox.zip"));
        assert!(find(dir.path(), "chrome").is_err());
    }
}
