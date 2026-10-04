# Fixtures dos testes do probe-core

Certificados DER e vetores de referência gerados com **OpenSSL 3** por
`generate.sh`. Os testes **não** executam OpenSSL: leem os arquivos
commitados aqui. Todo valor esperado nos testes vem da configuração do
gerador (listada abaixo) ou dos manifestos em `vectors/`, que o próprio
OpenSSL preenche (`openssl x509 -fingerprint`, `-serial`, `-startdate`,
`asn1parse`, `pkeyutl`); nenhum vem da saída da implementação.

## Como regenerar

```
tests/fixtures/generate.sh            # precisa de bash >= 4, openssl >= 3.0, od, GNU date
KEEP_WORKDIR=1 tests/fixtures/generate.sh   # mantém chaves e CSRs numa pasta temporária
```

Leva de 1 a 3 minutos. As **chaves são aleatórias** e não são commitadas
(os testes nunca precisam de chave privada), então cada execução gera bytes
diferentes. Isso é seguro porque:

- nenhum teste embute impressão digital, assinatura ou chave: tudo isso vem de
  `vectors/*.txt`, regenerado junto com os certificados;
- os demais valores (bits RSA, curva, nomes, datas, número de série, OIDs,
  bits de KeyUsage) vêm da configuração, não do material da chave.

Layout: `certs/*.der` (133 certificados), `vectors/*.txt` (manifestos),
`gen/*.sh` (o gerador, dividido por assunto).

## Autoridade e certificado padrão

`ca.der` é a raiz autoassinada (P-256, série `01`, `CA:TRUE` crítico, KeyUsage
`keyCertSign,cRLSign`): `C=BR, O=SignLocal Test Authority, OU=Fixtures Root,
CN=SignLocal Test Root CA`.

Toda folha "comum" (função `leaf`/`variant` do gerador) tem:

| Campo | Valor |
|---|---|
| subject | `C=BR, O=SignLocal Test Fixtures, OU=Unit A, OU=Unit B, CN=Fixture <nome do arquivo>` (todos UTF8String, exceto C) |
| issuer | o da raiz acima |
| validade | 2020-01-01T00:00:00Z = **1577836800** até 2040-01-01T00:00:00Z = **2208988800** |
| BasicConstraints | `CA:FALSE` (SEQUENCE vazia) |
| KeyUsage (crítico) | `digitalSignature`, `nonRepudiation` |
| EKU | `1.3.6.1.5.5.7.3.4`, `1.3.6.1.5.5.7.3.2` (nesta ordem) |
| série | sequencial a partir de 0x1000 (não testada, exceto nos `serial-*`) |

O OpenSSL acrescenta SubjectKeyIdentifier/AuthorityKeyIdentifier; o SPEC manda ignorá-los.
Todos os certificados que dizem "chave p256" compartilham **a mesma chave EC P-256**
de `p256.der`; por isso a assinatura de `p256` também confere contra eles.

## Tipos de chave

| Arquivo | Chave | Esperado em `CertInfo.key` |
|---|---|---|
| `rsa2048`, `rsa2048b` | RSA 2048 (duas chaves diferentes) | `Rsa { bits: 2048 }` |
| `rsa3072` | RSA 3072 | `Rsa { bits: 3072 }` |
| `rsa4096` | RSA 4096 | `Rsa { bits: 4096 }` |
| `rsa2047` | módulo de 2047 bits | `Rsa { bits: 2047 }` |
| `rsapss2048` | OID RSASSA-PSS `1.2.840.113549.1.1.10`, sem parâmetros | `Rsa { bits: 2048 }` |
| `p256`, `p256b` | EC P-256 (duas chaves) | `Ec { curve: P256 }` |
| `p384`, `p521` | EC P-384 / P-521 | `Ec { curve: P384 / P521 }` |
| `ed25519` | Ed25519 | `Unsupported { oid: "1.3.101.112" }` |
| `ed448` | Ed448 | `Unsupported { oid: "1.3.101.113" }` |
| `dsa1024` | DSA | `Unsupported { oid: "1.2.840.10040.4.1" }` |
| `secp256k1` | EC, curva desconhecida | `Unsupported { oid: "1.3.132.0.10" }` |
| `secp224r1` | EC, curva desconhecida | `Unsupported { oid: "1.3.132.0.33" }` |
| `brainpoolP256r1` | EC, curva desconhecida | `Unsupported { oid: "1.3.36.3.3.2.8.1.1.7" }` |

