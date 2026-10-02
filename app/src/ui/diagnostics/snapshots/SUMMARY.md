# app/src/ui/diagnostics/snapshots

Kittest baselines per OS (fonts rasterize differently). A folder counts as reviewed once it has a `SUMMARY.md`;
until then tests only write `<name>.new.png` for review.

- `linux/` — baselines rendered with Vulkan llvmpipe
- `.gitignore` — keeps kittest's `.new`, `.diff` and `.old` images out of git
