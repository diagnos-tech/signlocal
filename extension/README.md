# WebeSign extension

The bridge between pages using `@websign/sdk` and the WebeSign app, over
native messaging. It announces itself to pages, validates every request,
attaches the origin the browser reports, and keeps the connection to the app
open while in use. Its only UI is the toolbar popup. Built with WXT (MV3) for
Chrome, Edge, Firefox and Safari.

- Contract: [`SPEC.md`](SPEC.md); wire: [`docs/architecture/protocol.md`](../docs/architecture/protocol.md).
- Proven reference: `docs/prototypes/kit/extension/`.
- Scripts: `bun run dev`, `bun run build`, `bun run typecheck`, `bun run test`.
  `bun install` runs `wxt prepare` (generates `.wxt/`).
- Locales in `public/_locales` are generated from `i18n/` by `cargo xtask gen`.
- License: GPL-3.0-or-later.
