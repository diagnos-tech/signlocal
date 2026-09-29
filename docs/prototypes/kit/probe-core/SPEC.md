# probe-core — especificação

Biblioteca Rust **pura** (sem chamadas ao sistema operacional) com as peças que
todas as origens de chave compartilham. É a fonte de verdade para quem escreve os
testes e para quem escreve a implementação: **nenhum dos dois vê o trabalho do
outro**. Onde esta especificação for omissa, registre a suposição num comentário
`// SPEC:` no código ou no teste — o revisor usa isso para conciliar.

A API pública está declarada em `src/` com corpos `todo!()`. **Não mude assinaturas
públicas.** Tipos, variantes e campos públicos são contrato. Pode criar módulos
privados à vontade.

Convenções gerais:

- Erros são `enum`/`struct` com `thiserror`, `Debug + Clone + PartialEq + Eq`.
- Nada entra em pânico com entrada externa (bytes de certificado, assinatura,
  strings): toda falha vira `Err`.
- OIDs aparecem como texto em notação pontuada (`"2.16.76.1.2.3.4"`).
- Tempos são segundos Unix (`i64`, UTC).

---

## 1. `hash` — `HashAlgorithm`

`enum HashAlgorithm { Sha256, Sha384, Sha512 }`

| Método | Comportamento |
|---|---|
| `ALL` | `[Sha256, Sha384, Sha512]`, nesta ordem |
| `digest_len()` | 32, 48, 64 |
| `name()` | `"SHA-256"`, `"SHA-384"`, `"SHA-512"` (nomes do WebCrypto) |
| `digest(data)` | SHA-2 correspondente sobre `data` |
| `check_digest(d)` | `Ok(())` se `d.len() == digest_len()`, senão `DigestLengthError { algorithm, expected, actual }` |
| `Display` | igual a `name()` |
| `FromStr` | aceita, sem diferenciar maiúsculas, `SHA-256`/`SHA256`, `SHA-384`/`SHA384`, `SHA-512`/`SHA512`. Qualquer outra coisa (inclusive espaços em volta, `SHA-1`, `SHA-224`, `SHA3-256`, vazio) → `UnknownAlgorithmError { name }` com a string original |

`DigestLengthError` exibe: `digest for SHA-256 must be 32 bytes, got 31`.

## 2. `algorithm` — `SignatureAlgorithm`

`enum SignatureAlgorithm { Ecdsa, RsaPkcs1v15, RsaPss }`

| Método | Comportamento |
|---|---|
| `ALL` | `[Ecdsa, RsaPkcs1v15, RsaPss]` |
| `name()` | `"ECDSA"`, `"RSASSA-PKCS1-v1_5"`, `"RSASSA-PSS"` (nomes do WebCrypto) |
| `is_rsa()` | `false`, `true`, `true` |
| `Display` | igual a `name()` |
| `FromStr` | aceita exatamente os três nomes, sem diferenciar maiúsculas. Outros → `UnknownAlgorithmError { name }` |

`UnknownAlgorithmError` exibe: `unknown algorithm: <name>`.

## 3. `pkcs1` — `digest_info`

`digest_info(hash, digest) -> Result<Vec<u8>, DigestLengthError>`

Devolve o `DigestInfo` DER (RFC 8017 §9.2, nota 1) = prefixo fixo ‖ digest.
É o que `CKM_RSA_PKCS` e assinadores RSA "crus" esperam quando o hash já foi feito.
Valida o tamanho do digest antes (mesmo erro de `check_digest`).

Prefixos (hex):

```
SHA-256: 30 31 30 0d 06 09 60 86 48 01 65 03 04 02 01 05 00 04 20
SHA-384: 30 41 30 0d 06 09 60 86 48 01 65 03 04 02 02 05 00 04 30
SHA-512: 30 51 30 0d 06 09 60 86 48 01 65 03 04 02 03 05 00 04 40
```

## 4. `ecdsa` — curvas e codificação da assinatura

