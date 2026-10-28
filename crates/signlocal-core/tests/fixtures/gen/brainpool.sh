# shellcheck shell=bash
# Brainpool keys, certificates and OpenSSL signatures (SPEC §4.1). The twisted
# `t1` curves exist to prove they stay unknown.

BRAINPOOL_CURVES="brainpoolP256r1:32:sha256 brainpoolP384r1:48:sha384 brainpoolP512r1:64:sha512"
BRAINPOOL_TWISTED="brainpoolP256t1 brainpoolP384t1 brainpoolP512t1"

gen_brainpool_keys() {
  log "brainpool keys"
  local spec curve
  for spec in $BRAINPOOL_CURVES; do new_ec "${spec%%:*}" "${spec%%:*}"; done
  for curve in $BRAINPOOL_TWISTED; do new_ec "$curve" "$curve"; done
}

gen_brainpool_certs() {
  log "brainpool certificates"
  local spec curve
  for spec in $BRAINPOOL_CURVES; do leaf "${spec%%:*}" "${spec%%:*}"; done
  for curve in $BRAINPOOL_TWISTED; do leaf "$curve" "$curve"; done
}

# brainpool-signatures.txt: key algorithm hash hex(raw r||s), every hash on
# every Brainpool key. brainpool-ecdsa.txt: the same shapes as ecdsa.txt.
gen_brainpool_vectors() {
  log "brainpool vectors"
  local spec key field h
  {
    echo "# key algorithm hash hex(raw r||s over the digest in digests.txt)"
    for spec in $BRAINPOOL_CURVES; do
      key=${spec%%:*}
      field=$(cut -d: -f2 <<<"$spec")
      for h in $HASHES; do
        sign_ecdsa "$key" "$h"
        echo "$key ecdsa $h $(der_to_raw_hex "$WORK/sig.der" "$field")"
      done
    done
  } >"$VECTORS/brainpool-signatures.txt"

  local curve hash attempt raw r s shape
  {
    echo "# curve shape hash hex(raw r||s) hex(DER); digest in digests.txt"
    for spec in $BRAINPOOL_CURVES; do
      IFS=: read -r curve field hash <<<"$spec"
      declare -A found=()
      for attempt in $(seq 1 20000); do
        sign_ecdsa "$curve" "$hash"
        raw=$(der_to_raw_hex "$WORK/sig.der" "$field")
        r=${raw:0:$((field * 2))}
        s=${raw:$((field * 2))}
        for shape in $(ecdsa_shapes "$r" "$s"); do
          if [ -z "${found[$shape]:-}" ]; then
            found[$shape]=1
            echo "$curve $shape $hash $raw $(hex_of "$WORK/sig.der")"
          fi
        done
        [ "${#found[@]}" -ge 6 ] && break
      done
      [ "${#found[@]}" -ge 6 ] || { echo "missing ECDSA shapes for $curve" >&2; return 1; }
      unset found
    done
  } >"$VECTORS/brainpool-ecdsa.txt"
}
