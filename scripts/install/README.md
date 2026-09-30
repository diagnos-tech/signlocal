# scripts/install — one-line installers

`install.sh` (macOS, Linux) and `install.ps1` (Windows), published with every
release. Contract: [`docs/architecture/packaging-and-release.md`](../../docs/architecture/packaging-and-release.md)
(§Install locations, §Install commands); user steps: [`docs/install.md`](../../docs/install.md).

Behavior common to both: pick the artifact for the OS and CPU; download it and
`SHA256SUMS` with `gh` (when logged in), else `WEBSIGN_GITHUB_TOKEN` as a bearer
token, else anonymous HTTPS; **verify the hash before touching the system** (a
mismatch or a missing line aborts); install; run `websign install`; print the
extension step. Idempotent: a second run upgrades in place.

| | `install.sh` | `install.ps1` |
|---|---|---|
| Version | `--version <v>` (or `WEBSIGN_VERSION`) | `-Version <v>` |
| Remove | `--uninstall` | `-Uninstall` |
| Skip registration | `--no-register` | `-NoRegister` |
| Preview | `--dry-run` (downloads and verifies, changes nothing) | `-DryRun` |
| Location | `--prefix DIR` (Linux root, default `~/.local`; macOS folder of the app) | `-InstallDir DIR` |
| Other | `--yes` (ask nothing, keep every default), `--system` (Linux: the deb/rpm through `sudo` or as root, without asking; fails where unusable) | `-NoPath`, `-NoShortcut` |

Linux installs the tarball per user by default (no root). On a deb or rpm
distribution it asks whether to use the system package instead; the default is
no, and `--yes` never implies it, so an unattended run cannot escalate. macOS never needs `sudo`: `/Applications` when
writable, else `~/Applications`; the quarantine flag is removed from
`WebeSign.app` only. Windows needs no administrator rights; the Mark of the Web
is removed from the installed files only, and a non-empty `-InstallDir` without
`websign.exe` is never emptied or deleted.

The scripts cannot read `project.toml`, so their identifiers (`SLUG`,
`APP_NAME`, `BUNDLE_ID`, the repository) are written out; `cargo xtask check
release` fails when they differ from `project.toml`.

Test hooks (also usable for offline installs): `WEBSIGN_RELEASE_DIR` (folder
with the release files instead of downloading), `WEBSIGN_OS_RELEASE`,
`WEBSIGN_UNAME_S`, `WEBSIGN_UNAME_M`, `WEBSIGN_INSTALL_LIB=1` (source
`install.sh` without running it), `WEBSIGN_REPO`.

## Tests

```sh
shellcheck -s sh scripts/install/install.sh scripts/install/tests/install_sh_test.sh
sh scripts/install/tests/install_sh_test.sh            # Linux and macOS
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install\tests\install_ps1_selftest.ps1   # 5.1
pwsh -NoProfile -File scripts/install/tests/install_ps1_selftest.ps1   # 7; also on Linux with PROCESSOR_ARCHITECTURE=AMD64
```

CI (`installers` job in `.github/workflows/ci.yml`) runs shellcheck and the
shell tests on Linux and macOS, and the self-test under PowerShell 7 and
Windows PowerShell 5.1 on Windows. Each self-test run starts the installer
with the same PowerShell that runs the test.
