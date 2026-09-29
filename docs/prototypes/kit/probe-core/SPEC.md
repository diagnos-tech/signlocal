# probe-core — especificação

Biblioteca Rust **pura** (sem chamadas ao sistema operacional) com as peças que
todas as origens de chave compartilham. É a fonte de verdade para os testes
(`tests/`, um arquivo por seção ou subseção, sobre fixtures do OpenSSL) e para a
implementação (`src/`, cujos testes de unidade cobrem o que o OpenSSL não gera).
Toda mudança de comportamento começa aqui. Se, ao escrever teste ou código, esta
especificação for omissa, registre a suposição num comentário `// SPEC:` para o
revisor conciliar; depois de conciliada, a regra entra neste arquivo e o
comentário sai.

**Não mude assinaturas públicas.** Tipos, variantes e campos públicos são
contrato. Módulos privados, à vontade.

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
| `FromStr` | aceita, sem diferenciar maiúsculas **ASCII**, `SHA-256`/`SHA256`, `SHA-384`/`SHA384`, `SHA-512`/`SHA512`. Qualquer outra coisa (inclusive espaços em volta, `SHA-1`, `SHA-224`, `SHA3-256`, vazio, e letras que só viram ASCII pela caixa Unicode, como `ſ` → `S`) → `UnknownAlgorithmError { name }` com a string original |

`DigestLengthError` exibe: `digest for SHA-256 must be 32 bytes, got 31`.

## 2. `algorithm` — `SignatureAlgorithm`

`enum SignatureAlgorithm { Ecdsa, RsaPkcs1v15, RsaPss }`

| Método | Comportamento |
|---|---|
| `ALL` | `[Ecdsa, RsaPkcs1v15, RsaPss]` |
| `name()` | `"ECDSA"`, `"RSASSA-PKCS1-v1_5"`, `"RSASSA-PSS"` (nomes do WebCrypto) |
| `is_rsa()` | `false`, `true`, `true` |
| `Display` | igual a `name()` |
| `FromStr` | aceita exatamente os três nomes, sem diferenciar maiúsculas ASCII. Outros → `UnknownAlgorithmError { name }` |

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

- Estrutura exigida: `30 len 02 len r 02 len s`, nada depois de `s` dentro da
  SEQUENCE nem depois do fim da SEQUENCE.
- Comprimentos em forma curta ou longa de 1 byte (`81 xx`, necessário na P-521;
  `81 xx` com `xx < 0x80`, não mínimo, também vale). Forma indefinida (`80`) ou
  longa com mais bytes → `Malformed`.
- INTEGER negativo (primeiro byte de conteúdo ≥ `0x80`) ou vazio → `Malformed`.
- Zeros à esquerda a mais são **tolerados** (alguns tokens não geram DER mínimo):
  remova-os.
- `r` ou `s` igual a zero → `Malformed`.
- Valor que, sem zeros à esquerda, não cabe em `field_len` bytes → `IntegerTooLarge`.
- Ordem: a estrutura inteira (inclusive o sinal e o zero de `r` **e** de `s`) é
  conferida antes dos tamanhos, então um defeito de estrutura dá `Malformed`
  mesmo que o outro inteiro seja grande demais.
- Não compara `r`/`s` com a ordem da curva: é conversão de formato, não
  validação (quem valida é `verify`).
- Saída: `r` e `s` preenchidos à esquerda até `field_len`, concatenados.

`raw_to_der(raw, curve) -> Result<Vec<u8>, EcdsaEncodingError>`

- `raw.len() != signature_len()` → `WrongLength { expected, actual }`.
- Cada metade vira INTEGER DER **mínimo**: remove zeros à esquerda (mantendo ao
  menos um byte) e acrescenta `00` se o primeiro byte restante for ≥ `0x80`.
- Usa forma longa `81 xx` quando o conteúdo da SEQUENCE passar de 127 bytes.
- Não valida valores: metade toda zero vira `02 01 00`, e valores ≥ ordem da
  curva passam.
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
| `FromStr` | ignora `:` e o espaço ASCII (U+0020) em qualquer posição; o resto tem de ser exatamente 64 dígitos hexadecimais ASCII (maiúsculos ou minúsculos). Qualquer outro caractere, inclusive tab, quebra de linha e outros espaços Unicode → `ParseFingerprintError` |

Deriva `Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord`.

## 6. `cert` — resumo de certificado X.509

`CertInfo::from_der(der) -> Result<CertInfo, CertError>`

