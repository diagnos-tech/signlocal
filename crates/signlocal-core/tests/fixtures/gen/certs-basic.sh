# shellcheck shell=bash
# Root CA, one certificate per key type, and one per extension scenario.

gen_root() {
  log "root CA"
  issue ca root "$ROOT_SUBJECT" 01 "$V_START" "$V_END" self <<EOF
basicConstraints=critical,CA:TRUE
keyUsage=critical,keyCertSign,cRLSign
EOF
  cp "$WORK/ca/ca.pem" "$WORK/ca/root.pem"
}

# One ordinary certificate per key. The unsupported kinds (Ed25519, other
# curves, DSA) exist to check PublicKeyKind::Unsupported and its OID.
gen_key_type_certs() {
  log "key type certificates"
  local key
  for key in rsa2048 rsa2048b rsa3072 rsa4096 rsa2047 rsapss2048 \
    p256 p256b p384 p521 \
    ed25519 ed448 secp256k1 secp224r1 dsa1024; do
    leaf "$key" "$key"
  done
}

gen_extension_certs() {
  log "extension certificates"
  local name

  # CA certificates: is_ca wins over any KeyUsage bit.
  variant ca-pathlen0 "$(leaf_subject ca-pathlen0)" <<EOF
basicConstraints=critical,CA:TRUE,pathlen:0
EOF
  variant ca-digital-signature "$(leaf_subject ca-digital-signature)" <<EOF
basicConstraints=critical,CA:TRUE
keyUsage=critical,digitalSignature,keyCertSign,cRLSign
EOF

  # KeyUsage: one certificate per bit, none of them a CA.
  local pair usage
  for pair in digital-signature:digitalSignature non-repudiation:nonRepudiation \
    key-encipherment:keyEncipherment data-encipherment:dataEncipherment \
    key-agreement:keyAgreement key-cert-sign:keyCertSign crl-sign:cRLSign \
    decipher-only:decipherOnly; do
    name=ku-${pair%%:*}
    usage=${pair##*:}
    variant "$name" "$(leaf_subject "$name")" <<EOF
basicConstraints=CA:FALSE
keyUsage=critical,$usage
EOF
  done
  variant ku-all "$(leaf_subject ku-all)" <<EOF
basicConstraints=CA:FALSE
keyUsage=critical,digitalSignature,nonRepudiation,keyEncipherment,dataEncipherment,keyAgreement,keyCertSign,cRLSign,encipherOnly,decipherOnly
EOF

  # ExtendedKeyUsage: the order below is neither sorted nor reversed.
  variant eku-multi "$(leaf_subject eku-multi)" <<EOF
keyUsage=critical,digitalSignature
extendedKeyUsage=emailProtection,1.3.6.1.4.1.311.10.3.12,clientAuth,anyExtendedKeyUsage,serverAuth
EOF
  variant eku-server-only "$(leaf_subject eku-server-only)" <<EOF
keyUsage=critical,digitalSignature
extendedKeyUsage=serverAuth
EOF

  # Certificate policies, with and without qualifiers.
  variant policies-multi "$(leaf_subject policies-multi)" <<EOF
certificatePolicies=1.2.3.4,2.23.140.1.2.2,1.3.6.1.4.1.99999.1.2
EOF
  variant policies-qualifiers "$(leaf_subject policies-qualifiers)" <<EOF
certificatePolicies=@pol_a,@pol_b
[pol_a]
policyIdentifier=1.3.6.1.4.1.99999.2.1
CPS.1="http://example.com/cps"
userNotice.1=@notice
[notice]
explicitText="Test notice"
[pol_b]
policyIdentifier=1.2.3.4.5
EOF

  # No KeyUsage, BasicConstraints, EKU or policies.
  variant bare "$(leaf_subject bare)" <<EOF
nsComment=fixture with no extension the crate reads
EOF
  variant v1 "$(leaf_subject v1)" </dev/null

  # Self-signed, version 1, empty names: small enough (under 256 bytes) for the
  # outer SEQUENCE to use the one-byte long length form, 30 81 xx.
  issue tiny p256 "/" 01 "$V_START" "$V_END" self </dev/null
}
