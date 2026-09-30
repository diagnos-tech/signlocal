# xtask/src/package

- `archive.rs` — zip and tar.gz through the system tools
- `build.rs` — finds (and builds) the release binary of a target
- `extension.rs` — renames the WXT browser zips to the release names
- `formats.rs` — artifact kinds and which target OS each applies to
- `linux.rs` — the share tree shared by packages and tarball; the tar.gz
- `macos.rs` — universal `.app`: lipo, Info.plist, ad-hoc signature, zip
- `names.rs` — exact artifact file names
- `nfpm.rs` — deb and rpm through nfpm (version pin)
- `project.rs` — identifiers from project.toml and the workspace version
- `sha256.rs` — SHA-256 without dependencies
- `stage.rs` — staging folders, template rendering, license files
- `sums.rs` — SHA256SUMS
- `target.rs` — supported target triples
- `template.rs` — strict `{{key}}` substitution
- `tool.rs` — running external tools with actionable errors
- `windows.rs` — the Windows zip
