#!/usr/bin/env bash
# Creates an isolated SoftHSM2 token for the PKCS#11 proofs: its own token
# directory and SOFTHSM2_CONF (the system's SoftHSM configuration is never
# touched), with RSA-2048, EC P-256, P-384 and P-521 keys. Each key and its
# self-signed certificate share a CKA_ID, exactly like on a real token.
#
#   source linux/softhsm-setup.sh [DIR]   exports the variables below
#   eval "$(bash linux/softhsm-setup.sh [DIR])"   the same, without sourcing
#
# DIR defaults to a fresh temporary directory. Running it again on the same
# DIR reuses the token. With SOFTHSM_ALWAYS_AUTH=1 the token also gets an
# RSA-2048 key with CKA_ALWAYS_AUTHENTICATE (the PIN is asked again for every
# signature, as on eIDAS qualified signature keys). Exported:
#
#   SOFTHSM2_CONF      configuration of the isolated token
#   SOFTHSM_MODULE     path of libsofthsm2.so (override with SOFTHSM2_MODULE)
#   SOFTHSM_USER_PIN   user PIN of the test token (a throwaway value)
#   SOFTHSM_DIR        where everything lives; delete it to clean up
#
# Needs softhsm2, opensc (pkcs11-tool) and openssl.

softhsm_setup() {
  local dir=${1:-}
  local pin=246810 so_pin=13579135 label=websign-test

  local tool
  for tool in softhsm2-util pkcs11-tool openssl; do
    command -v "$tool" >/dev/null || { echo "softhsm-setup: $tool is not installed" >&2; return 1; }
  done

  local module
  module=$(softhsm_module) || return 1

  if [[ -z $dir ]]; then
    dir=$(mktemp -d "${TMPDIR:-/tmp}/websign-softhsm.XXXXXX")
  fi
  mkdir -p "$dir/tokens"
  dir=$(cd "$dir" && pwd)

  export SOFTHSM2_CONF="$dir/softhsm2.conf"
  export SOFTHSM_MODULE=$module SOFTHSM_USER_PIN=$pin SOFTHSM_DIR=$dir

  if [[ -f $dir/ready ]]; then
    echo "softhsm-setup: reusing the token in $dir" >&2
    return 0
  fi

  cat >"$SOFTHSM2_CONF" <<CONF
directories.tokendir = $dir/tokens/
objectstore.backend = file
log.level = ERROR
slots.removable = false
CONF

  softhsm2-util --init-token --free --label "$label" --so-pin "$so_pin" --pin "$pin" >/dev/null
  echo "softhsm-setup: token created in $dir" >&2

  # id, label, openssl key options
  _softhsm_import "$dir" 01 rsa-2048 rsa "-algorithm RSA -pkeyopt rsa_keygen_bits:2048"
  _softhsm_import "$dir" 02 ec-p256 ec "-algorithm EC -pkeyopt ec_paramgen_curve:P-256 -pkeyopt ec_param_enc:named_curve"
  _softhsm_import "$dir" 03 ec-p384 ec "-algorithm EC -pkeyopt ec_paramgen_curve:P-384 -pkeyopt ec_param_enc:named_curve"
  _softhsm_import "$dir" 04 ec-p521 ec "-algorithm EC -pkeyopt ec_paramgen_curve:P-521 -pkeyopt ec_param_enc:named_curve"
  if [[ ${SOFTHSM_ALWAYS_AUTH:-0} == 1 ]]; then
    _softhsm_import "$dir" 05 rsa-2048-aa rsa "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" --always-auth
  fi

  touch "$dir/ready"
}

# The SoftHSM2 module of this machine.
softhsm_module() {
  local candidate
  for candidate in \
    "${SOFTHSM2_MODULE:-}" \
    /usr/lib/softhsm/libsofthsm2.so \
    /usr/lib/x86_64-linux-gnu/softhsm/libsofthsm2.so \
    /usr/lib/aarch64-linux-gnu/softhsm/libsofthsm2.so \
    /usr/lib64/softhsm/libsofthsm2.so \
    /usr/local/lib/softhsm/libsofthsm2.so; do
    if [[ -n $candidate && -f $candidate ]]; then
      echo "$candidate"
      return 0
    fi
  done
  echo "softhsm-setup: libsofthsm2.so not found (set SOFTHSM2_MODULE)" >&2
  return 1
}

# _softhsm_import DIR ID NAME KIND OPENSSL_GENPKEY_OPTIONS [PKCS11_TOOL_KEY_OPTIONS...]
# Generates a key and a self-signed signing certificate with openssl and
# stores both on the token under the same CKA_ID.
_softhsm_import() {
  local dir=$1 id=$2 name=$3 kind=$4 options=$5
  shift 5
  local work="$dir/keys/$name"
  mkdir -p "$work"

  # shellcheck disable=SC2086 # the options are a word list on purpose
  openssl genpkey $options -out "$work/key.pem" 2>/dev/null
  openssl req -new -x509 -key "$work/key.pem" -sha256 -days 3650 \
    -subj "/C=BR/O=WebeSign Test/CN=WebeSign Test $name" \
    -addext "basicConstraints=critical,CA:FALSE" \
    -addext "keyUsage=critical,digitalSignature,nonRepudiation" \
    -out "$work/cert.pem" 2>/dev/null
  openssl pkcs8 -topk8 -nocrypt -in "$work/key.pem" -outform DER -out "$work/key.der"
  openssl x509 -in "$work/cert.pem" -outform DER -out "$work/cert.der"

  local common=(--module "$SOFTHSM_MODULE" --token-label websign-test --login --pin "$SOFTHSM_USER_PIN")
  pkcs11-tool "${common[@]}" --write-object "$work/key.der" --type privkey \
    --id "$id" --label "$name" --usage-sign "$@" >/dev/null
  pkcs11-tool "${common[@]}" --write-object "$work/cert.der" --type cert \
    --id "$id" --label "$name" >/dev/null
  echo "softhsm-setup: imported $name (CKA_ID $id, $kind)" >&2
}

# Executed instead of sourced: print the exports so the caller can eval them.
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  set -euo pipefail
  softhsm_setup "${1:-}"
  for variable in SOFTHSM2_CONF SOFTHSM_MODULE SOFTHSM_USER_PIN SOFTHSM_DIR; do
    printf 'export %s=%q\n' "$variable" "${!variable}"
  done
fi
