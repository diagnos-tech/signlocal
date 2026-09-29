# Dúvidas da SPEC (agente de testes)

Onde a SPEC é ambígua, cada teste marca a suposição com `// SPEC:` (ou no
comentário logo acima do teste). Esta lista é o índice para o revisor conciliar.
"Suposição testada" = há teste que falha se a implementação ler diferente;
"não testado" = deixei de fora de propósito, o teste seria um palpite.

## Suposições testadas

1. **Caixa em `FromStr` (§1, §2)**: "sem diferenciar maiúsculas" foi lido como
   *ASCII*. `"ſha-256"` (U+017F, que vira `S` em `to_uppercase`) deve ser rejeitado.
   Teste: `hash.rs::case_folding_is_ascii_only`.
2. **CPF quando `2.16.76.1.3.1` existe mas é inválido (§6.3)**: "ou, se ausente, do
   2.16.76.1.3.4" foi lido literalmente: só cai para o `.3.4` quando o `.3.1` *não
   existe*; um `.3.1` presente e inválido dá `cpf = None`.
   Teste: `cert_icp_brasil.rs::a_present_but_invalid_person_value_is_not_replaced_by_the_responsible_one`.
3. **Política sem arcos depois de `n` (§6.3)**: `2.16.76.1.2.3` (exatamente isso) começa
   com `2.16.76.1.2.`, então conta como ICP-Brasil e dá `level = A3`.
   Teste: `reads_the_level_from_a_policy_without_further_arcs`.
4. **`otherName` com tipo de valor fora dos quatro aceitos (§6.3)**: BMPString conta como
   ausente (`cpf = None`); o certificado continua ICP-Brasil.
   Teste: `ignores_a_cpf_other_name_of_an_unsupported_string_type`.
5. **Bytes dentro da SEQUENCE depois de `s` (§4)**: "nada depois do fim da SEQUENCE" foi
   lido como incluindo lixo *dentro* do corpo depois do INTEGER `s` → `Malformed`.
6. **`verify` não olha validade nem KeyUsage (§7)**: as cinco checagens listadas não incluem
   nenhuma dos dois; certificado expirado e certificado só `keyEncipherment` com a chave
   certa verificam com sucesso. Teste: `checks_certificate_bytes_only_for_the_key_not_for_validity_or_usage`.
7. **PSS estrito (§7)**: "salt do tamanho do digest, MGF1 com o mesmo hash" foi lido como
   *exigência*: assinaturas com salt 0, salt 20 ou MGF1-SHA1 dão `InvalidSignature`
   (nada de detectar o salt sozinho). Teste: `rejects_pss_signatures_that_do_not_use_the_agreed_parameters`.
8. **ECDSA com digest de tamanho diferente da curva (§7)**: testada a matriz completa
   3 curvas × 3 hashes, com assinaturas do OpenSSL. Digest maior que o campo é truncado
   (bytes além do campo não influenciam: os testes de "digest adulterado" só mexem nos
   bytes que contam); digest menor entra como inteiro (zeros à esquerda), inclusive
   P-521 + SHA-256. Coberto por `accepts_ecdsa_*_with_every_hash`.
9. **Datas antes de 1970 (§6)**: `time-pre-1970` (UTCTime `600101000000Z`, regra do RFC 5280:
   anos 50–99 são 19xx) deve dar `not_before` negativo (-315619200). O tipo `DateTime` da
   crate `der` só vai de 1970 em diante, então quem usar `x509-cert` tem de tratar isto à mão.
10. **`der_to_raw`/`raw_to_der` não checam `r`, `s` contra a ordem da curva (§4)**: "para todo
   `x` com `r` e `s` não nulos" foi lido literalmente; os testes usam `r = s = 2^256-1`.
11. **Nome de atributo ignorado (§6.1)**: NumericString e UniversalString em CN/OU são
    ignorados *sem* derrubar os outros atributos (`dn-numeric`, `dn-universal`).
12. **Chave RSA de 2047 bits (§6.2, §7)**: além de `bits = 2047`, os testes verificam
    PKCS#1 v1.5 e PSS com essa chave (bloco de 256 bytes, `emBits` = 2046 no PSS). É um
    caso extremo; se a `rsa` crate recusar, o revisor decide se derruba o teste.

## Não testado (SPEC omissa)

13. **Espaços em `Fingerprint::from_str`**: "ignora `:` e espaços" só é claro para U+0020.
    Tabulação e quebra de linha ficaram de fora.
14. **Sufixo do CN ICP-Brasil**: `"NOME:"` (dois-pontos sem dígitos) e `"A:1:2"` (qual
    sufixo sai?) não são testados. `"R2:D2:12345678901"` → `"R2:D2"` é.
15. **Arco de política > `u32`** (`2.16.76.1.2.4294967296.1`): só se exige ausência de pânico
    (aceitar ou rejeitar o certificado serve). `u32::MAX` exato → `Other(4294967295)` é testado.
16. **Comprimentos DER não mínimos**: `81 xx` com `xx < 128` (em assinatura ECDSA ou no
    certificado) pode ser aceito ou `Malformed`; a SPEC só proíbe `80` e mais de 1 byte.
17. **Série `00` sozinha** (conteúdo de um byte zero): `"00"` ou `""`? Sem fixture.
18. **KeyUsage presente sem nenhum bit** (BIT STRING vazia): sem fixture.
    `ku-decipher-only` (só o bit 8, fora da struct) dá `Some(all false)` e é testado.
19. **Strings inválidas** (UTF-8 quebrado numa UTF8String, BMPString de tamanho ímpar):
    ignorar o atributo ou `Malformed`? Só se exige ausência de pânico (varredura de bytes).
20. **Primeiro atributo ilegível e segundo válido** (`CN=<Numeric> CN=<UTF8>`): "vale o
    primeiro" é o primeiro *decodificável* ou o primeiro no certificado?
21. **Curva EC com parâmetros explícitos** (SEQUENCE no lugar do OID): qual OID vai em
    `Unsupported`? Sem fixture. Idem chave RSA-PSS com parâmetros restritivos (hash, salt).
22. **Vários `QcType` ou statements repetidos**; `QcType` com `statementInfo` que não é
    `SEQUENCE OF OID`: sem fixture.
23. **Ordem entre `Malformed` e `IntegerTooLarge`** quando os dois se aplicam (por exemplo
    `r` grande demais e `s` negativo): sem teste.
24. **`masked_cpf` / `formatted_cnpj` com campo público fora do padrão** (tamanho errado,
    não ASCII): só se exige ausência de pânico.
