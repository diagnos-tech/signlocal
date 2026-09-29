# shellcheck shell=bash
# Serial numbers, validity encodings, subject name encodings and malformed
# known extensions. All use the shared p256 key.

gen_serial_and_time_certs() {
  log "serial and validity certificates"
  local pair
  # The serial file is hex; 80 is stored as 00 80 (sign byte), ff*20 as 00 ff*20.
  for pair in serial-01:01 serial-7f:7f serial-80:80 serial-0102:0102 serial-1234:1234 \
    serial-ff20:ffffffffffffffffffffffffffffffffffffffff; do
    issue "${pair%%:*}" p256 "$(leaf_subject "${pair%%:*}")" "${pair##*:}" < <(std_ext)
  done

  # name:start:end (UTCTime for 1950-2049, GeneralizedTime otherwise).
  local row name start end
  for row in \
    time-window:20240315102030Z:20240915184050Z \
    time-pre-1970:19600101000000Z:20491231235959Z \
    time-generalized:20500101000000Z:20510101000000Z \
    time-y2k:19991231235959Z:20000101000000Z \
    time-no-expiry:20200101000000Z:99991231235959Z \
    expired:20100101000000Z:20110101000000Z; do
    IFS=: read -r name start end <<<"$row"
    issue "$name" p256 "$(leaf_subject "$name")" "$(next_serial)" "$start" "$end" < <(std_ext)
  done
}

# Subject names: repeated attributes, multi-valued RDN, missing attributes and
# every ASN.1 string type the SPEC mentions (plus two it says to ignore).
gen_name_certs() {
  log "name certificates"
  variant dn-duplicates "/C=BR/C=US/O=First/O=Second/OU=A/OU=B/OU=C/CN=One/CN=Two" < <(std_ext)
  variant dn-multivalue-rdn "/C=BR/O=Org/OU=Unit+CN=Multi" < <(std_ext)
  variant dn-empty "/" < <(std_ext)
  variant dn-country-only "/C=BR" < <(std_ext)
  variant dn-org-only "/O=Only Org Name" < <(std_ext)

  # string_mask bits: 0x2 Printable, 0x4 Teletex (T61), 0x800 BMP; the default
  # (utf8only) gives UTF8String. `ca` keeps those types.
  variant dn-utf8 "/C=BR/O=Açougue & Cia Ltda/OU=Divisão São João/CN=JOSÉ AÇAÍ DA SILVA" < <(std_ext)
  STRING_MASK=MASK:0x2 variant dn-printable \
    "/C=BR/O=Printable Org/OU=Printable Unit/CN=PRINTABLE NAME 123" < <(std_ext)
  STRING_MASK=MASK:0x4 variant dn-teletex \
    "/C=BR/O=Ação Ltda/OU=Divisão/CN=JOSÉ TELETEX" < <(std_ext)
  STRING_MASK=MASK:0x800 variant dn-bmp \
    "/C=BR/O=Êxito Ltda/OU=Divisão/CN=JOSÉ Ω BMP" < <(std_ext)

  gen_retagged_name_certs
}

# OpenSSL never puts IA5, Numeric or Universal strings in a CN/O/OU, so these
# start as PrintableString certificates whose string tags are then rewritten.
# Tags: 13 Printable, 16 IA5, 12 Numeric, 1c Universal (UCS-4BE, 4 bytes/char).
gen_retagged_name_certs() {
  local cn=0603550403 org=060355040a unit=060355040b
  local ia5_cn=ia5.name.example.com ia5_org=ia5.org.example ia5_unit=ia5.unit

  STRING_MASK=MASK:0x2 variant dn-ia5 \
    "/C=BR/O=$ia5_org/OU=$ia5_unit/CN=$ia5_cn" < <(std_ext)
  retag_string dn-ia5 $org $ia5_org 13 16
  retag_string dn-ia5 $unit $ia5_unit 13 16
  retag_string dn-ia5 $cn $ia5_cn 13 16

  # Only the CN and the middle OU become NumericString.
  STRING_MASK=MASK:0x2 variant dn-numeric \
    "/C=BR/O=Numeric Org/OU=First/OU=222/OU=Third/CN=12345" < <(std_ext)
  retag_string dn-numeric $unit 222 13 12
  retag_string dn-numeric $cn 12345 13 12

  # The CN "UNIVERSAL NAME" (14 chars) becomes 56 bytes of UCS-4BE; the
  # PrintableString placeholder has the same 56 bytes.
  local filler text="UNIVERSAL NAME"
  filler=$(printf 'X%.0s' $(seq 1 56))
  STRING_MASK=MASK:0x2 variant dn-universal \
    "/C=BR/O=Universal Org/OU=Unit/CN=$filler" < <(std_ext)
  patch_cert dn-universal "${cn}1338$(ascii_hex "$filler")" \
    "${cn}1c38$(ascii_hex "$text" | sed 's/../000000&/g')"
}

# Known extensions whose content is not the type the extension defines
# (an INTEGER where a BIT STRING or SEQUENCE belongs), plus an unknown
# critical extension that must be skipped.
gen_malformed_extension_certs() {
  log "malformed extension certificates"
  local row name line
  for row in \
    'bad-ext-key-usage|2.5.29.15=critical,DER:02:01:05' \
    'bad-ext-extended-key-usage|2.5.29.37=DER:02:01:05' \
    'bad-ext-policies|2.5.29.32=DER:02:01:05' \
    'bad-ext-basic-constraints|2.5.29.19=critical,DER:02:01:05' \
    'bad-ext-san|2.5.29.17=DER:02:01:05' \
    'bad-ext-qc-statements|1.3.6.1.5.5.7.1.3=DER:02:01:05' \
    'bad-qc-statement-element|1.3.6.1.5.5.7.1.3=DER:30:03:02:01:05'; do
    name=${row%%|*}
    line=${row#*|}
    variant "$name" "$(leaf_subject "$name")" <<<"$line"
  done

  variant unknown-extension "$(leaf_subject unknown-extension)" <<EOF
keyUsage=critical,digitalSignature
1.3.6.1.4.1.99999.9=critical,DER:02:01:05
EOF
}
