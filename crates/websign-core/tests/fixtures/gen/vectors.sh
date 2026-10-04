# shellcheck shell=bash
# Reference vectors, all produced by OpenSSL: digests, PKCS#1 DigestInfo,
# signatures for every hash x algorithm x key kind and ECDSA DER/raw pairs.

MESSAGE="SignLocal probe-core fixture message"
HASHES="sha256 sha384 sha512"

digest_file() { echo "$WORK/digest-$1.bin"; }

pss_options() { # HASH [SALT_LEN [MGF1_HASH]]
  local salt=${2:-$(($(wc -c <"$(digest_file "$1")")))}
  printf -- '-pkeyopt rsa_padding_mode:pss -pkeyopt digest:%s -pkeyopt rsa_pss_saltlen:%s -pkeyopt rsa_mgf1_md:%s' \
    "$1" "$salt" "${3:-$1}"
}

# sign_rsa KEY HASH pkcs1|pss [SALT_LEN [MGF1_HASH]] -> hex of the signature
sign_rsa() {
  local key=$1 hash=$2 mode=$3 out="$WORK/sig.bin"
  local opts=(-pkeyopt "digest:$hash")
  # shellcheck disable=SC2207
  if [ "$mode" = pss ]; then opts=($(pss_options "$hash" "${4:-}" "${5:-}")); fi
  openssl pkeyutl -sign -inkey "$WORK/keys/$key.pem" -in "$(digest_file "$hash")" \
    "${opts[@]}" -out "$out"
  hex_of "$out"
}

# der_to_raw_hex DER_FILE FIELD_LEN -> raw r||s as hex, read back by OpenSSL's
# own ASN.1 parser (so the expected raw form does not come from this crate).
der_to_raw_hex() {
  local ints i out=""
  mapfile -t ints < <(openssl asn1parse -inform DER -in "$1" | sed -n 's/.*INTEGER *://p')
  for i in 0 1; do
    local v=${ints[$i]}
    v=$(printf '%s' "$v" | tr 'A-F' 'a-f')
    while [ "${v:0:2}" = 00 ] && [ "${#v}" -gt 2 ]; do v=${v:2}; done
    out+=$(pad_hex "$v" $(($2 * 2)))
  done
  printf '%s' "$out"
}

# sign_ecdsa KEY HASH -> writes sig.der, prints nothing
sign_ecdsa() {
  openssl pkeyutl -sign -inkey "$WORK/keys/$1.pem" -in "$(digest_file "$2")" -out "$WORK/sig.der"
}

gen_digests() {
  local h
  {
    echo "# hash hex(digest of \"$MESSAGE\")"
    for h in $HASHES; do
      printf '%s' "$MESSAGE" | openssl dgst "-$h" -binary >"$(digest_file "$h")"
      echo "$h $(hex_of "$(digest_file "$h")")"
    done
  } >"$VECTORS/digests.txt"
}

# DigestInfo built by OpenSSL's ASN.1 generator from the hash OIDs, then
# checked against a "raw" PKCS#1 signature over it, which must equal the
# signature OpenSSL makes when it wraps the digest itself.
gen_digest_info() {
  local h oid cfg="$WORK/digestinfo.cnf" out="$WORK/digestinfo.bin"
  {
    echo "# hash hex(DigestInfo of the digest in digests.txt)"
    for h in $HASHES; do
      case $h in
        sha256) oid=2.16.840.1.101.3.4.2.1 ;;
        sha384) oid=2.16.840.1.101.3.4.2.2 ;;
        sha512) oid=2.16.840.1.101.3.4.2.3 ;;
      esac
      cat >"$cfg" <<EOF
asn1=SEQUENCE:info
[info]
algorithm=SEQUENCE:algorithm
digest=FORMAT:HEX,OCTETSTRING:$(hex_of "$(digest_file "$h")")
[algorithm]
oid=OID:$oid
params=NULL:
EOF
      openssl asn1parse -genconf "$cfg" -noout -out "$out"
      # rsautl is the one OpenSSL command that pads arbitrary input as-is
      # (pkeyutl refuses input longer than a hash); its deprecation notice is noise.
      openssl rsautl -sign -pkcs -inkey "$WORK/keys/rsa2048.pem" -in "$out" \
        -out "$WORK/raw.sig" 2>/dev/null
      [ "$(hex_of "$WORK/raw.sig")" = "$(sign_rsa rsa2048 "$h" pkcs1)" ] \
        || { echo "DigestInfo for $h disagrees with OpenSSL" >&2; return 1; }
      echo "$h $(hex_of "$out")"
    done
  } >"$VECTORS/digestinfo.txt"
}

