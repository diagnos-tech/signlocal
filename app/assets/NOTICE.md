# Third-party assets in the SignLocal app

The `websign` binary embeds the fonts and icons below. Their licenses travel with the binary; the app's
About/notices view shows this file. Rust crates (code, not assets) are listed separately by `cargo about`.

| Asset | Version | Copyright | License | License text |
|-------|---------|-----------|---------|--------------|
| Inter (Regular, Medium, SemiBold; subset) | 3.19 | 2020 The Inter Project Authors | SIL Open Font License 1.1 | `fonts/OFL-Inter.txt` |
| JetBrains Mono (Regular, Medium; ASCII subset) | 2.211 | 2020 The JetBrains Mono Project Authors | SIL Open Font License 1.1 | `fonts/OFL-JetBrainsMono.txt` |
| Phosphor Icons (regular and fill; only the glyphs the app uses), via the `egui-phosphor` crate | 2.1 | 2023 Phosphor Icons | MIT | `icons/LICENSE-Phosphor.txt` |
| Ubuntu Light (egui's fallback for scripts outside the Inter subset), via `epaint_default_fonts` | egui 0.36 | Canonical Ltd. | Ubuntu Font Licence 1.0 | `licenses/egui-default-fonts/UFL.txt` |
| Noto Emoji (egui fallback), via `epaint_default_fonts` | egui 0.36 | Google Inc. | SIL Open Font License 1.1 | `licenses/egui-default-fonts/OFL.txt` |
| emoji-icon-font (egui fallback), via `epaint_default_fonts` | egui 0.36 | 2014 John Slegers | MIT | `licenses/egui-default-fonts/emoji-icon-font-mit-license.txt` |
| Hack (egui fallback), via `epaint_default_fonts` | egui 0.36 | 2018 Source Foundry Authors; Bitstream Vera Sans Mono 2003 Bitstream Inc. | MIT and Bitstream Vera License | `licenses/egui-default-fonts/Hack-Regular.txt` |

The Inter and JetBrains Mono files here are modified versions (subsets, no hinting) made by `fonts/subset.sh`;
neither family declares a Reserved Font Name, so the OFL allows them under the original names.

The egui fallback license texts are copied from `epaint_default_fonts` 0.36.2; update them when egui
changes its default fonts.
