# scripts/install — one-line installers

`install.sh` (macOS, Linux) and `install.ps1` (Windows), published with every
release. Contract (behavior, flags, install locations, checksum verification,
private-repository authentication, unsigned-build handling):
[`docs/architecture/packaging-and-release.md`](../../docs/architecture/packaging-and-release.md)
§One-line install and §Install locations.

Requirements for both scripts:

- POSIX `sh` (not bash-only) for `install.sh`; Windows PowerShell 5.1 and
  PowerShell 7 for `install.ps1`.
- Never run as root unless `--system` (Linux packages path).
- Verify `SHA256SUMS` before touching the system; abort on mismatch.
- Idempotent: running twice upgrades in place.
- `--uninstall` runs `websign uninstall` and removes the files.