## Extensões (chave p256)

| Arquivo | Conteúdo | Esperado |
|---|---|---|
| `ca-pathlen0` | `CA:TRUE,pathlen:0`, sem KeyUsage | `is_ca`, `key_usage None`, `can_sign false` |
| `ca-digital-signature` | `CA:TRUE`, KeyUsage `digitalSignature,keyCertSign,cRLSign` | `is_ca`, `can_sign false` |
| `ku-<bit>` | só um bit de KeyUsage: `digital-signature`, `non-repudiation`, `key-encipherment`, `data-encipherment`, `key-agreement`, `key-cert-sign`, `crl-sign` | exatamente esse bit `true`; `can_sign` só para os dois primeiros |
| `ku-decipher-only` | só `decipherOnly` (bit 8, fora da struct) | `Some(KeyUsage::default())`, `can_sign false` |
| `ku-all` | os 7 bits + `encipherOnly` + `decipherOnly`, `CA:FALSE` | os 7 `true`, `is_ca false`, `can_sign true` |
| `eku-multi` | EKU: `1.3.6.1.5.5.7.3.4`, `1.3.6.1.4.1.311.10.3.12`, `1.3.6.1.5.5.7.3.2`, `2.5.29.37.0`, `1.3.6.1.5.5.7.3.1` | mesma ordem |
| `eku-server-only` | EKU só `serverAuth` + `digitalSignature` | `can_sign true` (EKU não entra) |
| `policies-multi` | `1.2.3.4`, `2.23.140.1.2.2`, `1.3.6.1.4.1.99999.1.2` | mesma ordem |
| `policies-qualifiers` | `1.3.6.1.4.1.99999.2.1` (com CPS e UserNotice), `1.2.3.4.5` | só os OIDs, na ordem |
| `bare` | só `nsComment` (v3, sem KeyUsage/BC/EKU/políticas) | tudo vazio/`None`, `can_sign true` |
| `v1` | versão 1, sem extensões | idem |
| `tiny` | v1 autoassinado, nomes vazios, cerca de 233 bytes, menos que 256 (SEQUENCE externa `30 81 xx`) | subject/issuer vazios, série `01` |
| `unknown-extension` | KeyUsage `digitalSignature` + extensão crítica desconhecida `1.3.6.1.4.1.99999.9` com lixo | ignorada; `key_usage` só `digital_signature` |

## Série, datas e nomes (chave p256)

