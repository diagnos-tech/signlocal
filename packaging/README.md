# packaging

Everything that turns the `websign` binary and the extension into release
artifacts: Linux packages (nfpm: deb, rpm), desktop entry and AppStream data,
the Windows zip, the macOS `.app` bundle (Info.plist with the `websign:`
scheme). Later: MSIX and the Mac App Store bundle.

- Contract: [`docs/architecture/packaging-and-release.md`](../docs/architecture/packaging-and-release.md).
- Files here are **templates** (`*.in`, `{{key}}` placeholders): identifiers
  come from `project.toml` and the workspace version, filled in by
  `xtask/src/package/`. An unknown key fails the build.
- Build: `cargo xtask package --target <triple> [--format ...]`, then
  `cargo xtask package --sums-only` once every artifact is in `dist/`.
  Requires nfpm **2.43.0** for deb/rpm (`NFPM_VERSION` in `xtask/src/package/nfpm.rs`),
  `lipo` + `codesign` on macOS, `zip` (or bsdtar) and `tar`. The extension zips
  come from `bun run zip` in `extension/`.
- Inspect a package: `dpkg-deb -I -c dist/websign_<v>_amd64.deb`, `rpm -qpi -R dist/websign-<v>-1.x86_64.rpm`.
- Proven material to promote: `docs/prototypes/kit/msix/` (MSIX manifest and
  scripts), `docs/prototypes/kit/macos/sandbox/` (entitlements).
- Icons. `app/assets` has no app icon yet, so Linux ships the neutral
  placeholder `linux/websign.svg` as the scalable hicolor icon (enough for
  freedesktop menus and software centers), and the `.app` and `websign.exe`
  carry none. `TODO(gustavo)`: the brand icon, pending the trademark search.
  When it lands as a square PNG master in `app/assets`, `cargo xtask package`
  derives the rest without new heavy dependencies: PNG sizes for hicolor
  (16–512), `.icns` and `.ico` are both containers of PNG images, written with
  the `png` crate already in `Cargo.lock`; rasterizing the SVG itself would
  need `resvg`, which is why the master is a PNG. `install.sh --uninstall`
  already removes every `hicolor/*/apps/websign.*`.
