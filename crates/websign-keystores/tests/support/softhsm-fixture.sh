#!/usr/bin/env bash
# Creates an isolated SoftHSM2 token for the PKCS#11 contract tests, in DIR
# (its own token directory and softhsm2.conf; the system configuration is
# never touched).
#
#   bash softhsm-fixture.sh DIR
#
# The token holds, each key next to its certificate under one CKA_ID:
#
#   keys/rsa-2048      RSA-2048, self-signed
#   keys/ec-p256|p384|p521
#   keys/bp-256|bp-384 brainpoolP256r1/P384r1, when this SoftHSM build takes them
#   keys/rsa-aa        RSA-2048 with CKA_ALWAYS_AUTHENTICATE
#   keys/rsa-issued    RSA-2048 issued by ca/intermediate, issued by ca/root;
#                      both CA certificates are stored on the token without keys
#
# Every created key leaves keys/<name>/cert.der; the CA certificates are
# ca/root.der and ca/intermediate.der. DIR/fixture.conf holds KEY=VALUE
# lines (values unquoted): SOFTHSM2_CONF, SOFTHSM_MODULE, SOFTHSM_USER_PIN,
# SOFTHSM_SO_PIN, SOFTHSM_TOKEN_LABEL, SOFTHSM_TOKEN_SERIAL.
# Needs softhsm2, opensc (pkcs11-tool) and openssl; exits 2 when one is missing.

set -euo pipefail

readonly PIN=246810 SO_PIN=13579135 LABEL="websign contract"

main() {
  local dir=${1:?usage: softhsm-fixture.sh DIR}
  local tool
  for tool in softhsm2-util pkcs11-tool openssl; do
    command -v "$tool" >/dev/null || { echo "softhsm-fixture: $tool is not installed" >&2; exit 2; }
  done
  MODULE=$(find_module)
  mkdir -p "$dir/tokens" "$dir/keys" "$dir/ca"
  DIR=$(cd "$dir" && pwd)
  export SOFTHSM2_CONF="$DIR/softhsm2.conf"
  cat >"$SOFTHSM2_CONF" <<CONF
directories.tokendir = $DIR/tokens/
objectstore.backend = file
log.level = ERROR
slots.removable = false
CONF
  softhsm2-util --init-token --free --label "$LABEL" --so-pin "$SO_PIN" --pin "$PIN" >/dev/null

  self_signed rsa-2048 01 "-algorithm RSA -pkeyopt rsa_keygen_bits:2048"
  self_signed ec-p256 02 "$(ec_options P-256)"
  self_signed ec-p384 03 "$(ec_options P-384)"
  self_signed ec-p521 04 "$(ec_options P-521)"
  self_signed bp-256 05 "$(ec_options brainpoolP256r1)" || skipped bp-256
  self_signed bp-384 06 "$(ec_options brainpoolP384r1)" || skipped bp-384
  self_signed rsa-aa 07 "-algorithm RSA -pkeyopt rsa_keygen_bits:2048" --always-auth
  issued_chain

  local serial
  serial=$(softhsm2-util --show-slots | awk -F': *' '/Serial number/ && $2 != "" {print $2; exit}')
  {
    echo "SOFTHSM2_CONF=$SOFTHSM2_CONF"
    echo "SOFTHSM_MODULE=$MODULE"
    echo "SOFTHSM_USER_PIN=$PIN"
    echo "SOFTHSM_SO_PIN=$SO_PIN"
    echo "SOFTHSM_TOKEN_LABEL=$LABEL"
    echo "SOFTHSM_TOKEN_SERIAL=$serial"
  } >"$DIR/fixture.conf"
}

find_module() {
  local candidate
  for candidate in \
    "${SOFTHSM2_MODULE:-}" \
    /usr/lib/softhsm/libsofthsm2.so \
    /usr/lib/x86_64-linux-gnu/softhsm/libsofthsm2.so \
    /usr/lib/aarch64-linux-gnu/softhsm/libsofthsm2.so \
    /usr/lib64/softhsm/libsofthsm2.so \
    /usr/lib64/pkcs11/libsofthsm2.so \
    /usr/local/lib/softhsm/libsofthsm2.so \
    /opt/homebrew/lib/softhsm/libsofthsm2.so; do
    if [[ -n $candidate && -f $candidate ]]; then
      echo "$candidate"
      return 0
    fi
  done
  echo "softhsm-fixture: libsofthsm2.so not found (set SOFTHSM2_MODULE)" >&2
  exit 2
}

