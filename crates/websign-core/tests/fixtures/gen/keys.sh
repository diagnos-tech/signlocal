# shellcheck shell=bash
# Key pairs. They only live in the scratch directory: tests never need a
# private key, and committing none keeps secret scanners quiet.

gen_keys() {
  log "keys"
  new_rsa rsa2048 2048
  new_rsa rsa2048b 2048
  new_rsa rsa3072 3072
  new_rsa rsa4096 4096
  # Odd modulus size: OpenSSL sets the top two bits, so n has exactly 2047 bits.
  new_rsa rsa2047 2047
  new_pkey rsapss2048 RSA-PSS -pkeyopt rsa_keygen_bits:2048

  new_ec p256 prime256v1
  new_ec p256b prime256v1
  new_ec p384 secp384r1
  new_ec p521 secp521r1
  new_ec root prime256v1

  new_pkey ed25519 ed25519
  new_pkey ed448 ed448
  new_ec secp256k1 secp256k1
  new_ec secp224r1 secp224r1
  openssl dsaparam -genkey -noout -out "$WORK/keys/dsa1024.pem" 1024 2>/dev/null
}
