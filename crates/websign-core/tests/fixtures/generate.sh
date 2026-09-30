#!/usr/bin/env bash
# Regenerates every fixture of the websign-core tests: certificates (DER) and
# reference vectors made by OpenSSL 3. Tests never run OpenSSL; they read the
# committed output. See README.md for what each file is and what to expect.
#
# Usage: tests/fixtures/generate.sh        (needs bash >= 4, openssl >= 3.0, od, GNU date)
#        KEEP_WORKDIR=1 tests/fixtures/generate.sh   keeps keys and CSRs
#        ADDITIVE=1 tests/fixtures/generate.sh       only the Brainpool and personal-name
#                                                    fixtures, next to the committed ones
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
[ -n "${ADDITIVE:-}" ] || rm -f "$CERTS"/*.der "$VECTORS"/*.txt

for part in lib keys brainpool certs-basic certs-fields certs-personal certs-icp certs-qualified oracles vectors; do
  # shellcheck source=/dev/null
  . "$HERE/gen/$part.sh"
done

openssl version >&2
init_ca_config

if [ -n "${ADDITIVE:-}" ]; then
  # The throwaway root and p256 key are new, so the issuer's key differs from
  # `ca.der`'s; nothing checks certificate signatures. `ca.der` is not touched.
  new_ec root prime256v1
  new_ec p256 prime256v1
  REAL_CERTS=$CERTS
  CERTS=$WORK/discard
  mkdir -p "$CERTS"
  gen_root
  CERTS=$REAL_CERTS
  gen_brainpool_keys
  gen_brainpool_certs
  gen_personal_name_certs
  gen_certificate_oracles
  gen_digests
  gen_brainpool_vectors
  log "done (additive)"
  exit 0
fi

gen_keys
gen_brainpool_keys
gen_root
gen_key_type_certs
gen_brainpool_certs
gen_extension_certs
gen_serial_and_time_certs
gen_name_certs
gen_personal_name_certs
gen_malformed_extension_certs
gen_icp_certs
gen_qualified_certs
gen_certificate_oracles
gen_vectors
gen_brainpool_vectors
log "done: $(find "$CERTS" -name '*.der' | wc -l) certificates"