Bytes que não têm a estrutura de um `Certificate` X.509 → `CertError::Malformed(<o
que quebrou>: <detalhe>)`. Extensão conhecida mal formada também → `Malformed`.
Extensões desconhecidas são ignoradas sem olhar o conteúdo, mesmo críticas (isto é
um resumo para a tela, não validação de caminho). A assinatura do certificado não
é conferida.

### 6.0 Leitura tolerante

Nenhum certificado real pode sumir da lista por detalhe de codificação que não muda
o resumo. Por isso a leitura aceita estes desvios de DER/RFC 5280, que existem em
certificados emitidos:

- comprimento em forma longa não mínima (1 a 4 octetos de comprimento);
- INTEGER não mínimo; número de série de qualquer tamanho (inclusive > 20
  octetos), zero ou negativo; módulo RSA sem o byte de sinal (lido sem sinal);
- BOOLEAN com qualquer octeto não nulo = TRUE (BER); valores DEFAULT escritos
  (versão v1 explícita, `critical FALSE`, `cA FALSE`); versão ausente ou qualquer;
- RDN multivalorado fora da ordem do DER; `issuerUniqueID`/`subjectUniqueID`;
- GeneralizedTime antes de 2050;
- extensão repetida (proibida pelo RFC 5280): vale a primeira;
- OID com arcos de até 128 bits (os UUID de `2.25` são os maiores em uso). OID
  que não dá para escrever (codificação inválida ou arco maior) é ignorado onde
  não precisa ser impresso (tipo de extensão, de atributo de nome, de otherName,
  de statement e de QcType) e é `Malformed` onde precisa (EKU, política,
  algoritmo da chave).

Continua `Malformed`: forma indefinida (`80`) ou comprimento com mais de 4
octetos, tag de mais de um octeto, valor truncado, bytes sobrando (no fim ou
dentro de uma estrutura lida), campo desconhecido ou fora de ordem no
TBSCertificate, série vazia, tempo fora das formas da tabela abaixo
(`not_before`), BIT STRING da chave RSA/EC com bits não usados, `RSAPublicKey`
ilegível.

Campos de `CertInfo`:

| Campo | Conteúdo |
|---|---|
| `fingerprint` | `Fingerprint::of(der)` |
| `subject`, `issuer` | `DistinguishedName` (abaixo) |
| `serial_hex` | hex minúsculo dos octetos de conteúdo do número de série sem os `00` à esquerda, sobrando ao menos um octeto (ex.: `00 80` → `"80"`; `12 34` → `"1234"`; `01 02` → `"0102"`; `00` → `"00"`; `00 00 05` → `"05"`). Série negativa fica em complemento de dois, como Windows e macOS mostram (`ff 7f` → `"ff7f"`) |
| `not_before`, `not_after` | segundos Unix, negativos antes de 1970. Só as formas do RFC 5280 §4.1.2.5: UTCTime `AAMMDDHHMMSSZ` (AA 50–99 → 19AA, 00–49 → 20AA) e GeneralizedTime `AAAAMMDDHHMMSSZ` em qualquer ano. Sem segundos, com fração, com fuso ou com data/hora impossível (30/02, 24h, 60 s) → `Malformed` |
| `key` | `PublicKeyKind` |
| `key_usage` | `Some(KeyUsage)` se a extensão 2.5.29.15 existir (mesmo sem nenhum bit ligado, ou só com bits fora da struct), senão `None` |
| `extended_key_usage` | OIDs da extensão 2.5.29.37, na ordem do certificado; vazio se ausente ou vazia |
| `policies` | OIDs de política da extensão 2.5.29.32, na ordem; vazio se ausente |
| `is_ca` | `cA` de BasicConstraints (2.5.29.19); `false` se ausente |
| `icp_brasil` | `Some` se for certificado ICP-Brasil (seção 6.3) |
| `qualified` | `Some` se a extensão qcStatements (1.3.6.1.5.5.7.1.3) existir (seção 6.4) |

### 6.1 `DistinguishedName`

`{ common_name: Option<String>, organization: Option<String>, organizational_units: Vec<String>, country: Option<String> }`

- CN (2.5.4.3), O (2.5.4.10), OU (2.5.4.11, todos, na ordem), C (2.5.4.6).
- Strings: UTF8String, PrintableString e IA5String como UTF-8; BMPString como
  UTF-16BE; TeletexString como Latin-1. Outros tipos (NumericString,
  UniversalString, VisibleString…) e bytes inválidos para o tipo (UTF-8 quebrado,
  BMPString de tamanho ímpar ou com surrogate solto): só esse atributo é
  ignorado, como se não existisse.
