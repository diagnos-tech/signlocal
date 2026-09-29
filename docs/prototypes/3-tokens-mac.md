# Prova 3 — Tokens no Mac: CryptoTokenKit ou PKCS#11?

**Status:** falta token real · **Resultado parcial:** PKCS#11 **não** carrega dentro da sandbox (CI) → token só-PKCS#11 exige o complemento ·
**Decisão:** _depende da matriz abaixo_ — o critério está em [§1](#1-critério-o-complemento-dmg-existe)

## Objetivo

1. Quais middlewares expõem o token ao **CryptoTokenKit** (CTK), isto é, fazem as chaves aparecerem
   no keychain para qualquer app — inclusive o da Mac App Store — sem PKCS#11?
2. Um módulo **PKCS#11** carrega e assina **dentro da sandbox** da Mac App Store?
3. Com isso: o **complemento `.dmg`** (Developer ID + notarização + Sparkle, fora da sandbox,
   chamado pelo app da loja) precisa existir?

Pelo lado do app, os dois caminhos já estão no kit: `macos:ctk` (consulta com
`kSecAttrAccessGroupToken`, ver [prova 2](2-mac.md#1-como-o-kit-prova)) e `pkcs11:<módulo>` (lista
de caminhos conhecidos, p11-kit e `--module`). O mesmo certificado visto pelos dois caminhos é
deduplicado pela impressão digital, preferindo o do sistema.

## 1. Critério: o complemento `.dmg` existe?

O complemento existe **se e somente se** houver ao menos um middleware **relevante** para o qual:

1. o token **não** aparece em `macos:ctk` (token inserido, middleware atual instalado, macOS
   suportado pelo fabricante); **e**
2. o PKCS#11 **não** resolve dentro do app da loja — porque falha na sandbox (`dlopen`,
   `C_Initialize`, `C_Login` ou `C_Sign`; ver §3) **ou** porque a App Review recusa o carregamento de
   módulo externo (diretriz 2.5.2: o app não pode executar código que não veio no pacote).

"Relevante" = usado por clientes do Diagnos (ICP-Brasil A3) ou pelos cartões europeus da lista.
`TODO(gustavo)`: fechar a lista com a participação de mercado de cada token entre os médicos.

Consequências:

- Todos os relevantes passam por CTK → **sem complemento**; PKCS#11 no Mac fica só para o
  diagnóstico ("adicionar módulo").
- Algum só funciona por PKCS#11 **e** o PKCS#11 funciona na sandbox **e** a Review aceita →
  **sem complemento**; o app da loja carrega o módulo.
- Caso contrário → **complemento**, e o `devices.json` marca `macos.cryptotokenkit: false` para
  esse dispositivo (é o que dispara a sugestão na interface, ver `docs/ux.md` §7).

Mesmo com CTK funcionando, vale medir se o PIN do CTK é aceitável para o médico (diálogo do sistema
a cada assinatura ou cache por sessão, conforme o driver).

## 2. Matriz de middlewares

Legenda: **SIM** = documentado pelo fabricante; **provável** = indícios públicos; **?** = sem
informação; "Prova" = resultado do roteiro §4 num Mac real (a preencher).

| Middleware (tokens) | Expõe via CTK? | Módulo PKCS#11 no macOS | Fonte | Prova |
|---|---|---|---|---|
| **SafeSign IC** (A.E.T. Europe) — tokens e cartões JCOP e G+D usados na ICP-Brasil | **SIM** desde a 4.0: *Smart Card Extension* `aetsce.appex` dentro de `tokenadmin.app/Contents/PlugIns` ("used for Apple (native) applications, such as Safari and Mail"; testado com Chrome 111 no macOS 13.2) | `/Applications/tokenadmin.app/Contents/Frameworks/libaetpkss.dylib` (4.0); versões antigas: `/usr/local/lib/libaetpkss.dylib` (a confirmar) | *SafeSign IC Standard Version 4.0 for macOS Release Document* (mar/2023, espelho KPN); notas 3.5/3.6/3.7/4.1/4.2 no UZI-register | |
| **SafeNet Authentication Client** (Thales) — eToken 5110/5110+/5110 CC, IDPrime | **provável** (relatos de certificados do token aparecendo no keychain no macOS recente; não achei documento da Thales citando CTK) | `/usr/local/lib/libeTPkcs11.dylib` | KB da DigiCert (módulo para o Acrobat); anúncio SAC 10.9 para Mac (jan/2025, macOS 15) | |
| **OpenSC** — ePass2003 (driver `epass2003`), DNIe, PIV, muitos cartões | **SIM** no pacote oficial: `OpenSCToken.app` (extensão `org.opensc-project.mac.opensctoken.OpenSCTokenApp.OpenSCToken`). A fórmula do Homebrew não traz o token CTK (a confirmar) | `/Library/OpenSC/lib/opensc-pkcs11.so` (e link em `/usr/local/lib/opensc-pkcs11.so`) | `MacOSX/build` e `MacOSX/opensc-uninstall` no repositório do OpenSC | |
| **Feitian ePass2003** com middleware próprio (distribuído por ACs) | ? | ? (a confirmar no instalador) | — | |
| **Watchdata** (tokens WD usados por ACs brasileiras) | ? | Variante indiana (ProxKey): `/usr/local/lib/wdProxKeyUsbKeyTool/libwdpkcs_Proxkey.dylib`; variante brasileira: ? | Guias de revendedores indianos (Acrobat no Mac) | |
| **G+D StarSign** (StarSign Crypto USB Token S) | **SIM via SafeSign** (o token está na lista de suportados do SafeSign IC 4.0 para macOS); middleware próprio da G+D: ? | via SafeSign: `libaetpkss.dylib` | Release Document SafeSign 4.0 (seção 7) | |
| **Cartão de Cidadão PT** (Autenticação.gov) | **SIM** desde a 3.11.0: módulo `PteidToken` ("implementa a framework CryptoTokenKit") | `/usr/local/lib/libpteidpkcs11.dylib` | Manual de Utilização da Autenticação.gov. O CC emitido desde jun/2024 usa **ECDSA** | |
| **DNIe ES** (Polícia Nacional, `libpkcs11-dnie` 1.6.8) | **NÃO** no pacote oficial: só PKCS#11 (o `.pkg` não traz `.appex`); alternativa: OpenSC, que tem driver `dnie` e o `OpenSCToken` | `/Library/Libpkcs11-dnie/lib/libpkcs11-dnie.so` | Payload do `libpkcs11-dnie-1.6.8_arm.pkg` inspecionado (install-location `/Library`; traz `DialogSign.app`) | |
| **Apple PIV** (`com.apple.pivtoken`, embutido) — YubiKey e cartões PIV | **SIM** (nativo) | não precisa | macOS | |

Notas que pesam na decisão:

- **SafeSign exige o entitlement de CTK do app hospedeiro mesmo pelo PKCS#11.** O documento 4.0
  diz: *"If an application (based on PKCS #11) does not have CTK entitlement, the SafeSign PKCS #11
  Library that is loaded by that application does not have this entitlement either"*; há um
  contorno por PC/SC (`EnableMacOSXPCSCLayerFallback`, ligado por padrão, em
  `~/Library/Application Support/safesign/registry`). O app da loja declara
  `com.apple.security.smartcard`, então o caminho CTK da própria biblioteca deve funcionar; o
  contorno lê um arquivo do `$HOME`, que na sandbox é o contêiner — a confirmar.
- **DNIe abre um app auxiliar** (`DialogSign.app`) para o PIN/confirmação. Dentro da sandbox, iniciar
  outro executável herda a sandbox e pode ser negado — risco específico a medir.
- **Brasil:** as ACs distribuem versões próprias (e às vezes antigas) do SafeSign e do SAC. O que
  vale é a versão que o médico recebe da AC, não a última do fabricante — anotar a origem do
  instalador em cada prova.
- **Teste barato do caminho CTK sem middleware:** uma YubiKey 5 com certificado PIV
  (`ykman piv keys generate` + `ykman piv certificates generate`) aparece em `macos:ctk` pelo driver
  nativo da Apple. Prova o código do kit e a sandbox antes de ter os tokens brasileiros na mão.

## 3. PKCS#11 dentro da sandbox

O `kit/macos/sandbox/sandbox-test.sh` (CI) mede com o SoftHSM2 do Homebrew, com o token **dentro do
contêiner** (`~/Library/Containers/dev.websign.app/Data/websign-softhsm`, via `SOFTHSM2_CONF`):

| ID | O que mede | Por quê |
|---|---|---|
| `pkcs11-setup` | Preparar o token no contêiner a partir de fora | No macOS 14+ o sistema protege contêineres de outros apps; se falhar, o script refaz fora do contêiner (aí só o `dlopen` é conclusivo) |
| `pkcs11-load:store` | `list --module libsofthsm2.so` na sandbox, assinado como o build da loja (sem *hardened runtime*) | `dlopen` de biblioteca fora do pacote (`/opt/homebrew`) e `C_Initialize` lendo a configuração |
| `pkcs11-sign:store` | `sign` com `C_Login` + `C_Sign` | O fluxo completo com PIN pelo app |
| `pkcs11-load:hardened` / `pkcs11-sign:hardened` | O mesmo com `--options runtime` | O complemento Developer ID precisa de *hardened runtime*; sem `com.apple.security.cs.disable-library-validation`, a validação de biblioteca deve recusar módulos de outro Team ID — esperado **NÃO**, confirma que o complemento precisa desse entitlement |

O SoftHSM prova as regras da sandbox para arquivos e código, mas não PC/SC: módulo de token real
ainda fala com o leitor (via `com.apple.security.smartcard`), grava logs e lê configuração em
lugares próprios. Só o roteiro §4 com o token real fecha a questão.

Resultado no CI (run [36629289998](https://github.com/diagnos-tech/web-esign/actions/runs/36629289998),
macOS 26.6.2 arm64):

| ID | Resultado | Evidência |
|---|---|---|
| `pkcs11-setup` | SIM | token SoftHSM2 criado dentro do contêiner |
| `pkcs11-load:store` | **NÃO** | `dlopen(/opt/homebrew/opt/softhsm/lib/softhsm/libsofthsm2.so)`: `file system sandbox blocked open()` |
| `pkcs11-sign:store` | **NÃO** | consequência do anterior |
| `pkcs11-load:hardened` / `pkcs11-sign:hardened` | **NÃO** | mesma negação |

Log do kernel: `Sandbox: websign-probe(…) deny(1) file-read-data /opt/homebrew/Cellar/softhsm/2.7.0/lib/softhsm/libsofthsm2.so`.

**Leitura:** o app da loja **não consegue sequer abrir** um módulo PKCS#11 instalado fora do próprio
pacote. A única saída dentro da loja seria uma exceção `temporary-exception.files.absolute-path.read-only`
para cada pasta de fabricante (`/usr/local/lib`, `/Library/…`), que a App Review tende a recusar pela
diretriz 2.5.2 (executar código que não veio no pacote). Então **todo token que só funciona por PKCS#11
no Mac precisa do complemento** (§1). Com o que já se sabe da matriz (§2), isso inclui o **DNIe**.
A pergunta que falta responder com tokens reais é quais middlewares **não** passam pelo CryptoTokenKit.

## 4. Roteiro de teste para o Gustavo

Um bloco por token. Anotar: modelo do token, ATR, middleware + versão + **de onde veio** (site do
fabricante ou da AC), versão do macOS e chip.

Preparação (uma vez): `cd docs/prototypes/kit && cargo build --release -p websign-probe &&
export PROBE_EXE=$PWD/target/release/websign-probe`.

1. **Sistema, sem o probe** (token inserido):
   ```sh
   system_profiler SPSmartCardsDataType      # leitores, tokens, drivers CTK disponíveis
   pluginkit -mAvvv -p com.apple.ctk-tokens   # extensões CTK instaladas
   security list-smartcards                   # token IDs presentes (parte antes do ":" = driver)
   sc_auth identities                         # identidades que o sistema vê no cartão
   ```
2. **Dispositivo:** `"$PROBE_EXE" devices` (VID:PID, leitor, ATR) → vai para o `devices.json`.
3. **Todos os caminhos:** `"$PROBE_EXE" list --every-path` (se o módulo não estiver na lista de
   conhecidos: `--module <caminho da dylib>`). Esperado com CTK: linha `macos:ctk (<driver>,
   hardware; PIN by OS)`; com PKCS#11: `also via pkcs11:<módulo>`.
4. **Assinar via CTK:** `"$PROBE_EXE" sign --cert <16 hex> --hash all --pss`. O PIN é pedido pelo
   sistema/driver. Testar também **Cancelar** (esperado: "cancelled by the user"). **Não** testar PIN
   errado mais de uma vez: o token bloqueia.
5. **Assinar via PKCS#11:** `read -rs WEBSIGN_PIN && export WEBSIGN_PIN` e
   `"$PROBE_EXE" sign --cert <16 hex> --every-path --pin-env WEBSIGN_PIN --hash sha256`.
6. **Dentro da sandbox** (o que o app da loja conseguiria):
   ```sh
   bash macos/sandbox/run-sandboxed.sh list --every-path
   bash macos/sandbox/run-sandboxed.sh sign --cert <16 hex> --hash sha256
   bash macos/sandbox/run-sandboxed.sh list --module <dylib> --no-known-modules --no-p11-kit
   bash macos/sandbox/run-sandboxed.sh sign --cert <16 hex> --every-path --pin-env WEBSIGN_PIN --module <dylib>
   HARDENED=1 bash macos/sandbox/run-sandboxed.sh list --module <dylib>   # como o complemento
   ```
   O script imprime as negações da sandbox no fim; copiar todas.
7. **Pelo navegador:** passo 6 da [prova 2](2-mac.md#7-roteiro-para-o-gustavo-mac-real) com esse
   token (Chrome iniciando o host sandboxed).
8. **Relatório:** `"$PROBE_EXE" report --run-signatures --cert <fp> --every-path --hash all --pss --out token-<modelo>.md`
   (sem nomes, CPF ou números de série) e preencher a linha da matriz §2.

## 5. Referências

- SafeSign IC Standard 4.0 for macOS, Release Document (A.E.T. Europe, mar/2023), espelho em
  `certificaat.kpn.com/files/drivers/SafeSign/`; versões 3.5–4.2 em `uziregister.nl`.
- Thales: anúncios do SAC 10.8 R2 e 10.9 para Mac em `data-protection-updates.gemalto.com`;
  DigiCert KB "SafeNet hardware token not detected in Adobe Reader on Mac OS".
- OpenSC: `MacOSX/build`, `MacOSX/opensc-uninstall` (github.com/OpenSC/OpenSC).
- Autenticação.gov: Manual de Utilização (amagovpt.github.io/docs.autenticacao.gov).
- DNIe: dnielectronico.es, área de downloads → "Software para Sistemas MacOS" (1.6.8).
- Apple: `com.apple.security.smartcard` (necessário para `TKSmartCardSlotManager` e para PC/SC na
  sandbox); `kSecAttrAccessGroupToken` (concedido por padrão a todo app).
