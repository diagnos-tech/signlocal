//! The Windows zip: `websign.exe`, `README.txt` and the licenses, flat, so
//! `install.ps1` can expand it straight into the install folder.

use std::path::PathBuf;

use super::stage::{copy, fresh, licenses, render_template};
use super::target::Target;
use super::{Context, archive, build, names};

pub fn zip(context: &Context, target: &Target) -> Result<PathBuf, String> {
    let slug = &context.project.slug;
    let stage = fresh(context, &format!("{}-zip", target.triple))?;
    copy(
        &build::binary(context, &target.triple)?,
        &stage.join(format!("{slug}.exe")),
    )?;
    render_template(
        context,
        "windows/README.txt.in",
        &stage.join("README.txt"),
        &[],
    )?;
    licenses(context, &stage.join("licenses"))?;
    let out = context
        .out
        .join(names::windows_zip(slug, &context.version, target.arch()?));
    archive::zip(&stage, &out)?;
    Ok(out)
}
