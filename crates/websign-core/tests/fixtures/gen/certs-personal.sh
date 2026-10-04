# shellcheck shell=bash
# Subjects with givenName, surname and serialNumber (SPEC §6.1a). All are
# personal data of invented people.

gen_personal_name_certs() {
  log "personal name certificates"
  local given=060355042a surname=0603550404 serial=0603550405

  variant dn-pii "/C=PT/O=WebeSign Test Fixtures/GN=José Ângelo/SN=Conceição/serialNumber=IDCPT-12345123/CN=JOSÉ ÂNGELO CONCEIÇÃO" < <(std_ext)
  variant dn-pii-only "/C=PT/GN=MARIA/SN=SILVA/serialNumber=IDCPT-12345123" < <(std_ext)
  variant dn-pii-duplicates \
    "/C=PT/GN=First/GN=Second/SN=One/SN=Two/serialNumber=IDCPT-1111111/serialNumber=IDCPT-2222222/CN=Dup" < <(std_ext)

  # The first value of each attribute becomes a NumericString (tag 12), which
  # the reader ignores, so the second one is the first readable value.
  STRING_MASK=MASK:0x2 variant dn-pii-unreadable-first \
    "/C=PT/GN=First/GN=Second/SN=Alpha/SN=Beta/serialNumber=111/serialNumber=IDCPT-2222222/CN=Unreadable" < <(std_ext)
  retag_string dn-pii-unreadable-first "$given" First 13 12
  retag_string dn-pii-unreadable-first "$surname" Alpha 13 12
  retag_string dn-pii-unreadable-first "$serial" 111 13 12

  STRING_MASK=MASK:0x800 variant dn-pii-bmp "/C=PT/GN=JOSÉ Ω/SN=ÊXITO/CN=Bmp Person" < <(std_ext)
}
