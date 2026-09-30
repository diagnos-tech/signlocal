# crates/websign-ui-model/src/diagnostics

- `mod.rs` — The diagnostics window's decisions (`docs/ux.md` §8): traffic lights, the first-steps strip, and the exact "Copy diagnostics" text.
- `onboarding.rs` — "Getting started" (`docs/ux.md` §8.2).
- `report.rs` — The exact text "Copy diagnostics" puts on the clipboard (`docs/ux.md` §8.7).
- `report/` — Golden and edge tests of the report text.
- `report_lines.rs` — One formatter per section of the diagnostics report.
- `status.rs` — Traffic lights per tab and overall (`docs/ux.md` §8.1).
