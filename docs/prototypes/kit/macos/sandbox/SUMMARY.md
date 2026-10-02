# macos/sandbox

Everything needed to run the probe inside the Mac App Store sandbox and record what the sandbox allows.

- `Info.plist` — minimal bundle metadata; the bundle identifier defines the sandbox container
- `entitlements.mas.plist` — the entitlements the store app would carry, with the reason for each
- `run-sandboxed.sh` — runs any probe command inside the sandbox and prints the sandbox denials from the system log
- `sandbox-test.sh` — the experiment: sandbox applied, manifests written, host over stdio, Keychain, CryptoTokenKit query, PKCS#11 loading