| Arquivo | Esperado |
|---|---|
| `serial-01`, `serial-7f`, `serial-1234`, `serial-0102` | `"01"`, `"7f"`, `"1234"`, `"0102"` |
| `serial-80` | conteúdo `00 80` → `"80"` |
| `serial-ff20` | vinte bytes `ff` (conteúdo com `00` na frente) → `"ff"` × 20 |
| `time-window` | 2024-03-15T10:20:30Z (**1710498030**) até 2024-09-15T18:40:50Z (**1726425650**) |
| `time-pre-1970` | UTCTime `600101000000Z` = 1960-01-01 (**-315619200**) até 2049-12-31T23:59:59Z (**2524607999**, ainda UTCTime) |
| `time-generalized` | GeneralizedTime: 2050-01-01 (**2524608000**) até 2051-01-01 (**2556144000**) |
| `time-y2k` | 1999-12-31T23:59:59Z (**946684799**) até 2000-01-01T00:00:00Z (**946684800**) |
| `time-no-expiry` | `notAfter` = 9999-12-31T23:59:59Z (**253402300799**) |
| `expired` | 2010-01-01 (**1262304000**) até 2011-01-01 (**1293840000**) |
| `dn-duplicates` | `C=BR,C=US,O=First,O=Second,OU=A,OU=B,OU=C,CN=One,CN=Two` → vale o primeiro C/O/CN; OU `[A,B,C]` |
| `dn-multivalue-rdn` | RDN multivalorado `OU=Unit+CN=Multi` |
| `dn-empty`, `dn-country-only`, `dn-org-only` | subject vazio; só `C=BR`; só `O=Only Org Name` |
| `dn-utf8` | UTF8String: CN `JOSÉ AÇAÍ DA SILVA`, O `Açougue & Cia Ltda`, OU `Divisão São João` |
| `dn-printable` | PrintableString: CN `PRINTABLE NAME 123`, O `Printable Org`, OU `Printable Unit` |
| `dn-teletex` | TeletexString (Latin-1): CN `JOSÉ TELETEX`, O `Ação Ltda`, OU `Divisão` |
| `dn-bmp` | BMPString (UTF-16BE): CN `JOSÉ Ω BMP`, O `Êxito Ltda`, OU `Divisão` |
| `dn-ia5` | IA5String: CN `ia5.name.example.com`, O `ia5.org.example`, OU `ia5.unit` |
| `dn-numeric` | O `Numeric Org`; OU `First`, **`222` (NumericString)**, `Third`; **CN `12345` (NumericString)** → CN e o OU do meio ignorados |
| `dn-universal` | CN em UniversalString (ignorado); O `Universal Org`, OU `Unit` |

`dn-ia5`, `dn-numeric` e `dn-universal` são feitos em duas etapas: o OpenSSL não
coloca IA5/Numeric/Universal em CN/O/OU, então o certificado nasce com
PrintableString e o gerador troca o tag (`patch_cert`/`retag_string`) sem mudar
o tamanho (no Universal, o conteúdo vira UCS-4BE). **A assinatura desses três
deixa de conferir**, o que não importa: `CertInfo` não verifica a assinatura do
certificado.

## Extensões conhecidas malformadas

Cada uma traz um INTEGER `02 01 05` onde o tipo da extensão exige BIT STRING ou
SEQUENCE. Todas devem dar `CertError::Malformed`:
`bad-ext-key-usage`, `bad-ext-extended-key-usage`, `bad-ext-policies`,
`bad-ext-basic-constraints`, `bad-ext-san`, `bad-ext-qc-statements`, e
`bad-qc-statement-element` (qcStatements = `SEQUENCE { INTEGER 5 }`).

## ICP-Brasil

Valor do otherName de pessoa (`2.16.76.1.3.1`) = nascimento (8) + CPF (11) +
NIS (11) + RG (15) + órgão emissor (6): `15051990` `12345678901` `98765432109`
`000000123456789` `SSP-SP`. Responsável (`.3.4`): `10101980` + CPF
`98765432100` + zeros + `SSP-SP`. CNPJ (`.3.3`): `12345678000195`. Um segundo CPF,
`22222222222`, distingue de qual otherName veio o CPF. Sujeito:
`C=BR, O=ICP-Brasil, OU=Autoridade Certificadora Teste, OU=Certificado <nome>, CN=...`.

