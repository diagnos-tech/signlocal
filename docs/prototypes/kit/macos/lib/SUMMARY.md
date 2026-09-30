# macos/lib

Shell helpers for the macOS proof scripts, written for the bash 3.2 that ships with macOS.

- `bundle.sh` — wraps `websign-probe` in `WebeSign.app`, signed ad hoc with the store entitlements, so it runs sandboxed like the store app
- `common.sh` — shared functions (logging, checks, keychain handling); sourced, never run