ec_options() {
  echo "-algorithm EC -pkeyopt ec_paramgen_curve:$1 -pkeyopt ec_param_enc:named_curve"
}

skipped() {
  rm -rf "$DIR/keys/$1"
  echo "softhsm-fixture: $1 not supported here, skipped" >&2
}

# self_signed NAME ID OPENSSL_GENPKEY_OPTIONS [PKCS11_TOOL_KEY_OPTIONS...]
self_signed() {
  local name=$1 id=$2 options=$3
  shift 3
  local work="$DIR/keys/$name"
  mkdir -p "$work"
  # shellcheck disable=SC2086 # the options are a word list on purpose
  openssl genpkey $options -out "$work/key.pem" 2>/dev/null
  openssl req -new -x509 -key "$work/key.pem" -sha256 -days 3650 \
    -subj "/C=BR/O=WebSign Contract/CN=Maria Contract Holder $name" \
    -addext "basicConstraints=critical,CA:FALSE" \
    -addext "keyUsage=critical,digitalSignature,nonRepudiation" \
    -out "$work/cert.pem" 2>/dev/null
  store "$work" "$id" "$name" "$@"
}

# Root CA -> intermediate CA -> leaf; the CA certificates go on the token
# without their keys, as ICP-Brasil and eIDAS tokens ship them.
issued_chain() {
  local ca="$DIR/ca" work="$DIR/keys/rsa-issued"
  mkdir -p "$work"
  openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256 -out "$ca/root.key" 2>/dev/null
  openssl req -new -x509 -key "$ca/root.key" -sha256 -days 3650 \
    -subj "/C=BR/O=WebSign Contract/CN=Contract Root CA" \
    -addext "basicConstraints=critical,CA:TRUE" \
    -addext "keyUsage=critical,keyCertSign,cRLSign" \
    -out "$ca/root.pem" 2>/dev/null
  openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256 -out "$ca/intermediate.key" 2>/dev/null
  openssl req -new -key "$ca/intermediate.key" -subj "/C=BR/O=WebSign Contract/CN=Contract Issuing CA" \
    -out "$ca/intermediate.csr" 2>/dev/null
  printf 'basicConstraints=critical,CA:TRUE,pathlen:0\nkeyUsage=critical,keyCertSign,cRLSign\n' >"$ca/ca.ext"
  openssl x509 -req -in "$ca/intermediate.csr" -CA "$ca/root.pem" -CAkey "$ca/root.key" \
    -CAcreateserial -sha256 -days 3650 -extfile "$ca/ca.ext" -out "$ca/intermediate.pem" 2>/dev/null

  openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$work/key.pem" 2>/dev/null
  openssl req -new -key "$work/key.pem" -subj "/C=BR/O=WebSign Contract/CN=Maria Contract Holder rsa-issued" \
    -out "$work/req.csr" 2>/dev/null
  printf 'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,nonRepudiation\n' >"$work/leaf.ext"
  openssl x509 -req -in "$work/req.csr" -CA "$ca/intermediate.pem" -CAkey "$ca/intermediate.key" \
    -CAcreateserial -sha256 -days 3650 -extfile "$work/leaf.ext" -out "$work/cert.pem" 2>/dev/null
  store "$work" 08 rsa-issued

  local which
  for which in root intermediate; do
    openssl x509 -in "$ca/$which.pem" -outform DER -out "$ca/$which.der"
    pkcs11 --write-object "$ca/$which.der" --type cert --label "$which" >/dev/null
  done
}

# store WORK ID LABEL [PKCS11_TOOL_KEY_OPTIONS...]
store() {
  local work=$1 id=$2 label=$3
  shift 3
  openssl pkcs8 -topk8 -nocrypt -in "$work/key.pem" -outform DER -out "$work/key.der"
  openssl x509 -in "$work/cert.pem" -outform DER -out "$work/cert.der"
  pkcs11 --write-object "$work/key.der" --type privkey --id "$id" --label "$label" --usage-sign "$@" >/dev/null || return 1
  pkcs11 --write-object "$work/cert.der" --type cert --id "$id" --label "$label" >/dev/null
}

pkcs11() {
  pkcs11-tool --module "$MODULE" --token-label "$LABEL" --login --pin "$PIN" "$@" 2>/dev/null
}

main "$@"
