# shellcheck shell=bash
# eIDAS qualified certificates: qcStatements (RFC 3739, ETSI EN 319 412-5)
# assembled with OpenSSL's ASN1: extension syntax. Statement OIDs are
# 0.4.0.1862.1.<n>: 1 QcCompliance, 2 QcLimitValue, 3 QcRetentionPeriod,
# 4 QcSSCD, 5 QcPDS, 6 QcType (6.1 esign, 6.2 eseal, 6.3 web).

# qc NAME -- the [qcs] statement list (and its sections) comes from stdin.
qc() {
  local name=$1
  {
    echo "keyUsage=critical,nonRepudiation"
    echo "1.3.6.1.5.5.7.1.3=ASN1:SEQUENCE:qcs"
    echo "[qcs]"
    cat
  } | variant "$name" "/C=PT/O=Test Qualified Provider/CN=QC fixture $name"
}

# Sections reused by several certificates.
QC_SECTIONS='
[compliance]
id=OID:0.4.0.1862.1.1
[sscd]
id=OID:0.4.0.1862.1.4
[esign]
id=OID:0.4.0.1862.1.6
info=SEQUENCE:esign_info
[esign_info]
t1=OID:0.4.0.1862.1.6.1
[unknown]
id=OID:1.2.3.4.5
info=OCTETSTRING:whatever
'

gen_qualified_certs() {
  log "qualified certificates"

  # Everything a qualified signature certificate on a QSCD carries, with
  # statements this crate does not read before, between and after the ones it does.
  qc qc-esign-sscd <<EOF
s1=SEQUENCE:limit
s2=SEQUENCE:compliance
s3=SEQUENCE:retention
s4=SEQUENCE:sscd
s5=SEQUENCE:pds
s6=SEQUENCE:esign
s7=SEQUENCE:unknown
[limit]
id=OID:0.4.0.1862.1.2
info=SEQUENCE:limit_info
[limit_info]
currency=SEQUENCE:currency
amount=INTEGER:1000
exponent=INTEGER:0
[currency]
iso=PRINTABLESTRING:EUR
[retention]
id=OID:0.4.0.1862.1.3
info=INTEGER:15
[pds]
id=OID:0.4.0.1862.1.5
info=SEQUENCE:pds_list
[pds_list]
loc=SEQUENCE:pds_location
[pds_location]
url=IA5STRING:https://example.com/pds_en.pdf
lang=PRINTABLESTRING:en
$QC_SECTIONS
EOF

  qc qc-eseal <<EOF
s1=SEQUENCE:compliance
s2=SEQUENCE:eseal
[eseal]
id=OID:0.4.0.1862.1.6
info=SEQUENCE:eseal_info
[eseal_info]
t1=OID:0.4.0.1862.1.6.2
$QC_SECTIONS
EOF

  qc qc-web <<EOF
s1=SEQUENCE:web
[web]
id=OID:0.4.0.1862.1.6
info=SEQUENCE:web_info
[web_info]
t1=OID:0.4.0.1862.1.6.3
$QC_SECTIONS
EOF

  # Types keep certificate order; the unknown 6.9 is skipped.
  qc qc-type-order <<EOF
s1=SEQUENCE:mixed
[mixed]
id=OID:0.4.0.1862.1.6
info=SEQUENCE:mixed_info
[mixed_info]
t1=OID:0.4.0.1862.1.6.3
t2=OID:0.4.0.1862.1.6.9
t3=OID:0.4.0.1862.1.6.1
t4=OID:0.4.0.1862.1.6.2
$QC_SECTIONS
EOF

  qc qc-sscd-only <<EOF
s1=SEQUENCE:sscd
$QC_SECTIONS
EOF

  qc qc-unknown-only <<EOF
s1=SEQUENCE:unknown
$QC_SECTIONS
EOF

  # Hand-assembled DER where OpenSSL's syntax cannot express the shape.
  local row name der
  for row in \
    'qc-empty|30:00' \
    'qc-type-empty-info|30:0C:30:0A:06:06:04:00:8E:46:01:06:30:00'; do
    name=${row%%|*}
    der=${row#*|}
    variant "$name" "/C=PT/O=Test Qualified Provider/CN=QC fixture $name" <<EOF
1.3.6.1.5.5.7.1.3=DER:$der
EOF
  done
}
