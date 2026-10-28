#!/usr/bin/env bash
# Proves each installed browser in turn: the cross-browser core
# (browsers/core.spec.ts) with SIGNLOCAL_E2E_BROWSER_NAME/SIGNLOCAL_E2E_BROWSER
# set, so the host is registered as a person's install does.
#
# Usage: run-browsers.sh <list file>
#   Each line of the list is `<label>=<executable>`: the label is a browser
#   name of lib/browsers.ts, optionally with a suffix (`firefox-esr`).
#   An empty or missing executable means the browser could not be installed
#   on this machine: it is reported as skipped, not failed.
#
# Needs the variables of a normal run (SIGNLOCAL_E2E_APP, keys; see
# lib/environment.ts). SIGNLOCAL_E2E_SCREENSHOTS gets one folder per browser.
# Writes a Markdown table (browser, version, result) to
# SIGNLOCAL_E2E_BROWSER_RESULTS (default ../target/e2e-results/browsers.md) and,
# in GitHub Actions, to the job summary. Exits non-zero when any installed
# browser failed.
set -uo pipefail

list=${1:?usage: run-browsers.sh <list file>}
cd "$(dirname "$0")" || exit 1
shots=${SIGNLOCAL_E2E_SCREENSHOTS:-${TMPDIR:-/tmp}/signlocal-e2e-screenshots}
results=${SIGNLOCAL_E2E_BROWSER_RESULTS:-../target/e2e-results/browsers.md}
mkdir -p "$(dirname "$results")"
os=${RUNNER_OS:-$(uname -s)}
printf '| OS | Browser | Version | Result |\n|---|---|---|---|\n' > "$results"

# Playwright's own deadline, since macOS has no `timeout`: a hung browser
# fails its run instead of the job.
run_suite() {
  local args=(playwright test --reporter=line --global-timeout=900000)
  if [[ $os == Linux ]]; then
    xvfb-run -a -s "-screen 0 1600x900x24" bunx "${args[@]}"
  else
    bunx "${args[@]}"
  fi
}

failed=0
while IFS='=' read -r label executable || [[ -n $label ]]; do
  [[ -z $label || $label == \#* ]] && continue
  name=${label%%-*}
  if [[ -z $executable || ! -e $executable ]]; then
    echo "::notice::$label is not installed on this runner: skipped"
    printf '| %s | %s | — | skipped (not installable here) |\n' "$os" "$label" >> "$results"
    continue
  fi
  echo "::group::$label ($executable)"
  folder="$shots/$label"
  mkdir -p "$folder" || echo "::warning::cannot create $folder"
  if SIGNLOCAL_E2E_BROWSER_NAME=$name SIGNLOCAL_E2E_BROWSER=$executable \
    SIGNLOCAL_E2E_SCREENSHOTS=$folder run_suite < /dev/null; then
    result=passed
  else
    result=FAILED
    failed=1
    echo "::error::the cross-browser core failed in $label"
  fi
  echo "::endgroup::"
  # Written by the suite's first test, so absent when the browser never started.
  version=unknown
  if [[ -s $folder/browser-version.txt ]]; then
    version=$(tr -d '\r\n' < "$folder/browser-version.txt")
  fi
  printf '| %s | %s | %s | %s |\n' "$os" "$label" "$version" "$result" >> "$results"
done < "$list"

cat "$results"
if [[ -n ${GITHUB_STEP_SUMMARY:-} ]]; then
  { echo "### Cross-browser core"; echo; cat "$results"; } >> "$GITHUB_STEP_SUMMARY"
fi
exit "$failed"
