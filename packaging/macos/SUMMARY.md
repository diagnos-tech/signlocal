# packaging/macos

- `Extension-Info.plist.in` — Info.plist template of the Safari appex (`safari/SPEC.md` §4 keys included)
- `Info.plist.in` — bundle Info.plist template; the URL-scheme fragment comes from `websign-registration` through xtask
- `safari-extension.entitlements` — the Safari appex's sandbox: smart cards, USB, read-only PKCS#11 locations
- `safari-host.entitlements` — the `websign` copy inside the appex: inherits the appex's sandbox
