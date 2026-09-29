#!/usr/bin/env bash
# Linux proof with software keys, run by .github/workflows/prototypes.yml from
# docs/prototypes/kit and runnable by hand:
#
#   PROBE_EXE=target/release/websign-probe bash linux/ci-linux.sh
#
# What it proves, against an isolated SoftHSM2 token (see softhsm-setup.sh):
#   1. `list` sees the RSA and EC certificates and marks them as software;
#   2. `sign` signs every hash with every algorithm and each signature verifies;
#   3. a wrong PIN is reported as such;
#   4. modules registered with p11-kit (user directory) are discovered and a
#      registration that points nowhere is reported, not fatal;
#   5. the same certificate through two modules (SoftHSM directly and through
#      p11-kit-proxy.so) is listed once, with "+1 other path";
#   6. a key that demands the PIN on every signature (CKA_ALWAYS_AUTHENTICATE)
#      either signs or is refused with a message that names the reason;
#   7. `devices` runs on a machine without readers;
#   8. `report` writes a Markdown report with no token label and no PIN;
#   9. the browser path works end to end (nm-e2e), when Node and Chromium exist.
#
# Environment:
#   PROBE_EXE    the websign-probe binary (required)
#   REPORT_PATH  where to write the Markdown report (default: a temporary file)
#   CHROMIUM     Chromium/Chrome executable for the native messaging test
#                (default: the one playwright-core installed)
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
kit=$(dirname "$here")

probe=${PROBE_EXE:-}
if [[ -z $probe || ! -x $probe ]]; then
  echo "Set PROBE_EXE to the websign-probe binary (got: '${probe}')." >&2
  exit 2
fi
probe=$(cd "$(dirname "$probe")" && pwd)/$(basename "$probe")

work=$(mktemp -d "${TMPDIR:-/tmp}/websign-ci-linux.XXXXXX")
cleanup() { rm -rf "$work"; }
trap cleanup EXIT

report=${REPORT_PATH:-$work/report-linux.md}
failures=()
last_output=""
last_status=0

# ---------------------------------------------------------------- helpers ---

step() { printf '\n== %s\n' "$*"; }

# indent: shows a block of output two spaces in, under the command that made it.
indent() { sed 's/^/  /'; }

fail() {
  failures+=("$1")
  printf '  !! %s\n' "$1"
}

# run_probe ARGS...: runs the probe, shows its output indented and keeps it in
# $last_output / $last_status. A failing exit code is data, not an abort.
run_probe() {
  printf '\n> websign-probe %s\n' "$*"
  set +e
  last_output=$("$probe" "$@" 2>&1)
  last_status=$?
  set -e
  indent <<<"$last_output"
  echo "  (exit code $last_status)"
}

expect_status() {
  [[ $last_status -eq $1 ]] || fail "$2: exit code $last_status, expected $1"
}

expect_output() {
  grep -Fq -- "$1" <<<"$last_output" || fail "$2: output does not contain '$1'"
}

expect_no_output() {
  if grep -Fq -- "$1" <<<"$last_output"; then fail "$2: output contains '$1'"; fi
}

# expect_count REGEX N WHAT: exactly N lines match.
expect_count() {
  local found
  found=$(grep -Ec -- "$1" <<<"$last_output" || true)
  [[ $found -eq $2 ]] || fail "$3: $found line(s) match '$1', expected $2"
}

# use_home DIR / restore_home: p11-kit reads registrations from the user's
# config directory, so the tests point HOME at a directory of their own.
use_home() {
  saved_home=$HOME
  saved_xdg=${XDG_CONFIG_HOME-__unset__}
  export HOME="$1"
  unset XDG_CONFIG_HOME
}

restore_home() {
  export HOME="$saved_home"
  if [[ $saved_xdg == __unset__ ]]; then unset XDG_CONFIG_HOME; else export XDG_CONFIG_HOME="$saved_xdg"; fi
}

# --------------------------------------------------------------- the token ---

step "SoftHSM2 token"
# shellcheck source=softhsm-setup.sh
source "$here/softhsm-setup.sh"
softhsm_setup "$work/softhsm"
echo "  module: $SOFTHSM_MODULE"
export SOFTHSM2_CONF WEBSIGN_PROBE_PIN=$SOFTHSM_USER_PIN

pin_var=WEBSIGN_PROBE_PIN
# Only the SoftHSM module: what these steps prove is that module, not whatever
# else this machine happens to have registered.
isolate=(--no-known-modules --no-p11-kit --module "$SOFTHSM_MODULE")

