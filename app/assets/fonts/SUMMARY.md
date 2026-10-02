# app/assets/fonts

The UI fonts (`docs/ux.md` §11.2), embedded by `app/src/ui/fonts.rs`. Both are SIL Open Font License 1.1 without a
Reserved Font Name; the subsets are modified versions under the same license.

- `Inter-Regular.ttf` — Inter 3.19 weight 400, subset: Latin, Greek, Cyrillic, punctuation, arrows
- `Inter-Medium.ttf` — Inter 3.19 weight 500, same subset
- `Inter-SemiBold.ttf` — Inter 3.19 weight 600, same subset
- `JetBrainsMono-Regular.ttf` — JetBrains Mono 2.211 weight 400, ASCII subset (codes, paths, IDs)
- `JetBrainsMono-Medium.ttf` — JetBrains Mono 2.211 weight 500, ASCII subset (the verification code)
- `OFL-Inter.txt` — Inter's copyright notice and the OFL-1.1 text
- `OFL-JetBrainsMono.txt` — JetBrains Mono's copyright notice and the OFL-1.1 text
- `subset.sh` — rebuilds the five font files from the upstream releases (pinned npm tarballs) with fonttools
