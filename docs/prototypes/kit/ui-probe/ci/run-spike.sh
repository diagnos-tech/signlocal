#!/usr/bin/env bash
# Runs every approach for one OS label, never failing on a single approach.
# Usage: run-spike.sh <os-label>   (from the repository root)
# Env: PROFILE=debug to use a debug build (default release).
# Output: ui-spike-out/<os-label>-<approach>.png, results.md, GitHub summary.
set -u

os="$1"
out="ui-spike-out"
kit="docs/prototypes/kit"
profile="${PROFILE:-release}"
target="${CARGO_TARGET_DIR:-$kit/target}/$profile"
profile_flag="--release"
[ "$profile" = "debug" ] && profile_flag=""
mkdir -p "$out"
exe=""
[ "${OS:-}" = "Windows_NT" ] && exe=".exe"
results="$out/results.md"
{
  echo "| approach | status | detail |"
  echo "|---|---|---|"
} > "$results"

# macOS has no `timeout` by default; the app also has its own --timeout-secs.
limit() {
  local secs="$1"; shift
  if timeout --version >/dev/null 2>&1; then timeout "$secs" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then gtimeout "$secs" "$@"
  else "$@"; fi
}

record() { echo "| $1 | $2 | $3 |" >> "$results"; }

for renderer in auto wgpu glow; do
  log="$out/$os-$renderer.log"
  if limit 60 "$target/ui-probe$exe" --renderer "$renderer" --timeout-secs 45 \
      --out "$out/$os-$renderer.png" > "$log" 2>&1; then
    record "window --renderer $renderer" "ok" "$(grep -h '^RESULT' "$log" | tail -1 | sed 's/|/\\|/g')"
  else
    record "window --renderer $renderer" "FAIL" "$(tail -3 "$log" | tr '\n' ' ' | sed 's/|/\\|/g')"
  fi
done

log="$out/$os-kittest.log"
if (cd "$kit" && UI_PROBE_KITTEST_OUT="$PWD/../../../$out/$os-kittest.png" \
    limit 300 cargo test -p ui-probe $profile_flag --test snapshot -- --nocapture) > "$log" 2>&1; then
  record "kittest (wgpu)" "ok" "$(grep -h 'KITTEST' "$log" | tail -1)"
else
  record "kittest (wgpu)" "FAIL" "$(grep -hiE 'panicked|error|adapter' "$log" | head -2 | tr '\n' ' ' | sed 's/|/\\|/g')"
fi

echo "== ui-spike results: $os =="
cat "$results"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  { echo "### ui-spike: $os"; cat "$results"; } >> "$GITHUB_STEP_SUMMARY"
fi
exit 0