- Se um atributo de valor único aparecer mais de uma vez, vale o primeiro
  **legível** (um CN ignorado pela regra acima não esconde o CN seguinte).

### 6.2 `PublicKeyKind` e `KeyUsage`

`enum PublicKeyKind { Rsa { bits: u32 }, Ec { curve: Curve }, Unsupported { oid: String } }`

- RSA (`1.2.840.113549.1.1.1` e RSA-PSS `1.2.840.113549.1.1.10`): `bits` = tamanho em
  bits do módulo (ex.: 2048, 3072, 4096; módulo de 2047 bits → 2047).
  Os parâmetros de uma chave RSA-PSS são ignorados, e ela aceita também
  `RsaPkcs1v15` em `supports` (a API não distingue as duas; o RFC 4055 restringe
  essa chave ao PSS, mas tokens com ela são raros).
- EC (`1.2.840.10045.2.1`) com curva nomeada conhecida → `Ec`. Curva nomeada
  desconhecida → `Unsupported { oid: <OID da curva> }`. Sem curva nomeada
  (parâmetros explícitos, `NULL` ou ausentes) → `Unsupported { oid:
  "1.2.840.10045.2.1" }`, já que não há OID de curva para mostrar.
- Qualquer outro algoritmo → `Unsupported { oid: <OID do algoritmo> }`.
- O ponto EC e o par RSA não são validados aqui (isso exige aritmética da
  curva); `verify` os recusa (seção 7).
- `supports(alg)`: RSA aceita `RsaPkcs1v15` e `RsaPss`; EC aceita `Ecdsa`;
  `Unsupported` não aceita nada.

`KeyUsage` tem um `bool` público por bit: `digital_signature`, `non_repudiation`,
`key_encipherment`, `data_encipherment`, `key_agreement`, `key_cert_sign`, `crl_sign`.

### 6.3 ICP-Brasil — `IcpBrasil`

`{ level: Option<IcpLevel>, holder_name: Option<String>, cpf: Option<String>, cnpj: Option<String> }`

É ICP-Brasil (`icp_brasil = Some`) se alguma política começar com `2.16.76.1.2.` **ou**
algum `otherName` do SubjectAltName (2.5.29.17) tiver OID começando com `2.16.76.1.3.`.

- `level`: da primeira política que começa com `2.16.76.1.2.`, lendo o arco `n`
  logo depois do prefixo (`2.16.76.1.2.3.1` e também `2.16.76.1.2.3`, sem mais
  arcos, dão n = 3): n = 1..4 → `A1..A4`; 101..104 → `S1..S4`; 303 → `T3`;
  304 → `T4`; outro n → `Other(n)`. `n` que não cabe em `u32` → `None` (o
  certificado continua ICP-Brasil; as políticas seguintes não são consultadas).
  Sem política ICP → `None`.
- `holder_name`: CN do titular sem o **último** `:` quando ele é seguido de um ou
  mais dígitos ASCII e nada mais (`"ANA BEATRIZ SOUZA:12345678901"` → `"ANA
  BEATRIZ SOUZA"`; `"R2:D2:123"` → `"R2:D2"`; `"A:1:2"` → `"A:1"`; `"MARIA:"`,
  `"MARIA:12A"` e `"MARIA: 12"` ficam como estão). CN ausente → `None`.
- Valor de `otherName`: aceite OCTET STRING, PrintableString, UTF8String ou
  IA5String com conteúdo só ASCII. Outro tipo (BMPString, INTEGER…) ou byte não
  ASCII: o otherName existe, mas o valor é ilegível.
- `cpf`: do `otherName` 2.16.76.1.3.1 (pessoa física) ou, **só se ele não
  existir**, do 2.16.76.1.3.4 (responsável pela pessoa jurídica). O valor começa
  com a data de nascimento (8 caracteres, não conferidos) seguida do CPF (11):
  `cpf` = caracteres 8..19. `None` se o valor for ilegível, curto demais, se os 11
  não forem todos dígitos ou se forem todos `0`. Um 2.16.76.1.3.1 presente e
  inválido dá `None` e **não** cai para o 2.16.76.1.3.4: o DOC-ICP-04 preenche com
  zeros o dado indisponível, e mostrar o CPF do responsável como se fosse do
  titular atribuiria o CPF à pessoa errada.
- `cnpj`: `otherName` 2.16.76.1.3.3, valor inteiro; tem de ter 14 dígitos e não ser
  todo `0`, senão `None`.
