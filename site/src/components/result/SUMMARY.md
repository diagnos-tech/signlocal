# site/src/components/result

The cards of the wizard's result step; each variant is rendered and the wizard shows the ones that match.

- `CertificateCard.astro` — get the token, card or certificate file ready
- `CertificateFile.astro` — installing a certificate file (Windows, macOS; not yet Linux)
- `CertificateHolder.astro` — a token or a card: driver, plug-in step, card service on Linux
- `ChoiceChip.astro` — one of the summary chips of the choices, with its edit button
- `ConnectCard.astro` — connect signlocal to the browser (the open-the-app-once variant for Safari)
- `DevicePicker.astro` — the list of known models and the driver of the chosen one
- `ExtensionCard.astro` — add the extension (every browser but Safari)
- `ExtensionChromium.astro` — Chrome, Edge, Opera, Brave and others: store button or developer install
- `ExtensionFirefox.astro` — Firefox: add-ons page or the temporary add-on
- `ExtensionSafari.astro` — Safari: the settings that turn the extension on
- `InstallCard.astro` — install the app: automatic or manual, per system
- `InstallLinux.astro` — Linux install steps: script, deb and rpm packages
- `InstallMacos.astro` — macOS install steps
- `InstallWindows.astro` — Windows install steps
- `NoReleaseCard.astro` — shown while no release is published
- `ResultHeader.astro` — title, choices, share link, notices and the release line
- `StepCard.astro` — a card's frame: number, title, live status, collapsible body
- `TestCard.astro` — the last card: test your setup