# The four certificates the setup creates: 1 RSA (3 hashes x 2 algorithms) and
# 3 EC (3 hashes x 1 algorithm).
expected_certificates=4
expected_signatures=$((3 * 2 + 3 * 3 * 1))

# ------------------------------------------------------------------- list ---

step "list: certificates in the token"
run_probe list "${isolate[@]}"
expect_status 0 "list"
expect_count '^ *[0-9]+\. ' "$expected_certificates" "list"
for key in "RSA-2048" "EC P-256" "EC P-384" "EC P-521"; do
  expect_output "$key" "list"
done
expect_output "software" "list (SoftHSM must be marked as software)"
expect_output "PIN by app" "list (PKCS#11 PINs are collected by the app)"
expect_no_output "websign-test" "list (the token label must not appear)"

rsa_fingerprint=$(awk -F' [|] ' '/RSA-2048/ { print $NF; exit }' <<<"$last_output" | tr -d ' ')

# ------------------------------------------------------------- wrong PIN ---

step "sign: a wrong PIN is reported as a wrong PIN"
WRONG_PIN=000000 run_probe sign --cert "$rsa_fingerprint" --pin-env WRONG_PIN "${isolate[@]}"
expect_status 1 "sign with a wrong PIN"
expect_output "wrong PIN" "sign with a wrong PIN"

# ------------------------------------------------------------------- sign ---

step "sign: every hash and algorithm, every signature verified"
run_probe sign --all --hash all --pss --pin-env "$pin_var" "${isolate[@]}"
expect_status 0 "sign"
expect_count '^   OK ' "$expected_signatures" "sign"
expect_count '^   FAIL ' 0 "sign"
expect_output "via C_Sign" "sign"

# ------------------------------------------------------------------ p11-kit ---

step "p11-kit: modules registered in the user directory"
home="$work/home"
modules="$home/.config/pkcs11/modules"
mkdir -p "$modules" "$work/lib"
# A copy of the library is a different file, so it is loaded next to the
# system's own SoftHSM and its certificates are seen twice.
cp "$SOFTHSM_MODULE" "$work/lib/libsofthsm2-registered.so"
echo "module: $work/lib/libsofthsm2-registered.so" >"$modules/websign-test.module"
echo "module: $work/lib/libsofthsm2-missing.so" >"$modules/websign-missing.module"

use_home "$home"
run_probe list --every-path --no-known-modules
restore_home
expect_status 0 "p11-kit discovery"
expect_output "pkcs11:libsofthsm2-registered.so" "p11-kit discovery"
expect_output "warning: pkcs11:libsofthsm2-missing.so: module file not found" "p11-kit broken registration"
expect_count '^ *[0-9]+\. ' "$expected_certificates" "p11-kit discovery (deduplicated by certificate)"