`enum Curve { P256, P384, P521 }`

| Método | Comportamento |
|---|---|
| `field_len()` | 32, 48, 66 |
| `signature_len()` | `2 * field_len()` |
| `name()` | `"P-256"`, `"P-384"`, `"P-521"` |
| `from_oid(&str)` | `1.2.840.10045.3.1.7` → P256, `1.3.132.0.34` → P384, `1.3.132.0.35` → P521, outro → `None` |

Assinaturas ECDSA circulam em dois formatos: **cru** (`r ‖ s`, cada um com
`field_len` bytes big-endian, preenchido com zeros à esquerda — IEEE P1363, é o
que o SDK entrega, o que CNG e PKCS#11 devolvem) e **DER** (`ECDSA-Sig-Value ::=
SEQUENCE { r INTEGER, s INTEGER }` — o que o macOS devolve).

`der_to_raw(der, curve) -> Result<Vec<u8>, EcdsaEncodingError>`

- Estrutura exigida: `30 len 02 len r 02 len s`, nada depois do fim da SEQUENCE.
- Comprimentos em forma curta ou longa de 1 byte (`81 xx`, necessário na P-521).
  Forma indefinida (`80`) ou longa com mais bytes → `Malformed`.
- INTEGER negativo (primeiro byte de conteúdo ≥ `0x80`) ou vazio → `Malformed`.
- Zeros à esquerda a mais são **tolerados** (alguns tokens não geram DER mínimo):
  remova-os.
- `r` ou `s` igual a zero → `Malformed`.
- Valor que, sem zeros à esquerda, não cabe em `field_len` bytes → `IntegerTooLarge`.
- Saída: `r` e `s` preenchidos à esquerda até `field_len`, concatenados.

`raw_to_der(raw, curve) -> Result<Vec<u8>, EcdsaEncodingError>`

- `raw.len() != signature_len()` → `WrongLength { expected, actual }`.
- Cada metade vira INTEGER DER **mínimo**: remove zeros à esquerda (mantendo ao
  menos um byte) e acrescenta `00` se o primeiro byte restante for ≥ `0x80`.
- Usa forma longa `81 xx` quando o conteúdo da SEQUENCE passar de 127 bytes.
- `der_to_raw(raw_to_der(x)) == x` para todo `x` com `r` e `s` não nulos.

## 5. `fingerprint` — `Fingerprint`

SHA-256 do DER do certificado. É a identidade de um certificado em todo o projeto
(deduplicação, "último usado", seleção por linha de comando).

| Método | Comportamento |
|---|---|
| `of(der)` | SHA-256 de `der` |
| `from_bytes([u8; 32])` / `as_bytes()` | conversões diretas |
| `to_hex()` | 64 caracteres hexadecimais minúsculos, sem separador |
| `Display` | pares hexadecimais **maiúsculos** separados por `:` (95 caracteres) |
| `Debug` | `Fingerprint(<to_hex()>)` |
| `FromStr` | ignora `:` e espaços em qualquer posição; o resto tem de ser exatamente 64 dígitos hexadecimais (maiúsculos ou minúsculos). Senão → `ParseFingerprintError` |

Deriva `Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord`.

## 6. `cert` — resumo de certificado X.509

`CertInfo::from_der(der) -> Result<CertInfo, CertError>`

DER que não é um `Certificate` X.509 válido (inclusive bytes sobrando no fim) →
`CertError::Malformed(<mensagem livre>)`. Extensão conhecida mal formada também →
`Malformed`. Extensões desconhecidas são ignoradas.

Campos de `CertInfo`:

| Campo | Conteúdo |
|---|---|
| `fingerprint` | `Fingerprint::of(der)` |
| `subject`, `issuer` | `DistinguishedName` (abaixo) |
| `serial_hex` | hex minúsculo dos octetos de conteúdo do número de série, sem o `00` de sinal inicial quando houver (ex.: série `0x80` codificada `00 80` → `"80"`; série `0x1234` → `"1234"`) |
| `not_before`, `not_after` | segundos Unix |
| `key` | `PublicKeyKind` |
| `key_usage` | `Some(KeyUsage)` se a extensão 2.5.29.15 existir, senão `None` |
| `extended_key_usage` | OIDs da extensão 2.5.29.37, na ordem do certificado; vazio se ausente |
| `policies` | OIDs de política da extensão 2.5.29.32, na ordem; vazio se ausente |
| `is_ca` | `cA` de BasicConstraints (2.5.29.19); `false` se ausente |
| `icp_brasil` | `Some` se for certificado ICP-Brasil (seção 6.3) |
| `qualified` | `Some` se a extensão qcStatements (1.3.6.1.5.5.7.1.3) existir (seção 6.4) |

### 6.1 `DistinguishedName`

`{ common_name: Option<String>, organization: Option<String>, organizational_units: Vec<String>, country: Option<String> }`

- CN (2.5.4.3), O (2.5.4.10), OU (2.5.4.11, todos, na ordem), C (2.5.4.6).
- Se um atributo de valor único aparecer mais de uma vez, vale o primeiro.
- Strings: UTF8String, PrintableString e IA5String como UTF-8; BMPString como
  UTF-16BE; TeletexString como Latin-1. Outros tipos: atributo ignorado.

### 6.2 `PublicKeyKind` e `KeyUsage`

`enum PublicKeyKind { Rsa { bits: u32 }, Ec { curve: Curve }, Unsupported { oid: String } }`

- RSA (`1.2.840.113549.1.1.1` e RSA-PSS `1.2.840.113549.1.1.10`): `bits` = tamanho em
  bits do módulo (ex.: 2048, 3072, 4096; módulo de 2047 bits → 2047).
- EC (`1.2.840.10045.2.1`) com curva nomeada conhecida → `Ec`. Curva desconhecida →
  `Unsupported { oid: <OID da curva> }`.
- Qualquer outro algoritmo → `Unsupported { oid: <OID do algoritmo> }`.
- `supports(alg)`: RSA aceita `RsaPkcs1v15` e `RsaPss`; EC aceita `Ecdsa`;
  `Unsupported` não aceita nada.

`KeyUsage` tem um `bool` público por bit: `digital_signature`, `non_repudiation`,
`key_encipherment`, `data_encipherment`, `key_agreement`, `key_cert_sign`, `crl_sign`.

### 6.3 ICP-Brasil — `IcpBrasil`

`{ level: Option<IcpLevel>, holder_name: Option<String>, cpf: Option<String>, cnpj: Option<String> }`

É ICP-Brasil (`icp_brasil = Some`) se alguma política começar com `2.16.76.1.2.` **ou**
algum `otherName` do SubjectAltName (2.5.29.17) tiver OID começando com `2.16.76.1.3.`.

- `level`: da primeira política `2.16.76.1.2.<n>.<...>`: n = 1..4 → `A1..A4`;
  101..104 → `S1..S4`; 303 → `T3`; 304 → `T4`; outro n → `Other(n)`. Sem política
  ICP → `None`.
- `holder_name`: CN do titular sem o sufixo `:` + só dígitos, se houver
  (`"ANA BEATRIZ SOUZA:12345678901"` → `"ANA BEATRIZ SOUZA"`). CN ausente → `None`.
- Valor de `otherName`: aceite OCTET STRING, PrintableString, UTF8String ou
  IA5String, lido como texto ASCII.
- `cpf`: do `otherName` 2.16.76.1.3.1 (pessoa física) ou, se ausente, do 2.16.76.1.3.4
  (responsável pela pessoa jurídica). O valor começa com a data de nascimento
  (8 caracteres) seguida do CPF (11): `cpf` = caracteres 8..19. `None` se o valor
  for curto demais, se os 11 não forem todos dígitos ou se forem todos `0`.
- `cnpj`: `otherName` 2.16.76.1.3.3, valor inteiro; tem de ter 14 dígitos e não ser
  todo `0`, senão `None`.
- `masked_cpf()`: `"***.456.789-**"` para o CPF `12345678901` (padrão gov.br);
  `None` sem CPF.
- `formatted_cnpj()`: `"12.345.678/0001-95"` para `12345678000195`; `None` sem CNPJ.

`enum IcpLevel { A1, A2, A3, A4, S1, S2, S3, S4, T3, T4, Other(u32) }`, com
`Display` = `"A1"` … `"T4"` e `Other(n)` → `"ICP-Brasil (n)"`.

### 6.4 Qualificado eIDAS — `Qualified`

`{ compliance: bool, sscd: bool, types: Vec<QcType> }` a partir de qcStatements
(RFC 3739 / ETSI EN 319 412-5): `SEQUENCE OF SEQUENCE { statementId OID, statementInfo ANY OPTIONAL }`.

- `compliance`: existe o statement `0.4.0.1862.1.1` (QcCompliance).
- `sscd`: existe `0.4.0.1862.1.4` (QcSSCD — chave num dispositivo qualificado).
- `types`: do statement `0.4.0.1862.1.6` (QcType), cuja info é `SEQUENCE OF OID`:
  `…1.6.1` → `ESign`, `…1.6.2` → `ESeal`, `…1.6.3` → `Web`; outros OIDs ignorados;
  na ordem do certificado.
- Statements desconhecidos são ignorados.

### 6.5 Métodos de `CertInfo`

- `is_valid_at(t)`: `not_before <= t <= not_after`.
- `can_sign()`: `!is_ca` e (`key_usage` ausente, ou com `digital_signature` ou
  `non_repudiation`). EKU não entra na conta: quem decide é o site.
- `display_name()`: `icp_brasil.holder_name`, senão `subject.common_name`, senão
  `subject.organization`, senão os 16 primeiros caracteres de `fingerprint.to_hex()`.

## 7. `verify` — conferir uma assinatura crua

`verify(cert_der, hash, algorithm, digest, signature) -> Result<(), VerifyError>`

Existe para o kit provar que o que o sistema devolveu é uma assinatura válida no
formato que o SDK promete. Ordem das checagens e erros:

1. Certificado inválido → `VerifyError::Certificate(CertError)`.
2. Digest de tamanho errado → `VerifyError::Digest(DigestLengthError)`.
3. Chave `Unsupported` → `UnsupportedKey`.
4. Algoritmo incompatível com a chave (`!key.supports(algorithm)`) → `KeyMismatch`.
5. Assinatura não confere, ou tem formato/tamanho errado → `InvalidSignature`.

Formatos: RSA = bloco do tamanho do módulo; ECDSA = cru `r ‖ s` com exatamente
`signature_len()` bytes. RSASSA-PSS: MGF1 com o mesmo hash, salt do tamanho do
digest. ECDSA com hash maior ou menor que a curva segue FIPS 186-5 (o digest é
truncado/usado como está), então P-256 + SHA-384 é válido.

## 8. `dedup` — mesmo certificado por dois caminhos

`enum SourceKind { System, Pkcs11 }` — `System` é o repositório do SO
(CNG/CAPI, Keychain/CryptoTokenKit); tem prioridade.

`dedup_by_fingerprint(items: Vec<T>, key: impl Fn(&T) -> (Fingerprint, SourceKind)) -> Vec<Deduped<T>>`

`Deduped<T> { primary: T, alternates: Vec<T> }`

- Agrupa por impressão digital.
- Ordem de saída: ordem da **primeira aparição** de cada impressão digital na entrada.
- `primary`: o primeiro item `System` do grupo (ordem de entrada); se não houver,
  o primeiro item do grupo.
- `alternates`: os demais itens do grupo, na ordem de entrada.
- Itens idênticos não são descartados (viram alternates).
- Entrada vazia → saída vazia.