- `masked_cpf()`: `"***.456.789-**"` para o CPF `12345678901` (padrão gov.br).
  `None` sem CPF ou se o campo (público, qualquer um pode preenchê-lo) não tiver
  exatamente 11 dígitos ASCII.
- `formatted_cnpj()`: `"12.345.678/0001-95"` para `12345678000195`. `None` sem
  CNPJ ou se o campo não tiver exatamente 14 dígitos ASCII.

`enum IcpLevel { A1, A2, A3, A4, S1, S2, S3, S4, T3, T4, Other(u32) }`, com
`Display` = `"A1"` … `"T4"` e `Other(n)` → `"ICP-Brasil (n)"`.

### 6.4 Qualificado eIDAS — `Qualified`

`{ compliance: bool, sscd: bool, types: Vec<QcType> }` a partir de qcStatements
(RFC 3739 / ETSI EN 319 412-5): `SEQUENCE OF SEQUENCE { statementId OID, statementInfo ANY OPTIONAL }`.

- `compliance`: existe o statement `0.4.0.1862.1.1` (QcCompliance).
- `sscd`: existe `0.4.0.1862.1.4` (QcSSCD — chave num dispositivo qualificado).
- `types`: dos statements `0.4.0.1862.1.6` (QcType), cuja info é `SEQUENCE OF OID`:
  `…1.6.1` → `ESign`, `…1.6.2` → `ESeal`, `…1.6.3` → `Web`; outros OIDs ignorados;
  na ordem do certificado. Havendo mais de um QcType, os tipos de todos entram,
  em ordem e sem remover repetidos. QcType sem info não acrescenta nada; info que
  não é `SEQUENCE OF OID` → `Malformed`.
- Statements desconhecidos são ignorados, e a info dos outros statements não é
  lida.
- A extensão existir basta para `Some`, mesmo vazia ou só com statements
  desconhecidos (`Some(Qualified::default())`).

### 6.5 Métodos de `CertInfo`

- `is_valid_at(t)`: `not_before <= t <= not_after`.
- `can_sign()`: `!is_ca` e (`key_usage` ausente, ou com `digital_signature` ou
  `non_repudiation`). KeyUsage presente sem nenhum desses bits → `false`. EKU não
  entra na conta: quem decide é o site.
- `display_name()`: `icp_brasil.holder_name`, senão `subject.common_name`, senão
  `subject.organization`, senão os 16 primeiros caracteres de `fingerprint.to_hex()`.
  Candidato vazio ou só com espaços é pulado (a janela de confirmação nunca
  mostra nome em branco; um CN `":123"` dá holder `""`, que também é pulado).

## 7. `verify` — conferir uma assinatura crua

`verify(cert_der, hash, algorithm, digest, signature) -> Result<(), VerifyError>`

Existe para o kit provar que o que o sistema devolveu é uma assinatura válida no
formato que o SDK promete. Ordem das checagens e erros:

1. Certificado inválido → `VerifyError::Certificate(CertError)` (o mesmo erro de
   `CertInfo::from_der`).
2. Digest de tamanho errado → `VerifyError::Digest(DigestLengthError)`.
3. Chave `Unsupported` → `UnsupportedKey`.
4. Algoritmo incompatível com a chave (`!key.supports(algorithm)`) → `KeyMismatch`.
5. Chave reconhecida mas inutilizável → `UnsupportedKey`: ponto EC fora da curva,
   identidade ou de tamanho errado; RSA com módulo zero ou par, maior que 8192
   bits, ou expoente par, menor que 3, maior que 2³³−1 ou não menor que o módulo.
6. Assinatura não confere, ou tem formato/tamanho errado → `InvalidSignature`.

Formatos: RSA = bloco com exatamente o tamanho do módulo em bytes e, lido como
inteiro, **menor que o módulo** (RFC 8017 §5.2.2: com um módulo de 2047 bits,
`s + n` ainda cabe no bloco e não pode valer como segunda assinatura). ECDSA = cru
`r ‖ s` com exatamente `signature_len()` bytes, `r` e `s` em `[1, n−1]`; `s` alto
é aceito (CMS/PAdES não exigem `s` baixo). RSASSA-PSS: MGF1 com o mesmo hash e
salt do tamanho do digest, exigidos (sem detectar o salt). ECDSA com hash maior ou
menor que a curva segue FIPS 186-5 (o digest é truncado/usado como está), então
P-256 + SHA-384 e P-521 + SHA-256 são válidos.

`verify` só confere a matemática: não olha validade, KeyUsage, EKU nem a
assinatura do próprio certificado.

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
