//! Linux files shared by the packages and the tarball, and the tarball.

use std::path::{Path, PathBuf};

use super::names;
use super::stage::{copy, fresh, licenses, render_template};
use super::target::Target;
use super::{Context, archive, build};

/// Writes the `share/` tree (desktop entry, AppStream data, icon, licenses)
/// that `/usr/share` (packages) and `~/.local/share` (tarball) both receive.
pub fn write_share(context: &Context, share: &Path) -> Result<(), String> {
    let slug = &context.project.slug;
    render_template(
        context,
        "linux/websign.desktop.in",
        &share.join(format!("applications/{slug}.desktop")),
        &[],
    )?;
    render_template(
        context,
        "linux/websign.metainfo.xml.in",
        &share.join(format!(
            "metainfo/{}.metainfo.xml",
            context.project.bundle_id
        )),
        &[],
    )?;
    copy(
        &context.root.join("packaging/linux/websign.svg"),
        &share.join(format!("icons/hicolor/scalable/apps/{slug}.svg")),
    )?;
    licenses(context, &share.join("doc").join(slug))
}

/// `websign-<v>-linux-<arch>.tar.gz`: `bin/` and `share/` under one folder,
/// which `install.sh` copies into its prefix.
pub fn tarball(context: &Context, target: &Target) -> Result<PathBuf, String> {
    let slug = &context.project.slug;
    let file = names::tarball(slug, &context.version, target.arch()?);
    let folder = file.trim_end_matches(".tar.gz");
    let stage = fresh(context, &format!("{}-tar", target.triple))?.join(folder);
    let binary = build::binary(context, &target.triple)?;
    copy(&binary, &stage.join("bin").join(slug))?;
    write_share(context, &stage.join("share"))?;
    let out = context.out.join(file);
    archive::tar_gz(&stage, &out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::project;
    use crate::testutil::TempDir;

    #[test]
    fn the_share_tree_renders_with_no_placeholder_left() {
        let root = crate::root::repo_root().unwrap();
        let (project, version) = project::load(&root).unwrap();
        let out = TempDir::new();
        let context = Context {
            root,
            out: out.path().to_path_buf(),
            project,
            version,
            no_build: true,
        };
        let share = out.path().join("share");
        write_share(&context, &share).unwrap();
        let desktop = std::fs::read_to_string(share.join("applications/websign.desktop")).unwrap();
        assert!(desktop.contains("Exec=websign\n") && !desktop.contains("{{"));
        assert!(share.join("doc/websign/LICENSE").is_file());
    }
}
