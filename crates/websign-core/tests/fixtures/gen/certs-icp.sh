# shellcheck shell=bash
# ICP-Brasil certificates: policies (levels), SubjectAltName otherNames
# (2.16.76.1.3.x) and the CN holder-name convention "NAME:DIGITS".
#
# Values of the person otherName (2.16.76.1.3.1) are
#   birth date (8) + CPF (11) + NIS (11) + RG (15) + issuing body (6).

BIRTH=15051990
CPF=12345678901
NIS=98765432109
RG=000000123456789
ISSUER=SSP-SP
PF_VALUE=$BIRTH$CPF$NIS$RG$ISSUER

# The responsible person of a company certificate (2.16.76.1.3.4): same layout.
RESP_CPF=98765432100
RESP_VALUE=10101980${RESP_CPF}$(printf '%026d' 0)SSP-SP
CNPJ=12345678000195

# A second CPF, to tell which otherName a certificate's CPF came from.
OTHER_CPF=22222222222

# on OID ASN1TYPE VALUE -- one otherName entry of a subjectAltName list.
on() { printf 'otherName:%s;%s:%s' "$1" "$2" "$3"; }

# icp NAME CN POLICIES [SAN]
#
# ICP-Brasil style certificate. CN "-" leaves the subject without a CN;
# empty POLICIES / SAN omit the extension.
icp() {
  local name=$1 cn=$2 policies=$3 san=${4:-}
  local subject="/C=BR/O=ICP-Brasil/OU=Autoridade Certificadora Teste/OU=Certificado $name"
  if [ "$cn" != - ]; then subject="$subject/CN=$cn"; fi
  {
    std_ext
    if [ -n "$policies" ]; then echo "certificatePolicies=$policies"; fi
    if [ -n "$san" ]; then echo "subjectAltName=$san"; fi
  } | variant "$name" "$subject"
}