| Arquivo | CN | Política / SAN | Esperado (`IcpBrasil`) |
|---|---|---|---|
| `icp-pf-a3` | `ANA BEATRIZ SOUZA:12345678901` | `2.16.76.1.2.3.1`; e-mail, `.3.1` (OCTET STRING), `.3.5`, UPN | `A3`, `ANA BEATRIZ SOUZA`, cpf `12345678901`, cnpj `None` |
| `icp-pf-printable` / `-utf8` / `-ia5` | idem | `.3.1` em PrintableString / UTF8String / IA5String | igual ao `icp-pf-a3` |
| `icp-pf-bmp` | idem | `.3.1` em BMPString | cpf `None` (tipo não aceito), resto igual |
| `icp-pf-len-19` / `-len-18` / `-len-8` | idem | valor com 19 / 18 / 8 caracteres | cpf `12345678901` / `None` / `None` |
| `icp-pf-cpf-letter` | idem | CPF `1234567890X` | cpf `None` |
| `icp-pf-cpf-zeros` | idem | CPF `00000000000` | cpf `None` |
| `icp-pf-odd-birth-tail` | idem | `ABCDEFGH` + CPF + `tail-with-letters` | cpf `12345678901` |
| `icp-pf-priority` | idem | `.3.4` (CPF `22222222222`) antes de `.3.1` (CPF `12345678901`) | cpf `12345678901` |
| `icp-pf-invalid-primary` | idem | `.3.1` só com nascimento; `.3.4` válido (`22222222222`) | cpf `None` (não cai para o `.3.4`; SPEC §6.3) |
| `icp-only-34` | idem | só `.3.4` (`22222222222`) | cpf `22222222222` |
| `icp-pj-a1` | `EMPRESA TESTE LTDA:12345678000195` | `2.16.76.1.2.1.1`; `.3.2`, `.3.3`, `.3.4` | `A1`, `EMPRESA TESTE LTDA`, cpf `98765432100`, cnpj `12345678000195` |
| `icp-pj-printable` / `-utf8` / `-ia5` | idem | `.3.3` e `.3.4` no tipo indicado | igual ao `icp-pj-a1` |
| `icp-pj-cnpj-13` / `-15` / `-zeros` / `-letter` | idem | CNPJ com 13 dígitos / 15 / zeros / letra | cnpj `None`, cpf `98765432100` |
| `icp-pj-no-responsible` | idem | só `.3.3` | cnpj `12345678000195`, cpf `None` |
| `icp-cn-plain`, `-suffix`, `-alpha-suffix`, `-mixed-suffix`, `-space-suffix`, `-nested-colon`, `-accented` | `MARIA SILVA`; `MARIA SILVA:12345678901`; `…:ABC`; `…:123ABC`; `…: 123`; `R2:D2:12345678901`; `JOSÉ D'ÁVILA:12345678901` | `2.16.76.1.2.1.1` | holder: `MARIA SILVA`; `MARIA SILVA`; `MARIA SILVA:ABC`; `MARIA SILVA:123ABC`; `MARIA SILVA: 123`; `R2:D2`; `JOSÉ D'ÁVILA` |
| `icp-no-cn` | (sem CN) | `2.16.76.1.2.1.1` | holder `None`, level `A1` |
| `icp-policy-only` | `POLICY ONLY:11111111111` | só a política `A1` | `Some`, sem CPF/CNPJ |
| `icp-san-only` | `SAN ONLY:11111111111` | só `.3.1` válido, sem política | `Some`, level `None`, cpf `12345678901` |
| `icp-san-other-arc` | `OTHER ARC:11111111111` | só `.3.2` | `Some`, level `None`, sem CPF/CNPJ |
| `icp-lookalike` | idem | políticas `2.16.76.1.20.3`, `2.16.760.1.2.3.1`; otherName `2.16.76.1.30.1` | `icp_brasil None` |
| `non-icp-upn` | `PLAIN NAME:12345678901` | e-mail, DNS, UPN `1.3.6.1.4.1.311.20.2.3` | `icp_brasil None` |
| `icp-multi-policy` | `ANA BEATRIZ SOUZA:…` | `1.2.3.4`, `2.16.76.1.2.3.4`, `2.16.76.1.2.1.2` | level `A3` (primeira política ICP) |
| `icp-multi-policy-other-first` | idem | `2.16.76.1.2.999.1`, `2.16.76.1.2.3.1` | level `Other(999)` |
| `icp-level-<n>` | `LEVEL <n>:11111111111` | `2.16.76.1.2.<n>.1` | ver tabela abaixo |
| `icp-level-no-subarc` | idem | `2.16.76.1.2.3` (sem mais arcos) | `A3` |
| `icp-level-huge` | idem | `2.16.76.1.2.4294967296.1` | política listada como está, level `None` (não cabe em `u32`) |

`icp-level-<n>`, com n = 0, 1, 2, 3, 4, 5, 100, 101, 102, 103, 104, 105, 302,
303, 304, 305, 999, 4294967295:

