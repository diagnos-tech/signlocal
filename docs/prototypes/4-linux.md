# Prova 4 — Linux: assinatura pelo p11-kit e native messaging

**Status:** rascunho · **Resultado (chaves de software):** **SIM** — o PKCS#11 lista, deduplica e assina RSA
(PKCS#1 v1.5 e PSS) e ECDSA (P-256, P-384, P-521) com SHA-256/384/512, pelo módulo direto e pelo
`p11-kit-proxy.so`, e a troca página → extensão → host → SoftHSM2 passou num Chromium real ·
**Resultado (token real):** _a preencher_ · **Decisão:** _depende da matriz da §6_

## Objetivo

1. Assinatura via p11-kit com token real (SafeNet, SafeSign, ePass2003, Watchdata…).
2. Native messaging no Firefox Snap do Ubuntu (§7).

Sem hardware: SoftHSM2, e o que ficou sem prova está registrado na §5.

## 1. Como o kit prova

```sh
cd docs/prototypes/kit
sudo apt-get install -y softhsm2 opensc p11-kit pcscd libpcsclite-dev
cargo build --release -p websign-probe
(cd nm-e2e && npm ci && npx playwright-core install --with-deps chromium)
PROBE_EXE=$PWD/target/release/websign-probe REPORT_PATH=/tmp/report-linux.md bash linux/ci-linux.sh
```

`CHROMIUM=/caminho/do/chrome` usa um Chromium já instalado no lugar do que o Playwright baixa. O CI faz o
mesmo no job `linux` do [workflow](../../.github/workflows/prototypes.yml) e publica o relatório como artefato.

| Arquivo | O que faz |
|---|---|
| [`linux/softhsm-setup.sh`](kit/linux/softhsm-setup.sh) | cria um token SoftHSM2 isolado (diretório e `SOFTHSM2_CONF` próprios) com RSA-2048, EC P-256, P-384 e P-521; chave e certificado com o mesmo `CKA_ID`; idempotente. `SOFTHSM_ALWAYS_AUTH=1` acrescenta uma chave `CKA_ALWAYS_AUTHENTICATE` |
| [`linux/ci-linux.sh`](kit/linux/ci-linux.sh) | roda `list`, `sign`, p11-kit, deduplicação, `devices`, `report` e o teste ponta a ponta; falha se qualquer expectativa falhar; limpa tudo ao sair |
| [`probe/src/keystores/pkcs11/`](kit/probe/src/keystores/pkcs11/) | descoberta, carga única, `list` sem PIN, `sign` (arquivos pequenos, um conceito por arquivo) |
| [`probe/src/devices/`](kit/probe/src/devices/) | USB (`nusb`) e leitores com ATR (`pcsc`) |

Fontes e riscos dos módulos: [pkcs11-modules.md](../research/pkcs11-modules.md). Tokens e ATRs:
[tokens.md](../research/tokens.md).

## 2. Resultado

Ambiente: Ubuntu 24.04.4 (kernel 6.18, x86_64), SoftHSM2 2.6.1, OpenSC 0.25.0~rc1, p11-kit 0.25.3,
libccid 1.5.5, pcscd 2.0.3 (sem leitores), Chromium 141 (Playwright 1194), Node 22.22.2, rustc 1.98.1,
`cryptoki` 0.12.1, `websign-probe` 0.1.0. O script inteiro leva ~5 s.

| # | Pergunta | Resultado | Evidência |
|---|---|---|---|
| 1 | `list` mostra os 4 certificados, marcados como software, com PIN pelo app, sem o rótulo do token? | **SIM** | §3.1 |
| 2 | `sign --all --hash all --pss` assina tudo e cada assinatura confere? | **SIM** — 15 de 15 (RSA 3 hashes × 2 algoritmos, 3 chaves EC × 3 hashes) | §3.2 |
| 3 | PIN errado vira "wrong PIN" e o token avisa que as tentativas estão acabando? | **SIM** | §3.3 |
| 4 | Módulo registrado no p11-kit (diretório do usuário) é descoberto, e registro quebrado é só aviso? | **SIM** | §3.4 |
| 5 | O mesmo certificado por dois módulos aparece uma vez, com "+1 other path"? | **SIM**, nas duas ordens de carga | §3.5 |
| 6 | O `p11-kit-proxy.so` assina sozinho? | **SIM** — 15 de 15 | §3.5 |
| 7 | Chave com `CKA_ALWAYS_AUTHENTICATE` assina? | **SIM** — 6 de 6: `C_SignInit`, `C_Login(CKU_CONTEXT_SPECIFIC)` e `C_Sign` cru pela tabela de funções do módulo (`cryptoki-sys`) | §3.6 |
| 8 | `devices` roda sem hardware, sem `pcscd` e sem `/sys/bus/usb`? | **SIM** — mensagens claras, código 0; com `pcscd` rodando e sem leitor: "none found" | §3.7 |
| 9 | `report` não vaza rótulo do token nem PIN? | **SIM** (o script confere) | §3.8 |
| 10 | Página → extensão → host → SoftHSM2 num Chromium real? | **SIM** — `NM-E2E: PASS`, assinatura verificada | §3.9 |

## 3. Evidência

Saída de `linux/ci-linux.sh` (o texto do provedor foi encurtado com `…` nas linhas; o resto é literal;
não há PIN nem nome de pessoa: o certificado de teste é `WebeSign Test …`).

### 3.1 `list`

```text
> websign-probe list --no-known-modules --no-p11-kit --module /usr/lib/softhsm/libsofthsm2.so
 1. WebeSign Test ec-p384 | certificate | EC P-384 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | 256adb9aabc7534b
 2. WebeSign Test ec-p521 | certificate | EC P-521 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | 3e1a37449c5cdd7d
 3. WebeSign Test rsa-2048 | certificate | RSA-2048 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | 6f0071f15c22c1de
 4. WebeSign Test ec-p256 | certificate | EC P-256 | until 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2; …; key hidden until login, software; PIN by app) | d2e5ac9b7e671211
```

O texto completo do provedor é `SoftHSM v2 (SoftHSM project); slot "SoftHSM slot ID 0x…"; libsofthsm2.so; key hidden
until login`: modelo e fabricante do token, descrição do slot, arquivo do módulo e se a chave é visível sem PIN.
**Nunca o rótulo do token**, que nos cartões brasileiros e europeus costuma ter o nome do titular.

### 3.2 `sign --all --hash all --pss` (o módulo direto)

```text
WebeSign Test rsa-2048 | certificate | RSA-2048 | … | 6f0071f15c22c1de
   OK    SHA-256 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms
   OK    SHA-256 RSASSA-PSS via C_Sign in 7 ms
   OK    SHA-384 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms
   OK    SHA-384 RSASSA-PSS via C_Sign in 3 ms
   OK    SHA-512 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms
   OK    SHA-512 RSASSA-PSS via C_Sign in 3 ms
WebeSign Test ec-p256 | … | d2e5ac9b7e671211
   OK    SHA-256 ECDSA via C_Sign in 2 ms
   OK    SHA-384 ECDSA via C_Sign in 2 ms
   OK    SHA-512 ECDSA via C_Sign in 2 ms
(… P-384 e P-521 idênticos: 3 OK cada; total 15 OK, 0 FAIL, código de saída 0)
```

Cada assinatura é conferida contra o certificado por `probe-core` (RSA: PKCS#1 v1.5 sobre o `DigestInfo`;
PSS com MGF1 do mesmo hash e sal do tamanho do digest; ECDSA em `r ‖ s`). O `CKM_RSA_PKCS` recebe o
`DigestInfo` já montado: os mecanismos `CKM_SHA256_RSA_PKCS` e afins nunca são usados, porque hasheariam o
digest de novo.

### 3.3 PIN errado

```text
> websign-probe sign --cert 6f0071f15c22c1de --pin-env WRONG_PIN --no-known-modules --no-p11-kit --module …
   FAIL  SHA-256 RSASSA-PKCS1-v1_5: wrong PIN
(exit code 1)
```

Logo depois, o `list` do token mostra `…key hidden until login; PIN attempts running low…`: o SoftHSM2 acendeu
`CKF_USER_PIN_COUNT_LOW`, que a interface usa para "restam poucas tentativas"
([ux.md](../ux.md), `pin.incorrect_low`). A próxima assinatura com o PIN certo zera o contador.

### 3.4 p11-kit registrado pelo usuário

Um `~/.config/pkcs11/modules/websign-test.module` aponta para uma cópia do SoftHSM2; um segundo aponta para um
arquivo que não existe:

```text
> websign-probe list --every-path --no-known-modules
warning: pkcs11:libsofthsm2-missing.so: module file not found: /tmp/websign-ci-linux.…/lib/libsofthsm2-missing.so
 1. WebeSign Test ec-p384 | … | pkcs11:libsofthsm2.so (SoftHSM v2; …) | 256adb9aabc7534b
    also via pkcs11:libsofthsm2-registered.so (SoftHSM v2; …)
```

### 3.5 O mesmo certificado por dois módulos

```text
> websign-probe list --no-known-modules --no-p11-kit --module /usr/lib/softhsm/libsofthsm2.so --module /usr/lib/x86_64-linux-gnu/p11-kit-proxy.so
 1. WebeSign Test ec-p384 | certificate | EC P-384 | … | pkcs11:libsofthsm2.so (SoftHSM v2; …) | 256adb9aabc7534b
    (+1 other path(s); --every-path to show)
```

Com `--every-path`: `also via pkcs11:p11-kit-proxy.so (…)`. Quatro certificados, e não oito. O
`p11-kit-proxy.so` sozinho (`--module …/p11-kit-proxy.so`) assina os mesmos 15 casos. O SoftHSM2 carregado
direto **e** pelo proxy no mesmo processo funciona: o segundo `C_Initialize` volta
`CKR_CRYPTOKI_ALREADY_INITIALIZED`, tratado como sucesso.

### 3.6 Chave com `CKA_ALWAYS_AUTHENTICATE`

```text
> websign-probe sign --cert ee8c71007793ea12 --hash all --pss --pin-env WEBSIGN_PROBE_PIN …
   OK    SHA-256 RSASSA-PKCS1-v1_5 via C_Sign in 5 ms
   OK    SHA-256 RSASSA-PSS via C_Sign in 5 ms
   (… 6 de 6, uma linha por hash e algoritmo)
```

O `cryptoki` 0.12 só oferece `C_Sign` junto com o próprio `C_SignInit`, e o PKCS#11 exige o
`C_Login(CKU_CONTEXT_SPECIFIC)` **entre** os dois. Por isso o kit faz `C_SignInit` pelo `cryptoki`, o
login de contexto e um `C_Sign` cru, obtido pelo `C_GetFunctionList` do módulo (o único símbolo que a
especificação obriga a exportar). Uma chamada só, com buffer para qualquer chave: uma consulta de tamanho
antes seria um segundo `C_Sign` depois do login de contexto, que alguns módulos contam como o uso único.
O script exige as 6 combinações.

### 3.7 `devices`

```text
> websign-probe devices
USB smart card readers and tokens:
  USB enumeration failed: /sys/bus/usb/devices/ not found (errno 2)

PC/SC readers:
  the PC/SC service is not running (start pcscd on Linux, or the Smart Card service on Windows)
(exit code 0)
```

Com `pcscd` rodando e sem leitor, a segunda seção diz `none found`. O `--json` traz os mesmos campos
(`usb.devices`, `usb.hidden`, `readers.readers[].atr`, `problem`); número de série USB nunca é lido, e o
que o pcsc-lite acrescenta ao nome do leitor (`[interface] (número de série)`) é cortado.

### 3.8 `report` (trechos)

```text
### Key sources
- `pkcs11:libsofthsm2.so`: opened
### Certificates
| # | Holder | Type | Key | Valid until | Source | Other paths |
| 1 | _hidden_ | certificate | EC P-384 | 2036-09-26 | pkcs11:libsofthsm2.so (SoftHSM v2 (SoftHSM project); slot "SoftHSM slot ID 0x…"; …) | — |
### Devices
_USB enumeration failed: /sys/bus/usb/devices/ not found (errno 2)_
_the PC/SC service is not running (start pcscd on Linux, or the Smart Card service on Windows)_
### Signatures
| 3 | `OK    SHA-256 RSASSA-PKCS1-v1_5 via C_Sign in 3 ms` |
```

### 3.9 Native messaging ponta a ponta (Chromium, SoftHSM2)

```text
> node nm-e2e/run.mjs --probe …/release/websign-probe --expect-sign --chromium /opt/pw-browsers/chromium-1194/chrome-linux/chrome
  NM-E2E: PASS
  … event=request type=list … event=certificates count=4 warnings=0
  … event=sign hash=SHA-256 algorithm=RSASSA-PKCS1-v1_5 digest_bytes=32
  … event=signed api=C_Sign verified=true
```

O host achou o SoftHSM2 sem nenhum `--module`: pela lista de caminhos conhecidos e pelo registro do p11-kit
(`SOFTHSM2_CONF` e `WEBSIGN_PROBE_PIN` chegam ao host pelo ambiente do navegador). Como o host nunca
recebe o PIN pelo navegador, esse teste usa a variável só porque não há janela de PIN no kit.

## 4. O que a prova fixou no código

| Decisão | Por quê |
|---|---|
| **`list` nunca pede PIN**; "tem chave" = existe `CKO_PRIVATE_KEY` com o mesmo `CKA_ID` visível sem login; se o token não lista **nenhuma** chave e exige login (`CKF_LOGIN_REQUIRED`), todo certificado conta; se não exige login, não há chave | O SoftHSM2 e muitos cartões escondem a chave privada até o PIN. A regra de duas etapas evita listar certificados de CA quando o token mostra as chaves; o `provider` diz qual regra valeu (`key visible` / `key hidden until login`) |
| `provider` = modelo e fabricante do token + descrição do slot + arquivo do módulo; **sem rótulo** e sem o número de série USB que o pcsc-lite põe entre parênteses no nome do leitor (a descrição do slot costuma ser esse nome; `devices` corta o mesmo trecho) | Repositório público; o rótulo costuma ter o nome do titular, e o número de série identifica o dispositivo |
| `hardware` = `Some(false)` só se o nome (modelo, fabricante, descrição do slot) diz "softhsm"/"software"; senão `Some(true)` | `CKF_HW_SLOT` não decide: o SoftHSM2 o deixa desligado, mas módulos de cartão reais também |
| Uma sessão somente leitura por assinatura, `C_Login(CKU_USER)` com o PIN direto do `SecretString` (sem cópia), logout (também quando a assinatura falha) e fechamento ao fim | Nenhum PIN em cache no probe. O app, pela [decisão D5](../plan.md), manterá a sessão aberta: isso muda o `Keystore`, não o `list`/`sign` daqui |
| Slot muda? Acha o certificado de novo pelo **DER** (locator só é o primeiro palpite) e a chave pelo `CKA_ID` | Numeração de slots muda com a ordem dos leitores |
| Mecanismo: RSA v1.5 = `CKM_RSA_PKCS` sobre `DigestInfo`; PSS = `CKM_RSA_PKCS_PSS` com hash, MGF1 e `sLen` = tamanho do digest; ECDSA = `CKM_ECDSA` (aceita `r ‖ s`; converte DER; recusa qualquer outro tamanho). `C_GetMechanismInfo` antes: `Unsupported` se o token não assina com ele | Assinatura correta sobre o digest, nunca sobre o hash do hash |
| Erros: `CKR_PIN_INCORRECT`/`PIN_INVALID`/`PIN_LEN_RANGE` → `WrongPin`; `PIN_LOCKED` → `PinLocked`; `FUNCTION_CANCELED`/`CANCEL`/`FUNCTION_REJECTED` → `Cancelled`; `USER_NOT_LOGGED_IN` sem PIN → `PinRequired`; o resto → `Native { api, code, message }` com o nome `CKR_*` | A interface reage aos tipos; suporte precisa do código |
| Carga única por arquivo (inode) e nunca `C_Finalize`; falha de carga = `SourceFailure`, nunca aborta | `C_Initialize` pode levar 1 s; descarregar biblioteca com threads derruba o processo |
| `provider` avisa `PIN attempts running low` / `last PIN attempt` / `PIN locked` quando o token informa | A UX de tentativas restantes usa essas flags. Falta um campo em `KeystoreError::WrongPin` para levá-las até a janela (`model.rs` é do orquestrador) |

## 5. O que ficou sem prova

| Item | Por que falta | Como provar |
|---|---|---|
| **Token real** por p11-kit ou caminho conhecido (SafeNet 5110, SafeSign, ePass2003, Watchdata, DXToken) | sem hardware | roteiro da §6 |
| **`CKA_ALWAYS_AUTHENTICATE` com cartão real** (Cartão de Cidadão, DNIe, cartão da Estônia) | provado só no SoftHSM2 (§3.6); cartões reais podem abrir o próprio diálogo de PIN no login de contexto | Cartão de Cidadão/DNIe reais, roteiro da §6 |
| **PIN pad** (`CKF_PROTECTED_AUTHENTICATION_PATH`) | o SoftHSM2 não tem; o código passa `NULL` ao `C_Login` e `pin: App { protected_path: true }` ao chamador | leitor com teclado (GemPCPinpad `08e6:3478`) |
| Token com `CKF_CLOCK_ON_TOKEN` e hora inválida | o `cryptoki` recusa `C_GetTokenInfo` e o slot some da lista | token real; se ocorrer, ler o `CK_TOKEN_INFO` cru |
| `pcscd` com leitor e cartão (ATR de verdade, `devices` com USB) | o contêiner não tem `/sys/bus/usb`; testei `pcscd` sem leitor | `websign-probe devices --all-usb` numa máquina com leitor |
| Módulos que abrem diálogo próprio de PIN durante `C_Sign` | sem token | Cartão de Cidadão/DNIe reais |
| Fedora/Arch e `aarch64` | só Ubuntu 24.04 x86_64; os caminhos `lib64` são convenção | CI em contêiner Fedora |
| `libpcsclite.so.1` é dependência dinâmica do binário (`ldd`) | o `devices` usa `pcsc`; sem pcsc-lite o host nem inicia, nem para PKCS#11 | `Depends: libpcsclite1` no `.deb` e `pcsc-lite-libs` no `.rpm`, ou carregar a biblioteca em tempo de execução |

## 6. Roteiro com token real (Linux)

1. `sudo apt install pcscd opensc p11-kit` e o middleware do fabricante ([tokens.md](../research/tokens.md)); `sudo systemctl start pcscd`.
2. Baixar `websign-probe` do artefato `report-linux` do CI (ou compilar).
3. `websign-probe devices --all-usb` → anotar `VID:PID` e ATR na matriz.
4. `websign-probe list` → o token aparece? por qual módulo? (`--every-path` mostra todos).
5. `PIN=… websign-probe sign --cert <impressão digital> --hash all --pss --pin-env PIN` (o PIN vem de uma
   variável **só neste teste**; sem `--pin-env` o probe pergunta no terminal). Cuidado: cada PIN errado gasta
   uma tentativa.
6. `websign-probe report --run-signatures --all --hash all --pss --pin-env PIN --out relatorio.md` e colar aqui
   (o relatório não tem nome, CPF/CNPJ nem número de série).

| Token / cartão | Módulo (arquivo) | Registrado no p11-kit? | `list` | RSA v1.5 | RSA-PSS | ECDSA | PIN errado | Observações |
|---|---|---|---|---|---|---|---|---|
| _SafeNet eToken 5110_ | | | | | | | | |
| _SafeSign / StarSign_ | | | | | | | | |
| _ePass2003_ | | | | | | | | |
| _Watchdata ProxKey_ | | | | | | | | |
| _DXToken_ | | | | | | | | |
| _Cartão de Cidadão_ | | | | | | | | `CKA_ALWAYS_AUTHENTICATE` |

## 7. Native messaging

| Navegador | Resultado | Evidência | Falta |
|---|---|---|---|
| Chromium (deb/Playwright) | **SIM**: ping, validação de digest e assinatura pelo SoftHSM2 conferida pelo host (`verified=true`) | `NM-E2E: PASS` no `ci-linux.sh` (neste container); log do host em §3.9 | — |
| Google Chrome, Edge, Brave, Vivaldi (deb) | **Provável SIM**: mesmo mecanismo do Chromium; o `register` grava em cada `~/.config/<navegador>/NativeMessagingHosts/` | [research/native-messaging.md](../research/native-messaging.md) §3 | Rodar o roteiro abaixo com o navegador instalado |
| Chromium Snap | **Provável SIM**: o Snap lê `~/snap/chromium/common/chromium/NativeMessagingHosts/` (o `register` grava lá) | idem, §3.3 | Ubuntu com Snap |
| Firefox (deb/rpm) | **Provável SIM**: manifesto em `~/.mozilla/native-messaging-hosts/`, `allowed_extensions` com o ID fixo | idem, §3.1 | Rodar o roteiro abaixo (o Playwright não carrega extensão no Firefox) |
| **Firefox Snap (Ubuntu)** | **Sem prova ainda.** O Firefox confinado não lê manifestos: pede ao portal `org.freedesktop.portal.WebExtensions` (patch do Ubuntu), que inicia o host **fora** do Snap com o ambiente do portal | idem, §3.4 | Ubuntu 24.04 com interface gráfica: a janela egui abre com o ambiente do portal? O diálogo "permitir" do portal aparece uma vez? |

**Consequências para o produto** (já no [plano](../plan.md)):

- O pacote `.deb`/`.rpm` instala os manifestos **de sistema** (`/etc/opt/chrome/native-messaging-hosts/`,
  `/etc/chromium/native-messaging-hosts/`, `/usr/lib/mozilla/native-messaging-hosts/` …), com `path` absoluto.
  É o que o portal do Firefox Snap consulta.
- A janela de Confirmação não pode depender de variáveis herdadas do navegador. Quando o host é iniciado
  pelo portal, o ambiente gráfico vem da sessão.
- O substituto do portal (`org.freedesktop.NativeMessagingProxy`, Firefox 157+) não exige mudança no host;
  acompanhar quando o Ubuntu o ligar no Snap estável.

**Roteiro (Ubuntu 24.04 com interface gráfica, Firefox Snap padrão):**

1. `./websign-probe register --browser firefox` e, para o portal, também o manifesto de sistema:
   `sudo install -Dm644 ~/.mozilla/native-messaging-hosts/dev.websign.host.json /usr/lib/mozilla/native-messaging-hosts/dev.websign.host.json`.
2. `about:debugging` → "Carregar extensão temporária" → `kit/extension/manifest.json`.
3. Abrir `kit/nm-e2e/page.html` servido em `http://localhost:8000` (`python3 -m http.server` na pasta).
4. Anotar: o diálogo do portal apareceu? A página mostrou `pong`? O log `/tmp/websign-probe-host.log`
   tem `family=firefox`? Com o token, o `sign` devolve `verified=true`?

## 8. Decisão

- **PKCS#11 pelo p11-kit e por caminhos conhecidos: sim**, com chaves de software. A descoberta, a
  deduplicação por arquivo e por certificado e as assinaturas RSA/PSS/ECDSA estão provadas; o que resta é
  token real (§6) e `CKA_ALWAYS_AUTHENTICATE` com cartão real (§5).
- **Dependência adotada no kit:** `cryptoki-sys = "0.5"` (a mesma versão que o `cryptoki` 0.12 usa) para o
  `C_Sign` cru das chaves de assinatura qualificada (§3.6). Sai quando o `cryptoki` oferecer `C_Sign` sem
  `C_SignInit`.
- **Pacote Linux:** `libpcsclite1` obrigatório (§5) e `p11-kit` recomendado.
