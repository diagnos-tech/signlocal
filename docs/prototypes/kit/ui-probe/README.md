# ui-probe

```sh
ui-probe --renderer auto|wgpu|glow --out shot.png   # window path; prints a RESULT line
cargo test -p ui-probe --test snapshot -- --nocapture # headless kittest path (wgpu)
```

`auto` tries wgpu first and retries with glow only when the graphics stack fails to
start (never for a missing display). Linux needs Xvfb plus mesa software drivers; see
`.github/workflows/ui-spike.yml` for the exact packages per distro.
