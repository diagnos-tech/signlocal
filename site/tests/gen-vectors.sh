#!/usr/bin/env sh
# Regenerates vectors.json: throwaway self-signed certificates and signatures
# over MESSAGE made with openssl (no secret is kept). Needs openssl and python3.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
printf 'SignLocal test message' > "$work/msg"

mk() { # name, genpkey args...
  name=$1; shift
  openssl genpkey "$@" -out "$work/$name.key" 2>/dev/null
  openssl req -new -x509 -key "$work/$name.key" -subj "/CN=Test $name" -days 2 -outform DER -out "$work/$name.der" 2>/dev/null
}
sig() { # key name, label, hash, extra sign args...
  name=$1; label=$2; h=$3; shift 3
  openssl dgst "-$h" -sign "$work/$name.key" "$@" -out "$work/$label.$h.sig" "$work/msg"
}

mk rsa -algorithm RSA -pkeyopt rsa_keygen_bits:2048
mk p256 -algorithm EC -pkeyopt ec_paramgen_curve:P-256
mk p384 -algorithm EC -pkeyopt ec_paramgen_curve:P-384
mk p521 -algorithm EC -pkeyopt ec_paramgen_curve:P-521
for h in sha256 sha384 sha512; do
  sig rsa rsa-pkcs1 $h
  sig rsa rsa-pss $h -sigopt rsa_padding_mode:pss -sigopt rsa_pss_saltlen:digest
  for c in p256 p384 p521; do sig $c $c $h; done
done

python3 - "$work" "$here/vectors.json" <<'PY'
import base64, glob, json, os, sys
work, out = sys.argv[1:]
b64 = lambda p: base64.b64encode(open(p, "rb").read()).decode()
data = {"message": "SignLocal test message", "certificates": {}, "signatures": []}
for name in ("rsa", "p256", "p384", "p521"):
    data["certificates"][name] = b64(f"{work}/{name}.der")
for path in sorted(glob.glob(f"{work}/*.sig")):
    name, hash_, _ = os.path.basename(path).split(".")
    cert = "rsa" if name.startswith("rsa-") else name
    data["signatures"].append({"cert": cert, "scheme": name, "hash": hash_, "der": b64(path)})
json.dump(data, open(out, "w"), indent=1)
PY
echo "wrote $here/vectors.json"