step "p11-kit: the same certificate through two modules is listed once"
proxy=""
for candidate in /usr/lib/*/p11-kit-proxy.so /usr/lib64/p11-kit-proxy.so /usr/lib/p11-kit-proxy.so; do
  if [[ -e $candidate ]]; then proxy=$candidate; break; fi
done
if [[ -z $proxy ]]; then
  echo "  WARNING: p11-kit-proxy.so not found; the two-path deduplication step is skipped."
else
  echo "module: $SOFTHSM_MODULE" >"$modules/softhsm2.module"
  echo "  proxy: $proxy"
  use_home "$home"
  run_probe list --no-known-modules --no-p11-kit --module "$SOFTHSM_MODULE" --module "$proxy"
  expect_status 0 "two-path list"
  expect_count '^ *[0-9]+\. ' "$expected_certificates" "two-path list (each certificate once)"
  expect_count '\+1 other path' "$expected_certificates" "two-path list"

  run_probe list --every-path --no-known-modules --no-p11-kit \
    --module "$SOFTHSM_MODULE" --module "$proxy"
  expect_output "also via pkcs11:p11-kit-proxy.so" "two-path list --every-path"

  step "sign through p11-kit-proxy.so alone"
  run_probe sign --all --hash all --pss --pin-env "$pin_var" \
    --no-known-modules --no-p11-kit --module "$proxy"
  restore_home
  expect_status 0 "sign via p11-kit-proxy"
  expect_count '^   OK ' "$expected_signatures" "sign via p11-kit-proxy"
  expect_count '^   FAIL ' 0 "sign via p11-kit-proxy"
fi

# -------------------------------------------------- always authenticate ---

step "sign: a key that asks for the PIN on every signature (CKA_ALWAYS_AUTHENTICATE)"
main_conf=$SOFTHSM2_CONF
SOFTHSM_ALWAYS_AUTH=1 softhsm_setup "$work/softhsm-aa"
run_probe list "${isolate[@]}"
aa_fingerprint=$(awk -F' [|] ' '/rsa-2048-aa/ { print $NF; exit }' <<<"$last_output" | tr -d ' ')
if [[ -z $aa_fingerprint ]]; then
  fail "always-authenticate: the key is not listed"
else
  run_probe sign --cert "$aa_fingerprint" --hash all --pss --pin-env "$pin_var" "${isolate[@]}"
  ok=$(grep -Ec '^   OK ' <<<"$last_output" || true)
  refused=$(grep -Ec '^   FAIL .*CKA_ALWAYS_AUTHENTICATE' <<<"$last_output" || true)
  # Two acceptable outcomes: the key signs every case (a single-part C_Sign
  # after the context-specific login), or every case is refused with the
  # reason spelled out. Anything else is a bug.
  if ((ok == 6)); then
    echo "  always-authenticate: signs (6 of 6)"
  elif ((refused == 6)); then
    echo "  always-authenticate: refused with an explanation (known limit of the PKCS#11 wrapper)"
  else
    fail "always-authenticate: $ok signed, $refused refused with the reason, 6 expected in one group"
  fi
fi
export SOFTHSM2_CONF=$main_conf SOFTHSM_DIR=$work/softhsm

# ---------------------------------------------------------------- devices ---

step "devices: must not fail without hardware"
run_probe devices
expect_status 0 "devices"
expect_output "PC/SC readers:" "devices"
run_probe devices --json
expect_status 0 "devices --json"
if command -v python3 >/dev/null; then
  python3 -c 'import json, sys; json.load(sys.stdin)' <<<"$last_output" || fail "devices --json is not valid JSON"
fi

# ----------------------------------------------------------------- report ---

step "report: Markdown without personal data"
run_probe report --run-signatures --all --hash all --pss --pin-env "$pin_var" "${isolate[@]}" \
  --out "$report"
expect_status 0 "report"
if [[ -s $report ]]; then
  sed 's/^/  | /' "$report"
  grep -q '^### Devices' "$report" || fail "report has no Devices section"
  grep -q '^### Signatures' "$report" || fail "report has no Signatures section"
  if grep -Fq "websign-test" "$report"; then fail "report contains the token label"; fi
  if grep -Fq "$SOFTHSM_USER_PIN" "$report"; then fail "report contains the PIN"; fi
else
  fail "report was not written to $report"
fi

# ------------------------------------------------- native messaging (e2e) ---

step "native messaging end to end (page -> extension -> host -> SoftHSM)"
e2e="$kit/nm-e2e"
if [[ ! -f $e2e/run.mjs ]]; then
  echo "  WARNING: nm-e2e/run.mjs does not exist; skipping."
elif ! command -v node >/dev/null; then
  echo "  WARNING: node is not installed; skipping."
elif [[ ! -d $e2e/node_modules/playwright-core ]]; then
  echo "  WARNING: run 'npm ci' in nm-e2e first (playwright-core is missing); skipping."
else
  e2e_args=(--probe "$probe" --expect-sign)
  if [[ -n ${CHROMIUM:-} ]]; then e2e_args+=(--chromium "$CHROMIUM"); fi
  printf '\n> node nm-e2e/run.mjs %s\n' "${e2e_args[*]}"
  set +e
  # The host inherits SOFTHSM2_CONF and WEBSIGN_PROBE_PIN through the browser.
  e2e_output=$(cd "$e2e" && node run.mjs "${e2e_args[@]}" 2>&1)
  e2e_status=$?
  set -e
  indent <<<"$e2e_output"
  echo "  (exit code $e2e_status)"
  [[ $e2e_status -eq 0 ]] || fail "native messaging end-to-end test failed"
  grep -Fq "NM-E2E: PASS" <<<"$e2e_output" || fail "native messaging test did not print NM-E2E: PASS"
fi

# ---------------------------------------------------------------- verdict ---

echo
if ((${#failures[@]} > 0)); then
  echo "LINUX PROOF: FAILED (${#failures[@]})"
  printf '  - %s\n' "${failures[@]}"
  exit 1
fi
echo "LINUX PROOF: PASSED"
