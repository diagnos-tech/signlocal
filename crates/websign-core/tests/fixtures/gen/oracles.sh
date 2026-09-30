# shellcheck shell=bash
# Per-certificate oracles: fingerprint, serial and validity exactly as OpenSSL
# reports them, so the tests never need to trust the crate under test.

# One line per certificate: fingerprint, serial and validity as OpenSSL reports them.
gen_certificate_oracles() {
  local f name
  {
    echo "# name sha256(DER) lowercase hex"
    for f in "$CERTS"/*.der; do
      name=$(basename "$f" .der)
      echo "$name $(openssl x509 -inform DER -in "$f" -noout -fingerprint -sha256 \
        | sed 's/.*=//' | tr -d ':' | tr 'A-F' 'a-f')"
    done
  } >"$VECTORS/fingerprints.txt"

  {
    echo "# name serial(hex, no sign byte) notBefore(unix) notAfter(unix)"
    for f in "$CERTS"/*.der; do
      name=$(basename "$f" .der)
      echo "$name $(openssl x509 -inform DER -in "$f" -noout -serial | sed 's/.*=//' | tr 'A-F' 'a-f') \
$(unix_time "$(openssl x509 -inform DER -in "$f" -noout -startdate | sed 's/.*=//')") \
$(unix_time "$(openssl x509 -inform DER -in "$f" -noout -enddate | sed 's/.*=//')")"
    done
  } >"$VECTORS/validity.txt"
}

# OpenSSL prints "Mar 15 10:20:30 2024 GMT"; GNU date turns it into Unix time.
unix_time() { date -u -d "$1" +%s; }
