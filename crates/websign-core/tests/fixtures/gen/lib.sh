# shellcheck shell=bash
# Helpers shared by the fixture generators. Sourced by ../generate.sh, which
# defines WORK (scratch directory), CERTS and VECTORS (output directories).

# Default validity: 2020-01-01T00:00:00Z .. 2040-01-01T00:00:00Z.
V_START=20200101000000Z
V_END=20400101000000Z
ROOT_SUBJECT="/C=BR/O=WebeSign Test Authority/OU=Fixtures Root/CN=WebeSign Test Root CA"

log() { printf '  %s\n' "$*" >&2; }

# Lowercase hex of a file, on one line.
hex_of() { od -An -v -tx1 "$1" | tr -d ' \n'; }

# Left-pads a hex string with zeros up to $2 digits.
pad_hex() {
  local h=$1
  while [ "${#h}" -lt "$2" ]; do h="0$h"; done
  printf '%s' "$h"
}

new_rsa() { openssl genrsa -out "$WORK/keys/$1.pem" "$2" 2>/dev/null; }
new_ec() { openssl ecparam -name "$2" -genkey -noout -out "$WORK/keys/$1.pem"; }
new_pkey() { openssl genpkey -algorithm "$2" -out "$WORK/keys/$1.pem" "${@:3}" 2>/dev/null; }

init_ca_config() {
  mkdir -p "$WORK/ca/out" "$WORK/keys" "$WORK/csr"
  cat >"$WORK/ca/ca.cnf" <<EOF
[ca]
default_ca = CA_default
[CA_default]
database = $WORK/ca/index.txt
new_certs_dir = $WORK/ca/out
serial = $WORK/ca/serial
default_md = sha256
policy = allow_all
unique_subject = no
copy_extensions = none
preserve = yes
[allow_all]
countryName = optional
stateOrProvinceName = optional
localityName = optional
organizationName = optional
organizationalUnitName = optional
commonName = optional
givenName = optional
surname = optional
serialNumber = optional
emailAddress = optional
EOF
}

# Writes a CSR configuration whose string_mask forces one ASN.1 string type.
# 0x2000 is UTF8String, the other bits are documented in README.md.
write_req_config() {
  cat >"$WORK/csr/req.cnf" <<EOF
[req]
distinguished_name = dn
default_md = sha256
string_mask = ${STRING_MASK:-utf8only}
[dn]
EOF
}

# issue NAME KEY SUBJECT SERIAL_HEX [START END [self]]
#
# Signs a certificate for KEY with the fixture root (or with KEY itself when
# the last argument is "self") and writes certs/NAME.der. The body of the
# [ext] section, plus any extra config sections, comes from stdin; an empty
# stdin produces a version 1 certificate without extensions.
#
# STRING_MASK forces the ASN.1 string type of the subject's attributes.
issue() {
  local name=$1 key=$2 subject=$3 serial=$4
  local start=${5:-$V_START} end=${6:-$V_END} mode=${7:-root}
  local csr="$WORK/csr/$name.csr" ext="$WORK/csr/$name.ext"

  write_req_config
  openssl req -new -utf8 -config "$WORK/csr/req.cnf" -key "$WORK/keys/$key.pem" \
    -subj "$subject" -out "$csr"

  { echo "[ext]"; cat; } >"$ext"
  local ext_args=()
  if [ "$(wc -l <"$ext")" -gt 1 ]; then ext_args=(-extfile "$ext" -extensions ext); fi

  local signer=(-cert "$WORK/ca/root.pem" -keyfile "$WORK/keys/root.pem")
  if [ "$mode" = self ]; then signer=(-selfsign -keyfile "$WORK/keys/$key.pem"); fi

  : >"$WORK/ca/index.txt"
  printf '%s\n' "$serial" >"$WORK/ca/serial"
  openssl ca -config "$WORK/ca/ca.cnf" -batch -notext -preserveDN "${signer[@]}" \
    -in "$csr" -startdate "$start" -enddate "$end" "${ext_args[@]}" \
    -out "$WORK/ca/$name.pem" 2>"$WORK/ca/$name.log" \
    || { cat "$WORK/ca/$name.log" >&2; return 1; }
  openssl x509 -in "$WORK/ca/$name.pem" -outform DER -out "$CERTS/$name.der"
}

# patch_cert NAME OLD_HEX NEW_HEX
#
# Rewrites bytes of certs/NAME.der in place (same length; the pattern must
# exist). Used for ASN.1 string types OpenSSL refuses to put in a subject.
# The signature no longer matches, which CertInfo never checks.
patch_cert() {
  local file="$CERTS/$1.der" old=$2 new=$3 hex escaped
  [ "${#old}" -eq "${#new}" ] || { echo "patch_cert: length differs" >&2; return 1; }
  hex=$(hex_of "$file")
  case $hex in *"$old"*) ;; *) echo "patch_cert: pattern not in $1" >&2; return 1 ;; esac
  hex=${hex//"$old"/"$new"}
  escaped=$(printf '%s' "$hex" | sed 's/../\\x&/g')
  # shellcheck disable=SC2059
  printf "$escaped" >"$file.tmp"
  mv "$file.tmp" "$file"
}

# ascii_hex TEXT -> hex of the ASCII bytes
ascii_hex() { printf '%s' "$1" | od -An -v -tx1 | tr -d ' \n'; }

# retag_string NAME ATTR_OID_TLV TEXT OLD_TAG NEW_TAG
# Changes the string tag of the attribute whose value is exactly TEXT.
retag_string() {
  local len
  len=$(printf '%02x' "${#3}")
  patch_cert "$1" "$2$4$len$(ascii_hex "$3")" "$2$5$len$(ascii_hex "$3")"
}

# Running serial (hex) for certificates whose serial does not matter. Kept in
# a file because callers use it inside command substitutions.
next_serial() {
  local counter="$WORK/serial.counter" n=4096
  [ -f "$counter" ] && n=$(<"$counter")
  echo $((n + 1)) >"$counter"
  printf '%x' "$n"
}

# leaf_subject NAME [CN] -- the subject every ordinary fixture certificate uses.
leaf_subject() {
  printf '/C=BR/O=WebeSign Test Fixtures/OU=Unit A/OU=Unit B/CN=%s' "${2:-Fixture $1}"
}

# Extensions of an ordinary end-entity certificate: digitalSignature +
# nonRepudiation, e-mail protection and client authentication.
std_ext() {
  cat <<EOF
basicConstraints=CA:FALSE
keyUsage=critical,digitalSignature,nonRepudiation
extendedKeyUsage=emailProtection,clientAuth
EOF
}

# leaf NAME KEY [CN] -- ordinary end-entity certificate for KEY.
leaf() {
  issue "$1" "$2" "$(leaf_subject "$1" "${3:-}")" "$(next_serial)" < <(std_ext)
}

# variant NAME SUBJECT -- certificate with the shared p256 key and the
# given subject; extensions come from stdin.
variant() {
  issue "$1" p256 "$2" "$(next_serial)"
}
