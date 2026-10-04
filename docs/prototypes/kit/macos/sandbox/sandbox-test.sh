#!/usr/bin/env bash
# App Sandbox experiment, evidence for docs/prototypes/2-mac.md and
# 3-tokens-mac.md. Wraps websign-probe in an .app signed ad hoc with the Mac
# App Store entitlements and, from inside the sandbox:
#   (a) registers the native messaging host and checks that the manifests
#       reach the browsers' real folders, then talks to the sandboxed host
#       the way Chrome starts it (origin argument, framed JSON on stdio);
#   (b) lists and signs with identities from a test keychain;
#   (c) loads SoftHSM2 as a PKCS#11 module (dlopen, C_Initialize, C_Sign)
#       with its token inside the app's container, signed as the store
#       build would be and then with the hardened runtime of a Developer ID
#       build.
#
# Needs PROBE_EXE. Every result prints as "RESULT <id> SIM|NÃO|N/A <what>".
# The CI step is continue-on-error, so the script runs every experiment and
# exits 1 at the end if any result is NÃO. It restores what it touches:
# browser folders it created, manifests it replaced, the keychain search list
# and, if it created it, the app's container.
#
# An ad hoc signature proves what the sandbox allows for these entitlements.
# It cannot prove what needs Apple's signature: App Review accepting the
# temporary exceptions, the app group (removed here: it needs a provisioning
# profile), the Safari extension, or TestFlight/App Store installs.

set -uo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source-path=SCRIPTDIR source=../lib/common.sh
. "$here/../lib/common.sh"
# shellcheck source-path=SCRIPTDIR source=../lib/bundle.sh
. "$here/../lib/bundle.sh"

: "${PROBE_EXE:?set PROBE_EXE to the websign-probe binary}"

readonly P12_PASSWORD=websign-sandbox
readonly KEYCHAIN_NAME=websign-sandbox.keychain
readonly TOKEN_PIN=1234
readonly TIMEOUT_SECONDS=120
readonly ISOLATE=(--no-known-modules --no-p11-kit)
readonly REGISTER_ARGS=(--browser chrome --browser edge --browser brave --browser firefox)
# (label, folder under ~/Library/Application Support) of the browsers in
# REGISTER_ARGS whose registration is checked; missing folders are created to
# simulate an install, since register only writes for installed browsers.
readonly BROWSERS=(
    "Google Chrome|Google/Chrome"
    "Microsoft Edge|Microsoft Edge"
    "Brave|BraveSoftware/Brave-Browser"
    "Firefox|Mozilla"
)

work=$(mktemp -d)
support="$HOME/Library/Application Support"
results=()
created_dirs=()
created_files=()
backups=()
container_existed=false
sandboxed=false

result() { # ID SIM|NÃO|N/A TEXT
    results+=("$1"$'\t'"$2"$'\t'"$3")
    printf 'RESULT %-22s %-4s %s\n' "$1" "$2" "$3"
}

# Runs a command with a time limit; sets OUT (stdout and stderr) and STATUS.
run() {
    echo "> $*"
    OUT=$(with_timeout "$TIMEOUT_SECONDS" "$@" 2>&1)
    STATUS=$?
    printf '%s\n' "$OUT" | sed 's/^/  /'
    echo "  (exit code $STATUS)"
}