| n | 0 | 1 | 2 | 3 | 4 | 5 | 100 | 101 | 102 | 103 | 104 | 105 | 302 | 303 | 304 | 305 | 999 | 4294967295 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| level | `Other(0)` | `A1` | `A2` | `A3` | `A4` | `Other(5)` | `Other(100)` | `S1` | `S2` | `S3` | `S4` | `Other(105)` | `Other(302)` | `T3` | `T4` | `Other(305)` | `Other(999)` | `Other(4294967295)` |

## eIDAS (qcStatements)

Assunto `C=PT, O=Test Qualified Provider, CN=QC fixture <nome>`, KeyUsage
`nonRepudiation`. Statements: `0.4.0.1862.1.<n>` (1 QcCompliance, 2 QcLimitValue,
3 QcRetentionPeriod, 4 QcSSCD, 5 QcPDS, 6 QcType; 6.1 esign, 6.2 eseal, 6.3 web).

| Arquivo | Statements, em ordem | Esperado (`Qualified`) |
|---|---|---|
| `qc-esign-sscd` | 1.2 (EUR 1000), 1.1, 1.3 (15), 1.4, 1.5 (URL+`en`), 1.6 `{6.1}`, `1.2.3.4.5` | compliance, sscd, `[ESign]` |
| `qc-eseal` | 1.1, 1.6 `{6.2}` | compliance, sem sscd, `[ESeal]` |
| `qc-web` | 1.6 `{6.3}` | sem compliance, sem sscd, `[Web]` |
| `qc-type-order` | 1.6 `{6.3, 6.9, 6.1, 6.2}` | `[Web, ESign, ESeal]` (6.9 ignorado) |
| `qc-sscd-only` | 1.4 | só sscd |
| `qc-unknown-only` | `1.2.3.4.5` | `Some(default)` |
| `qc-empty` | `SEQUENCE {}` | `Some(default)` |
| `qc-type-empty-info` | 1.6 com `SEQUENCE {}` | `Some(default)` |

## Manifestos (`vectors/`)

Texto, `#` comenta, um registro por linha, campos separados por espaço.

| Arquivo | Registro |
|---|---|
| `digests.txt` | `hash hex(digest)` da mensagem `SignLocal probe-core fixture message` (`openssl dgst`) |
| `digestinfo.txt` | `hash hex(DigestInfo)`, montado pelo gerador de ASN.1 do OpenSSL a partir dos OIDs dos hashes e conferido contra uma assinatura RSA "crua" (`rsautl -pkcs`) igual à que `pkeyutl -pkeyopt digest:` produz |
| `signatures.txt` | `chave algoritmo hash hex(assinatura)` sobre o digest acima; `chave` é o nome do certificado. `pkcs1`, `pss` (sal = tamanho do digest, MGF1 com o mesmo hash), `ecdsa` (crua `r‖s`). Chaves: `rsa2048`, `rsa2048b`, `rsa3072`, `rsa4096`, `rsa2047` (pkcs1 e pss × 3 hashes), `rsapss2048` (pss × 3), `p256`, `p256b`, `p384`, `p521` (ecdsa × 3) = 45 válidas. Mais 9 **inválidas de propósito**, em `rsa2048`: `pss-salt-0`, `pss-salt-20`, `pss-mgf1-sha1` × 3 hashes |
| `ecdsa.txt` | `curva forma hash hex(cru) hex(DER)`: a mesma assinatura nos dois formatos; o cru é lido de volta pelo `asn1parse` do OpenSSL. `forma`: `plain` (nenhum zero à esquerda nem bit alto), `high-r`, `high-s`, `high-both` (bit alto: o DER leva `00`), `short-r`, `short-s` (primeiro byte zero: o DER é mais curto que o campo). Uma linha por forma e curva; a P-521 sempre usa o comprimento longo `81 xx` |
| `fingerprints.txt` | `nome sha256(DER)` de cada certificado (`openssl x509 -fingerprint`) |
| `validity.txt` | `nome série notBefore notAfter` de cada certificado, como o OpenSSL informa (Unix em segundos, `date -u -d`) |