gen_signatures() {
  local key h
  {
    echo "# key algorithm hash hex(signature over the digest in digests.txt)"
    echo "# pkcs1 / pss: RSA block; ecdsa: raw r||s. Salt length = digest length."
    for key in rsa2048 rsa2048b rsa3072 rsa4096 rsa2047; do
      for h in $HASHES; do
        echo "$key pkcs1 $h $(sign_rsa "$key" "$h" pkcs1)"
        echo "$key pss $h $(sign_rsa "$key" "$h" pss)"
      done
    done
    for h in $HASHES; do echo "rsapss2048 pss $h $(sign_rsa rsapss2048 "$h" pss)"; done
    for key in p256:32 p256b:32 p384:48 p521:66; do
      for h in $HASHES; do
        sign_ecdsa "${key%%:*}" "$h"
        echo "${key%%:*} ecdsa $h $(der_to_raw_hex "$WORK/sig.der" "${key##*:}")"
      done
    done
    # Signatures a strict verifier must refuse: wrong salt length, wrong MGF1 hash.
    for h in $HASHES; do
      echo "rsa2048 pss-salt-0 $h $(sign_rsa rsa2048 "$h" pss 0)"
      echo "rsa2048 pss-salt-20 $h $(sign_rsa rsa2048 "$h" pss 20)"
      echo "rsa2048 pss-mgf1-sha1 $h $(sign_rsa rsa2048 "$h" pss '' sha1)"
    done
  } >"$VECTORS/signatures.txt"
}

# Random-looking (r, s) pairs never repeat, so sign the same digest until each
# shape of interest shows up: r or s with a leading zero byte (shorter than the
# field), r or s with the top bit set (DER needs a 00 pad).
gen_ecdsa_vectors() {
  local spec curve key field hash want attempt raw r s shape
  {
    echo "# curve shape hash hex(raw r||s) hex(DER); digest in digests.txt"
    for spec in p256:p256:32:sha256 p384:p384:48:sha384 p521:p521:66:sha512; do
      IFS=: read -r curve key field hash <<<"$spec"
      declare -A found=()
      want=6
      for attempt in $(seq 1 20000); do
        sign_ecdsa "$key" "$hash"
        raw=$(der_to_raw_hex "$WORK/sig.der" "$field")
        r=${raw:0:$((field * 2))}
        s=${raw:$((field * 2))}
        for shape in $(ecdsa_shapes "$r" "$s"); do
          if [ -z "${found[$shape]:-}" ]; then
            found[$shape]=1
            echo "$curve $shape $hash $raw $(hex_of "$WORK/sig.der")"
          fi
        done
        [ "${#found[@]}" -ge "$want" ] && break
      done
      [ "${#found[@]}" -ge "$want" ] || { echo "missing ECDSA shapes for $curve" >&2; return 1; }
      unset found
    done
  } >"$VECTORS/ecdsa.txt"
}

# ecdsa_shapes R_HEX S_HEX -> every shape the pair exhibits
ecdsa_shapes() {
  local r=$1 s=$2 first_r first_s high_r=0 high_s=0
  first_r=$(strip_zero_bytes "$r")
  first_s=$(strip_zero_bytes "$s")
  [[ ${first_r:0:1} =~ [89a-f] ]] && high_r=1
  [[ ${first_s:0:1} =~ [89a-f] ]] && high_s=1
  [ "${r:0:2}" = 00 ] && echo short-r
  [ "${s:0:2}" = 00 ] && echo short-s
  [ $high_r = 1 ] && echo high-r
  [ $high_s = 1 ] && echo high-s
  [ $high_r = 1 ] && [ $high_s = 1 ] && echo high-both
  [[ ${r:0:2} =~ ^(0[1-9a-f]|[1-7][0-9a-f])$ ]] && [[ ${s:0:2} =~ ^(0[1-9a-f]|[1-7][0-9a-f])$ ]] && echo plain
  return 0
}

strip_zero_bytes() {
  local v=$1
  while [ "${v:0:2}" = 00 ] && [ "${#v}" -gt 2 ]; do v=${v:2}; done
  printf '%s' "$v"
}

gen_vectors() {
  log "vectors"
  gen_digests
  gen_digest_info
  gen_signatures
  gen_ecdsa_vectors
}