cleanup() {
    local path backup
    for path in ${created_files[@]+"${created_files[@]}"}; do rm -f "$path"; done
    for backup in ${backups[@]+"${backups[@]}"}; do mv -f "$backup.websign-backup" "$backup"; done
    # Deepest first, and only if still empty: never remove a user's folder.
    local i
    for ((i = ${#created_dirs[@]} - 1; i >= 0; i--)); do rmdir "${created_dirs[$i]}" 2>/dev/null; done
    keychain_destroy
    if [[ $container_existed == false && -n ${container:-} ]]; then
        rm -rf "$(dirname "$container")" 2>/dev/null ||
            echo "note: could not remove $(dirname "$container") (container protection)"
    fi
    rm -rf "$work"
}
trap cleanup EXIT

contains() { # VALUE ITEM...
    local value=$1 item
    shift
    for item in "$@"; do [[ $item == "$value" ]] && return 0; done
    return 1
}

# Creates DIR and each missing parent, remembering them for cleanup.
make_dir() { # DIR
    local dir=$1 missing=()
    while [[ ! -d $dir ]]; do
        missing=("$dir" ${missing[@]+"${missing[@]}"})
        dir=$(dirname "$dir")
    done
    for dir in ${missing[@]+"${missing[@]}"}; do
        mkdir "$dir" && created_dirs+=("$dir")
    done
}

build_app() {
    section "Bundle signed ad hoc with the store entitlements"
    # Decided before anything runs sandboxed: cleanup removes the container
    # only if this script caused macOS to create it.
    [[ -d $HOME/Library/Containers/$(project_value macos_bundle_id) ]] && container_existed=true
    if ! build_sandboxed_app "$PROBE_EXE" "$work"; then
        result signed NÃO "could not build or sign the bundle"
        exit 1
    fi
    codesign -d --entitlements - "$app" 2>&1 | sed 's/^/  /'
}

check_sandboxed() {
    section "Sandbox is applied"
    run "$exe" --version
    if [[ $STATUS -eq 0 && -d $container ]]; then
        sandboxed=true
        result sandbox SIM "the probe ran and macOS created its container $container"
    else
        result sandbox NÃO "no container at $container (exit code $STATUS): the rest is not sandboxed"
    fi
}

# Manifest paths `register` would write for REGISTER_ARGS, in the real home:
# asked of the unsandboxed binary, whose $HOME is the user's.
planned_manifests() {
    "$PROBE_EXE" register --dry-run "${REGISTER_ARGS[@]}" 2>/dev/null |
        sed -n 's|^ *dry-run  *[^/]*\(/.*\)$|\1|p'
}

# Moves aside manifests register is about to overwrite and remembers what it
# will create, so cleanup leaves the user's browsers as they were.
protect_manifests() {
    local manifest hosts existing=() planned=()
    while IFS= read -r manifest; do
        [[ -n $manifest ]] || continue
        hosts=$(dirname "$manifest")
        [[ -d $hosts ]] && existing+=("$hosts")
        created_files+=("$manifest")
        if [[ -e $manifest ]]; then
            mv -f "$manifest" "$manifest.websign-backup" && backups+=("$manifest")
        fi
        planned+=("$manifest")
    done < <(planned_manifests)
    for manifest in ${planned[@]+"${planned[@]}"}; do
        hosts=$(dirname "$manifest")
        contains "$hosts" ${existing[@]+"${existing[@]}"} || created_dirs+=("$hosts")
    done
}

# (a) Native messaging manifests written from inside the sandbox.
test_register() {
    section "(a) register, from inside the sandbox"
    local entry label dir manifest name
    name="$(project_value native_host).json"
    for entry in "${BROWSERS[@]}"; do
        IFS='|' read -r label dir <<<"$entry"
        make_dir "$support/$dir"
    done
    protect_manifests

    run "$exe" register "${REGISTER_ARGS[@]}"

    for entry in "${BROWSERS[@]}"; do
        IFS='|' read -r label dir <<<"$entry"
        manifest="$support/$dir/NativeMessagingHosts/$name"
        if [[ -f $manifest ]]; then
            # The path may start with /private/var or /var: compare the end.
            if grep -qF "/SignLocal.app/Contents/MacOS/websign-probe\"" "$manifest"; then
                result "register:$label" SIM "$manifest points to the sandboxed binary"
            else
                result "register:$label" NÃO "$manifest exists but does not point to $exe"
            fi
        elif [[ -f "$container/Library/Application Support/$dir/NativeMessagingHosts/$name" ]]; then
            result "register:$label" NÃO "written inside the container: the probe used the sandbox's \$HOME, not the real home"
        else
            result "register:$label" NÃO "no manifest (see the register output above)"
        fi
    done
}

# Frames each argument as a native messaging message (32-bit length in native
# byte order, little-endian on every Mac, then UTF-8 JSON).
nm_frames() {
    perl -e 'print pack("V", length) . $_ for @ARGV' "$@"
}

# Prints each framed reply on its own line.
nm_replies() {
    perl -e 'while (read(STDIN, my $len, 4) == 4) { my $n = unpack("V", $len);
        read(STDIN, my $body, $n) == $n or last; print "$body\n" }'
}

# (a) The sandboxed binary answering a browser, started as Chrome starts it.
test_host() {
    section "(a) sandboxed host started the way Chrome starts it"
    local origin replies
    origin="chrome-extension://$(project_value dev_id)/"
    echo "> websign-probe $origin  (stdin: ping, list)"
    replies=$(nm_frames '{"v":1,"id":"ping-1","type":"ping"}' '{"v":1,"id":"list-1","type":"list"}' |
        with_timeout "$TIMEOUT_SECONDS" "$exe" "$origin" 2>"$work/host.stderr" | nm_replies)
    printf '%s\n' "$replies" | cut -c1-300 | sed 's/^/  /'
    sed 's/^/  stderr: /' "$work/host.stderr"
    if grep -q '"ping-1"' <<<"$replies" && grep -q '"list-1"' <<<"$replies"; then
        result host-stdio SIM "the sandboxed host answered ping and list over stdio"
    else
        result host-stdio NÃO "missing replies from the sandboxed host"
    fi
    local log
    log=$(find "$container" -name '*-probe-host.log' 2>/dev/null | head -n1)
    [[ -n $log ]] && sed 's/^/  host log: /' "$log" | tail -n 20
}

# (b) Keychain identities seen and used from inside the sandbox.
test_keychain() {
    section "(b) keychain from inside the sandbox"
    keychain_create "$KEYCHAIN_NAME" "$P12_PASSWORD"
    make_identity "$work" "SignLocal Sandbox RSA" rsa2048
    make_identity "$work" "SignLocal Sandbox P-256" p256
    keychain_import "$work/SignLocal Sandbox RSA.p12"
    keychain_import "$work/SignLocal Sandbox P-256.p12"
    keychain_allow "$app" "$PROBE_EXE"
    local rsa ec
    rsa=$(cert_fingerprint "$work/SignLocal Sandbox RSA.cert.pem")
    ec=$(cert_fingerprint "$work/SignLocal Sandbox P-256.cert.pem")

    run "$PROBE_EXE" list "${ISOLATE[@]}"
    local outside=NÃO
    grep -qF "$rsa" <<<"$OUT" && grep -qF "$ec" <<<"$OUT" && outside=SIM

    run "$exe" list "${ISOLATE[@]}"
    if grep -qF "$rsa" <<<"$OUT" && grep -qF "$ec" <<<"$OUT"; then
        result keychain-list SIM "the sandboxed probe lists both test identities"
    else
        result keychain-list NÃO "the sandboxed probe does not list them (outside the sandbox: $outside)"
    fi

    run "$exe" sign --cert "$rsa" --cert "$ec" --hash sha256 --pss "${ISOLATE[@]}"
    local ok
    ok=$(grep -cE '^ +OK .* via SecKeyCreateSignature' <<<"$OUT")
    if [[ $STATUS -eq 0 && $ok -eq 3 ]]; then
        result keychain-sign SIM "PKCS#1 v1.5, PSS and ECDSA signed and verified inside the sandbox"
    elif [[ $STATUS -eq 142 ]]; then
        result keychain-sign NÃO "timed out: a keychain access dialog, most likely"
    else
        result keychain-sign NÃO "$ok of 3 signatures verified (exit code $STATUS)"
    fi
    if grep -q '^warning: macos:ctk' <<<"$OUT"; then
        result ctk-query NÃO "the CryptoTokenKit query failed inside the sandbox"
    else
        result ctk-query SIM "the CryptoTokenKit query ran inside the sandbox (no token on the runner)"
    fi
}

# Installs SoftHSM2 and pkcs11-tool, then creates a token with one RSA
# identity in DIR. Prints nothing on success; sets MODULE.
make_softhsm_token() { # DIR
    local dir=$1 prefix
    HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_INSTALL_CLEANUP=1 brew install softhsm opensc >/dev/null || return 1
    prefix=$(brew --prefix softhsm) || return 1
    MODULE=$prefix/lib/softhsm/libsofthsm2.so
    mkdir -p "$dir/tokens" || return 1
    printf 'directories.tokendir = %s\nobjectstore.backend = file\nlog.level = INFO\n' \
        "$dir/tokens" >"$dir/softhsm2.conf" || return 1
    export SOFTHSM2_CONF=$dir/softhsm2.conf
    softhsm2-util --init-token --free --label websign-sandbox --so-pin 5678 --pin "$TOKEN_PIN" >/dev/null &&
        make_identity "$work" "SignLocal Sandbox PKCS11" rsa2048 &&
        "$OPENSSL" pkcs8 -topk8 -nocrypt -in "$work/SignLocal Sandbox PKCS11.key.pem" -out "$work/p11.key" &&
        "$OPENSSL" x509 -in "$work/SignLocal Sandbox PKCS11.cert.pem" -outform DER -out "$work/p11.der" &&
        softhsm2-util --import "$work/p11.key" --token websign-sandbox --label websign-p11 --id 01 \
            --pin "$TOKEN_PIN" >/dev/null &&
        pkcs11-tool --module "$MODULE" --token-label websign-sandbox --login --pin "$TOKEN_PIN" \
            --write-object "$work/p11.der" --type cert --id 01 --label websign-p11 >/dev/null
}

# (c) A PKCS#11 module loaded by the sandboxed process. SUFFIX names the
# signing variant in the result IDs.
test_pkcs11() { # SUFFIX
    local fingerprint
    fingerprint=$(cert_fingerprint "$work/SignLocal Sandbox PKCS11.cert.pem")
    run "$exe" list --module "$MODULE" "${ISOLATE[@]}"
    if grep -qF "$fingerprint" <<<"$OUT"; then
        result "pkcs11-load:$1" SIM "dlopen and C_Initialize worked; the token's certificate is listed"
    else
        result "pkcs11-load:$1" NÃO "$(grep -m1 -i 'pkcs11' <<<"$OUT" || echo "certificate not listed")"
    fi
    run env WEBSIGN_PIN="$TOKEN_PIN" "$exe" sign --cert "$fingerprint" --pin-env WEBSIGN_PIN \
        --hash sha256 --module "$MODULE" "${ISOLATE[@]}"
    if [[ $STATUS -eq 0 ]] && grep -qE '^ +OK .* via C_Sign' <<<"$OUT"; then
        result "pkcs11-sign:$1" SIM "C_Login and C_Sign worked inside the sandbox"
    else
        result "pkcs11-sign:$1" NÃO "exit code $STATUS"
    fi
}

test_pkcs11_variants() {
    section "(c) PKCS#11 module inside the sandbox"
    local token_dir=$container/websign-softhsm
    if [[ $sandboxed == false ]] || ! make_softhsm_token "$token_dir"; then
        result pkcs11-setup NÃO "could not prepare SoftHSM2 inside the container; retrying outside it"
        token_dir=$work/softhsm
        if ! make_softhsm_token "$token_dir"; then
            result pkcs11-setup NÃO "could not prepare SoftHSM2 at all (brew?)"
            return
        fi
    else
        result pkcs11-setup SIM "SoftHSM2 token created inside the container"
    fi
    echo "module: $MODULE"
    echo "SOFTHSM2_CONF: $SOFTHSM2_CONF"
    test_pkcs11 store

    section "(c) the same with the hardened runtime (Developer ID build)"
    if resign_sandboxed_app --options runtime; then
        test_pkcs11 hardened
    else
        result pkcs11-load:hardened N/A "codesign --options runtime failed"
    fi
}

# Every denial the sandbox logged for the probe during this run.
print_denials() {
    section "Sandbox denials logged during the run"
    log show --style compact --start "$started" \
        --predicate 'sender == "Sandbox" AND eventMessage CONTAINS "websign-probe"' 2>/dev/null |
        tail -n 80 || echo "(log show failed)"
}

summarize() {
    [[ -n ${GITHUB_ACTIONS:-} ]] && echo "::endgroup::"
    local entry id verdict text failed=0
    {
        echo "### macOS App Sandbox (ad hoc signature)"
        echo
        echo "| Experiment | Result | Detail |"
        echo "|---|---|---|"
        for entry in ${results[@]+"${results[@]}"}; do
            IFS=$'\t' read -r id verdict text <<<"$entry"
            echo "| $id | $verdict | ${text//|/\\|} |"
        done
    } | tee -a "${GITHUB_STEP_SUMMARY:-/dev/null}"
    for entry in ${results[@]+"${results[@]}"}; do
        [[ $entry == *$'\t'NÃO$'\t'* ]] && failed=1
    done
    return "$failed"
}

started=$(date '+%Y-%m-%d %H:%M:%S')
section "Environment"
sw_vers
echo "arch: $(uname -m)"
echo "probe: $("$PROBE_EXE" --version)"
find_openssl
build_app
check_sandboxed
test_register
test_host
test_keychain
test_pkcs11_variants
print_denials
summarize