gen_icp_certs() {
  log "ICP-Brasil certificates"
  local pf_cn="ANA BEATRIZ SOUZA:$CPF" pj_cn="EMPRESA TESTE LTDA:$CNPJ"
  local t label asn1

  # Natural person A3, the reference certificate.
  icp icp-pf-a3 "$pf_cn" 2.16.76.1.2.3.1 \
    "email:ana.souza@example.com.br,$(on 2.16.76.1.3.1 OCTETSTRING "$PF_VALUE"),$(on 2.16.76.1.3.5 OCTETSTRING 1234567890120123SP),$(on 1.3.6.1.4.1.311.20.2.3 UTF8 ana@corp.example)"

  # The same otherName in every string type the SPEC accepts, and one it does not.
  for t in printable:PRINTABLESTRING utf8:UTF8 ia5:IA5STRING bmp:BMPSTRING; do
    label=${t%%:*}
    asn1=${t##*:}
    icp "icp-pf-$label" "$pf_cn" 2.16.76.1.2.3.1 "$(on 2.16.76.1.3.1 "$asn1" "$PF_VALUE")"
  done

  # Value lengths around the 19 characters birth date + CPF need.
  icp icp-pf-len-19 "$pf_cn" 2.16.76.1.2.3.1 "$(on 2.16.76.1.3.1 OCTETSTRING "$BIRTH$CPF")"
  icp icp-pf-len-18 "$pf_cn" 2.16.76.1.2.3.1 "$(on 2.16.76.1.3.1 OCTETSTRING "$BIRTH${CPF%?}")"
  icp icp-pf-len-8 "$pf_cn" 2.16.76.1.2.3.1 "$(on 2.16.76.1.3.1 OCTETSTRING "$BIRTH")"

  # CPF content rules.
  icp icp-pf-cpf-letter "$pf_cn" 2.16.76.1.2.3.1 \
    "$(on 2.16.76.1.3.1 OCTETSTRING "${BIRTH}1234567890X$NIS$RG$ISSUER")"
  icp icp-pf-cpf-zeros "$pf_cn" 2.16.76.1.2.3.1 \
    "$(on 2.16.76.1.3.1 OCTETSTRING "${BIRTH}00000000000$NIS$RG$ISSUER")"
  icp icp-pf-odd-birth-tail "$pf_cn" 2.16.76.1.2.3.1 \
    "$(on 2.16.76.1.3.1 OCTETSTRING "ABCDEFGH${CPF}tail-with-letters")"

  # 3.1 wins over 3.4 whatever the order; an invalid 3.1 is not replaced by 3.4.
  local other_resp
  other_resp=$(on 2.16.76.1.3.4 OCTETSTRING "10101980$OTHER_CPF")
  icp icp-pf-priority "$pf_cn" 2.16.76.1.2.3.1 \
    "$other_resp,$(on 2.16.76.1.3.1 OCTETSTRING "$BIRTH$CPF")"
  icp icp-pf-invalid-primary "$pf_cn" 2.16.76.1.2.3.1 \
    "$(on 2.16.76.1.3.1 OCTETSTRING "$BIRTH"),$other_resp"
  icp icp-only-34 "$pf_cn" 2.16.76.1.2.3.1 "$other_resp"

  gen_icp_company_certs "$pj_cn"
  gen_icp_holder_name_certs
  gen_icp_detection_certs "$pf_cn"
  gen_icp_level_certs
}

gen_icp_company_certs() {
  local pj_cn=$1 t label asn1 san
  local policy=2.16.76.1.2.1.1

  san="$(on 2.16.76.1.3.2 UTF8 'MARIA RESPONSAVEL'),$(on 2.16.76.1.3.3 OCTETSTRING $CNPJ),$(on 2.16.76.1.3.4 OCTETSTRING "$RESP_VALUE")"
  icp icp-pj-a1 "$pj_cn" $policy "$san"
  for t in printable:PRINTABLESTRING utf8:UTF8 ia5:IA5STRING; do
    label=${t%%:*}
    asn1=${t##*:}
    icp "icp-pj-$label" "$pj_cn" $policy \
      "$(on 2.16.76.1.3.3 "$asn1" $CNPJ),$(on 2.16.76.1.3.4 "$asn1" "$RESP_VALUE")"
  done

  # CNPJ rules: exactly 14 digits, not all zero.
  local row cnpj_name cnpj_value
  for row in 13:1234567800019 15:123456780001950 zeros:00000000000000 letter:1234567800019X; do
    cnpj_name=${row%%:*}
    cnpj_value=${row##*:}
    icp "icp-pj-cnpj-$cnpj_name" "$pj_cn" $policy \
      "$(on 2.16.76.1.3.3 OCTETSTRING "$cnpj_value"),$(on 2.16.76.1.3.4 OCTETSTRING "$RESP_VALUE")"
  done
  icp icp-pj-no-responsible "$pj_cn" $policy "$(on 2.16.76.1.3.3 OCTETSTRING $CNPJ)"
}

# Holder name: the CN without a trailing ":<digits>".
gen_icp_holder_name_certs() {
  local row name cn
  for row in \
    'plain|MARIA SILVA' \
    'suffix|MARIA SILVA:12345678901' \
    'alpha-suffix|MARIA SILVA:ABC' \
    'mixed-suffix|MARIA SILVA:123ABC' \
    'space-suffix|MARIA SILVA: 123' \
    'nested-colon|R2:D2:12345678901' \
    "accented|JOSÉ D'ÁVILA:12345678901"; do
    name=${row%%|*}
    cn=${row#*|}
    icp "icp-cn-$name" "$cn" 2.16.76.1.2.1.1
  done
  icp icp-no-cn - 2.16.76.1.2.1.1
}

# What makes a certificate ICP-Brasil, and what only looks like it.
gen_icp_detection_certs() {
  local pf_cn=$1
  icp icp-policy-only "POLICY ONLY:11111111111" 2.16.76.1.2.1.1
  icp icp-san-only "SAN ONLY:11111111111" "" "$(on 2.16.76.1.3.1 OCTETSTRING "$PF_VALUE")"
  icp icp-san-other-arc "OTHER ARC:11111111111" "" "$(on 2.16.76.1.3.2 UTF8 'NOME RESPONSAVEL')"
  icp icp-lookalike "LOOKALIKE:11111111111" 2.16.76.1.20.3,2.16.760.1.2.3.1 \
    "$(on 2.16.76.1.30.1 OCTETSTRING "$PF_VALUE")"
  icp non-icp-upn "PLAIN NAME:12345678901" "" \
    "email:plain@example.com,DNS:plain.example.com,$(on 1.3.6.1.4.1.311.20.2.3 UTF8 plain@corp.example)"
  icp icp-multi-policy "$pf_cn" 1.2.3.4,2.16.76.1.2.3.4,2.16.76.1.2.1.2
  icp icp-multi-policy-other-first "$pf_cn" 2.16.76.1.2.999.1,2.16.76.1.2.3.1
}

# One certificate per policy arc n of 2.16.76.1.2.<n>.1, around every range
# boundary of the level table.
gen_icp_level_certs() {
  local n
  for n in 0 1 2 3 4 5 100 101 102 103 104 105 302 303 304 305 999 4294967295; do
    icp "icp-level-$n" "LEVEL $n:11111111111" "2.16.76.1.2.$n.1"
  done
  icp icp-level-huge "LEVEL HUGE:11111111111" 2.16.76.1.2.4294967296.1
  icp icp-level-no-subarc "LEVEL NO SUBARC:11111111111" 2.16.76.1.2.3
}
