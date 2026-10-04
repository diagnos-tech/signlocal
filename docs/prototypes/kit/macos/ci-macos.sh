#!/usr/bin/env bash
# Proof of the macOS keychain path with software keys: websign-probe lists
# and signs with RSA-2048, EC P-256 and EC P-384 identities imported into a
# throwaway keychain, and every outcome is checked against what is expected.
#
# Needs PROBE_EXE (the websign-probe binary). When REPORT_PATH is set, also
# writes the probe's Markdown report there. The keychain and the change to
# the keychain search list are undone on exit, also after a failure.
#
# Required (the script fails otherwise):
#   - `list` shows every test identity from macos:keychain, as a software
#     key whose PIN or password the OS asks for;
#   - RSA signs RSASSA-PKCS1-v1_5 and RSASSA-PSS, EC signs ECDSA, each with
#     SHA-256, SHA-384 and SHA-512, through SecKeyCreateSignature, and every
#     signature verifies (the probe checks it with the certificate).
# Observed only: the CryptoTokenKit source (the runner has no token, so it
# should list nothing and report no error).
#
# Signing selects the test certificates with --cert rather than --all: the
# System keychain of every Mac holds identities (com.apple.systemdefault,
# com.apple.kerberos.kdc) whose keys only system services may use, and
# signing with them opens a password dialog that nobody answers on a runner.
#
# PKCS#11 discovery is switched off: this proves the keychain; PKCS#11 inside
# the App Sandbox is sandbox/sandbox-test.sh's job.

set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source-path=SCRIPTDIR source=lib/common.sh
. "$here/lib/common.sh"

: "${PROBE_EXE:?set PROBE_EXE to the websign-probe binary}"
[[ -x $PROBE_EXE ]] || {
    echo "not executable: $PROBE_EXE" >&2
    exit 1
}

readonly P12_PASSWORD=websign-ci
readonly KEYCHAIN_NAME=websign-ci.keychain
readonly TIMEOUT_SECONDS=120
readonly ISOLATE=(--no-known-modules --no-p11-kit)
# name, key type, signatures expected with --hash all --pss
readonly IDENTITIES=(
    "SignLocal CI RSA-2048|rsa2048|6"
    "SignLocal CI P-256|p256|3"
    "SignLocal CI P-384|p384|3"
)

work=$(mktemp -d)
failures=()
names=()
fingerprints=()
expected_counts=()

cleanup() {
    keychain_destroy
    rm -rf "$work"
}
trap cleanup EXIT

fail() {
    failures+=("$1")
    echo "  !! $1"
}

# Runs the probe with a time limit; sets OUT (stdout and stderr) and STATUS.
probe() {
    echo "> websign-probe $*"
    set +e
    OUT=$(with_timeout "$TIMEOUT_SECONDS" "$PROBE_EXE" "$@" 2>&1)
    STATUS=$?
    set -e
    printf '%s\n' "$OUT" | sed 's/^/  /'
    echo "  (exit code $STATUS)"
    if [[ $STATUS -eq 142 ]]; then
        fail "websign-probe $1 timed out after ${TIMEOUT_SECONDS}s (a keychain dialog?)"
    fi
}

print_environment() {
    section "Environment"
    sw_vers
    echo "arch: $(uname -m)"
    echo "probe: $("$PROBE_EXE" --version)"
    codesign -dv "$PROBE_EXE" 2>&1 | grep -E '^(Identifier|Format|CodeDirectory|Signature)' || true
    find_openssl
}

prepare_keychain() {
    section "Test identities in a throwaway keychain"
    keychain_create "$KEYCHAIN_NAME" "$P12_PASSWORD"
    local entry name key count
    for entry in "${IDENTITIES[@]}"; do
        IFS='|' read -r name key count <<<"$entry"
        make_identity "$work" "$name" "$key"
        keychain_import "$work/$name.p12"
        names+=("$name")
        fingerprints+=("$(cert_fingerprint "$work/$name.cert.pem")")
        expected_counts+=("$count")
        echo "imported $name (${fingerprints[${#fingerprints[@]} - 1]})"
    done
    keychain_allow "$PROBE_EXE"
    security find-identity "$KEYCHAIN_PATH"
}

test_list() {
    section "list"
    probe list "${ISOLATE[@]}"
    [[ $STATUS -eq 0 ]] || fail "list exited with $STATUS"
    local i line
    for i in "${!names[@]}"; do
        line=$(grep -F "${fingerprints[$i]}" <<<"$OUT" | head -n1 || true)
        if [[ -z $line ]]; then
            fail "list does not show ${names[$i]}"
        elif [[ $line != *"macos:keychain (keychain, software; PIN by OS)"* ]]; then
            fail "list shows ${names[$i]} with the wrong source: $line"
        fi
    done
    if grep -q '^warning: macos:ctk' <<<"$OUT"; then
        warn "the CryptoTokenKit source reported an error (see the list output)"
    fi
}

test_sign() {
    section "sign"
    local i ok
    for i in "${!names[@]}"; do
        probe sign --cert "${fingerprints[$i]}" --hash all --pss "${ISOLATE[@]}"
        [[ $STATUS -eq 0 ]] || fail "sign ${names[$i]} exited with $STATUS"
        ok=$(grep -cE '^ +OK +SHA-[0-9]+ [A-Za-z0-9_.-]+ via SecKeyCreateSignature in [0-9]+ ms$' \
            <<<"$OUT" || true)
        [[ $ok -eq ${expected_counts[$i]} ]] ||
            fail "${names[$i]}: $ok verified signatures via SecKeyCreateSignature, expected ${expected_counts[$i]}"
    done
}

# Warning only: the report depends on sections owned by other commands.
write_report() {
    [[ -n ${REPORT_PATH:-} ]] || return 0
    section "report"
    local selection=() fingerprint
    for fingerprint in "${fingerprints[@]}"; do
        selection+=(--cert "$fingerprint")
    done
    probe report --run-signatures "${selection[@]}" --hash all --pss "${ISOLATE[@]}" \
        --out "$REPORT_PATH"
    [[ -f $REPORT_PATH ]] || warn "report did not write $REPORT_PATH (exit code $STATUS)"
}

summarize() {
    [[ -n ${GITHUB_ACTIONS:-} ]] && echo "::endgroup::"
    local verdict="All required checks passed."
    if [[ ${#failures[@]} -gt 0 ]]; then
        verdict="${#failures[@]} required check(s) failed:"
    fi
    {
        echo "### macOS keychain"
        echo
        echo "$verdict"
        local failure
        for failure in ${failures[@]+"${failures[@]}"}; do
            echo "- $failure"
        done
    } | tee -a "${GITHUB_STEP_SUMMARY:-/dev/null}"
}

print_environment
prepare_keychain
test_list
test_sign
write_report
summarize
[[ ${#failures[@]} -eq 0 ]]
