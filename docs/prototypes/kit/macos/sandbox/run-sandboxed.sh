#!/usr/bin/env bash
# Runs one websign-probe command inside the App Sandbox, for manual tests on
# a real Mac with real tokens and browsers:
#
#   PROBE_EXE=target/release/websign-probe \
#     bash macos/sandbox/run-sandboxed.sh list --module /usr/local/lib/libeTPkcs11.dylib
#
# The bundle is kept in WEBSIGN_SANDBOX_DIR (default
# ~/Library/Caches/dev.websign.sandbox-test) so that a browser can start it
# after `register`. Set HARDENED=1 to sign it like a Developer ID build.
# Sandbox denials of the run are printed at the end.

set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source-path=SCRIPTDIR source=../lib/common.sh
. "$here/../lib/common.sh"
# shellcheck source-path=SCRIPTDIR source=../lib/bundle.sh
. "$here/../lib/bundle.sh"

: "${PROBE_EXE:?set PROBE_EXE to the websign-probe binary}"
dir=${WEBSIGN_SANDBOX_DIR:-$HOME/Library/Caches/dev.websign.sandbox-test}
flags=()
[[ ${HARDENED:-0} == 1 ]] && flags=(--options runtime)

build_sandboxed_app "$PROBE_EXE" "$dir" ${flags[@]+"${flags[@]}"} >&2
echo "bundle: $app" >&2
echo "container: $container" >&2

started=$(date '+%Y-%m-%d %H:%M:%S')
status=0
"$exe" "$@" || status=$?

echo "--- sandbox denials since $started ---" >&2
log show --style compact --start "$started" \
    --predicate 'sender == "Sandbox" AND eventMessage CONTAINS "websign-probe"' 2>/dev/null |
    tail -n 50 >&2 || true
exit "$status"
