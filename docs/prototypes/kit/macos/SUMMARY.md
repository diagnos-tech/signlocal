# macos

Scripts of the macOS proof: Keychain signing and the App Sandbox experiment.

- `ci-macos.sh` — creates a temporary keychain with RSA and EC test identities, then runs `list` and `sign` against it and checks every signature
- `lib/` — shell helpers shared by the macOS scripts
- `sandbox/` — the sandbox experiment: entitlements, bundle metadata, and the scripts that run the probe inside the App Sandbox
