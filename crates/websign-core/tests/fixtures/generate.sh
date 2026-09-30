#!/usr/bin/env bash
# Regenerates every fixture of the websign-core tests: certificates (DER) and
# reference vectors made by OpenSSL 3. Tests never run OpenSSL; they read the
# committed output. See README.md for what each file is and what to expect.
#
# Usage: tests/fixtures/generate.sh        (needs bash >= 4, openssl >= 3.0, od, GNU date)
#        KEEP_WORKDIR=1 tests/fixtures/generate.sh   keeps keys and CSRs
#
# Keys are random, so each run produces different bytes; every value the tests
# expect comes from this configuration or from generated manifests, never from
# key material.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CERTS=$HERE/certs
VECTORS=$HERE/vectors
WORK=$(mktemp -d)
trap '[ -n "${KEEP_WORKDIR:-}" ] && echo "workdir: $WORK" >&2 || rm -rf "${WORK:?}"' EXIT

mkdir -p "$CERTS" "$VECTORS"
rm -f "$CERTS"/*.der "$VECTORS"/*.txt

for part in lib keys certs-basic certs-fields certs-icp certs-qualified oracles vectors; do
  # shellcheck source=/dev/null
  . "$HERE/gen/$part.sh"
done

openssl version >&2
init_ca_config
gen_keys
gen_root
gen_key_type_certs
gen_extension_certs
gen_serial_and_time_certs
gen_name_certs
gen_malformed_extension_certs
gen_icp_certs
gen_qualified_certs
gen_certificate_oracles
gen_vectors
log "done: $(find "$CERTS" -name '*.der' | wc -l) certificates"
