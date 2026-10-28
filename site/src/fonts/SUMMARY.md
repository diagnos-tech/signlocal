# site/src/fonts

Self-hosted fonts (SIL OFL-1.1, no Reserved Font Name). They are trimmed copies of the Fontsource Latin
files: weights 400 to 700 only, the characters the seven languages and the pages use, and the few OpenType
features the styles rely on. `subset.sh` rebuilds them from the pinned upstream releases.

- `inter-subset.woff2` — Inter 4.1, variable (optical size, weights 400 to 700)
- `jetbrains-mono-subset.woff2` — JetBrains Mono, variable (weights 400 to 700)
- `OFL-Inter.txt` — Inter's licence
- `OFL-JetBrains-Mono.txt` — JetBrains Mono's licence
- `subset.sh` — downloads `@fontsource-variable/inter` and `@fontsource-variable/jetbrains-mono` 5.3.0 and writes the two files with fonttools
