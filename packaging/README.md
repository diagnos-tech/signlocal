# packaging

Everything that turns the `websign` binary and the extension into release
artifacts: Linux packages (nfpm: deb, rpm) and desktop entries, the Windows
zip layout (and later MSIX), the macOS `.app` bundle (Info.plist with the
`websign:` scheme; later the Mac App Store bundle and the PKCS#11 Add-on).

- Contract: [`docs/architecture/packaging-and-release.md`](../docs/architecture/packaging-and-release.md).
- Built by `cargo xtask package --target <triple>`.
- Proven material to promote: `docs/prototypes/kit/msix/` (MSIX manifest and
  scripts), `docs/prototypes/kit/macos/sandbox/` (entitlements).
