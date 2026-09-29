# Prova 1 — Windows: CNG/CAPI, PIN em primeiro plano e MSIX

**Status:** em andamento · **Resultado:** _a preencher_ · **Decisão:** _a preencher_

## Objetivo

Responder com evidência, antes de construir o app:

1. O repositório de certificados do Windows assina com **qualquer** chave que o usuário tenha:
   minidriver (CNG), CSP legado (CAPI), A1 instalado a partir de `.pfx`, com tokens reais
   (SafeSign, SafeNet, ePass2003, Watchdata)?
2. O diálogo de PIN do sistema/middleware abre **na frente** do navegador
   (`NCRYPT_WINDOW_HANDLE_PROPERTY` / `PP_CLIENT_HWND`)?
3. O MSIX com `unvirtualizedResources` grava o HKCU **real**, e Chrome, Edge e Firefox iniciam o
   host pelo *app execution alias* e trocam mensagens com ele?
4. O pré-registro da extensão pelo app funciona?
5. A janela egui abre rápido em RDP?

Se o MSIX reprovar (item 3 ou a loja recusar a capacidade restrita), o plano B é MSI assinado
pela SignPath.

## Como o kit prova

### Repositório de certificados (`probe/src/keystores/windows/`)

| Arquivo | Papel |
|---|---|
| `store.rs`, `thumbprint.rs` | Abre `CurrentUser\MY` só leitura, com `CERT_STORE_CTRL_AUTO_RESYNC` (tokens entram e saem sem reabrir); enumera; reencontra pela impressão digital SHA-1 (`CERT_FIND_HASH`) |
| `key_info.rs` | Lê `CERT_KEY_PROV_INFO` (provedor, tipo, contêiner, keyspec) **sem abrir a chave** |
| `hardware.rs` | Decide se o provedor é hardware perguntando ao provedor, nunca à chave |
| `acquire.rs` | `CryptAcquireCertificatePrivateKey` com `COMPARE_KEY`, `WINDOW_HANDLE` e `ALLOW`/`PREFER`/`ONLY_NCRYPT_KEY` conforme `--ncrypt` |
| `ncrypt.rs` | `NCryptSignHash`: PKCS#1 v1.5, PSS (salt = tamanho do digest), ECDSA (já sai `r‖s`) |
| `capi.rs` | `CryptCreateHash` + `HP_HASHVAL` + `CryptSignHashW`, bytes invertidos (CAPI devolve little-endian) |
| `errors.rs` | Cancelamento → `Cancelled`; PIN errado → `WrongPin`; bloqueado → `PinLocked`; resto → código + mensagem do sistema |
| `window.rs` | Dono do diálogo de PIN: janela do chamador (o Chrome passa `--parent-window=<HWND>` ao host, mas `0` quando o pedido vem do service worker de uma extensão MV3) ou, sem ela, a do console — que, para um host iniciado pelo navegador, é um console **oculto** (o Chromium inicia o host com `start_hidden`) |

Decisões e porquês:

- **`list` nunca pede PIN.** Só lê propriedades do certificado. Para saber se é hardware, pergunta ao
  *provedor* (não à chave): `NCRYPT_IMPL_TYPE_PROPERTY` no KSP aberto com `NCryptOpenStorageProvider`,
  ou `PP_IMPTYPE` num contexto `CRYPT_VERIFYCONTEXT | CRYPT_SILENT` do CSP. Nenhum dos dois toca no
  cartão. Se o provedor não responder, heurística pelo nome: "smart card" ou "Platform Crypto Provider"
  (TPM) → hardware; demais provedores "Microsoft …" → software; terceiros → desconhecido.
  Consequência aceita: um certificado cujo `KEY_PROV_INFO` aponta para uma chave apagada aparece na
  lista e falha só ao assinar.
- **O que aparece em `provider`:** nome do KSP/CSP, `[CNG]` ou `[CAPI type N, AT_…]`, e o leitor quando
  o contêiner é totalmente qualificado (`\\.\leitor\…`). O nome do contêiner nunca é mostrado: há
  middlewares que o batizam com o nome do titular.
- **Chave aberta uma vez por sessão.** Sem `CRYPT_ACQUIRE_CACHE_FLAG`: o keystore guarda o handle
  (e o certificado, porque um handle em cache do Windows só vive enquanto o certificado vive) e o
  libera no `Drop`. Os middlewares associam o cache de PIN ao handle aberto, então a sessão pede o
  PIN uma vez. Se uma assinatura falha com erro nativo (cartão removido, middleware reiniciado), o
  handle é descartado e a próxima assinatura reabre.
- **`--ncrypt allow` é o que o app vai usar** (CNG quando a chave é CNG, CAPI quando é CSP), igual ao
  Windows faz por padrão. O chrome-token-signing usa `PREFER`, delegando ao Windows a ponte CAPI→CNG;
  a prova roda os três modos para comparar.
- **A1 em CSP `PROV_RSA_FULL`.** O assistente de importação coloca o `.pfx` no
  "Microsoft Enhanced Cryptographic Provider v1.0" (`PROV_RSA_FULL`), que não conhece SHA-2
  (`NTE_BAD_ALGID`). Para os três CSPs de software da Microsoft desse tipo, o kit reabre o mesmo
  contêiner no "Microsoft Enhanced RSA and AES Cryptographic Provider" (`PROV_RSA_AES`), que lê os
  mesmos contêineres — é o que o .NET faz. A saída mostra `CryptSignHash (PROV_RSA_AES)`.
  CSP **de terceiros** `PROV_RSA_FULL` sem SHA-2 não tem esse atalho: o erro vira
  `Unsupported("… try --ncrypt prefer")` e a saída real é PKCS#11. **Sem prova até testar tokens.**
- **PSS e ECDSA via CAPI** → `Unsupported` (CAPI não tem PSS nem ECC).
- **Uma chamada de assinatura, não duas.** Buffer para RSA-16384 e nova tentativa só se o provedor
  disser que é pequeno (como o .NET). Evita uma segunda ida ao cartão.
- **PIN em primeiro plano.** Três camadas: `CRYPT_ACQUIRE_WINDOW_HANDLE_FLAG` ao abrir a chave;
  `NCRYPT_WINDOW_HANDLE_PROPERTY` no handle CNG antes de cada assinatura; `PP_CLIENT_HWND` (contexto
  nulo = processo todo) antes de cada assinatura CAPI.

### MSIX (`msix/`)

- `AppxManifest.xml` (modelo preenchido a partir do `project.toml`): `runFullTrust`;
  `unvirtualizedResources` + `desktop6:RegistryWriteVirtualization` e
  `desktop6:FileSystemWriteVirtualization` desligados (sem isso as escritas em HKCU e em
  `%LOCALAPPDATA%` iriam para a cópia privada do pacote e nenhum navegador as veria);
  `uap5:AppExecutionAlias` com `desktop4:Subsystem="console"` (mesmo padrão do Python da Microsoft
  Store, que funciona com stdin/stdout); protocolo `websign:` que roda `register` (a loja não roda
  script de instalação). Identidade separada do app futuro (`WebeSign.Probe`) e publisher de teste.
- `build-msix.ps1`: layout, PNGs gerados, `makeappx pack` (valida o manifesto), certificado
  autoassinado com Subject = Publisher, `signtool`, confiança em `LocalMachine\TrustedPeople`,
  `Add-AppxPackage`.
- `test-msix.ps1`: apaga as chaves e manifestos do host; roda `register` **pelo alias**; confere
  (a) as chaves no HKCU real via `reg.exe` de fora do pacote, (b) o manifesto no caminho real e
  recém-escrito (se faltar, procura a cópia virtualizada para diagnosticar), (c) `path` = alias e o
  alias executa; (d) opcional: `node nm-e2e/run.mjs --probe <alias> --no-register`.
- `msix_alias_path()` (`probe/src/platform/windows.rs`): com identidade de pacote
  (`GetCurrentPackageFamilyName`), devolve `%LOCALAPPDATA%\Microsoft\WindowsApps\<família>\websign-probe.exe`
  se existir (não pode ser tomado por outro pacote com o mesmo alias), senão o da raiz de `WindowsApps`.

## Evidência do CI (chaves de software)

Workflow: `.github/workflows/prototypes.yml`, job `windows` (`windows-latest`).
Execução: _link do run_ · Commit: _sha_ · Imagem do runner: _versão_

### Casos e o que se espera (`windows/ci-windows.ps1`)

Certificados criados por `windows/make-test-certs.ps1` (removidos no fim):

| Certificado | Provedor | `allow` | `prefer` / `only` |
|---|---|---|---|
| `cng-rsa2048` | Software KSP (CNG) | 6/6 OK via `NCryptSignHash` | idem |
| `cng-p256`, `cng-p384` | Software KSP (CNG) | 3/3 ECDSA OK | idem |
| `capi-aes-rsa2048` | Enhanced RSA and AES CSP, `AT_SIGNATURE` | PKCS#1 3/3 OK via `CryptSignHash`; PSS = `not supported` (**falha esperada**) | observado |
| `a1-pfx-rsa2048` | `.pfx` importado no Enhanced CSP v1.0 (`PROV_RSA_FULL`, `AT_KEYEXCHANGE`) | PKCS#1 3/3 OK via `CryptSignHash (PROV_RSA_AES)`; PSS = `not supported` | observado |
| `capi-base-rsa2048` | Base CSP v1.0 (`PROV_RSA_FULL`), se o cmdlet aceitar | idem ao A1 | observado |

"Observado" = registrado, sem reprovar o CI: é onde a ponte CAPI→CNG do Windows entra.
Também é obrigatório: `list` mostra cada certificado com o provedor certo e marcado como software.

```text
<!-- ORQUESTRADOR: colar a saída de windows/ci-windows.ps1 (tabela "Summary" e falhas) -->
```

Relatório do probe (`report-windows.md`, artefato do job):

```text
<!-- ORQUESTRADOR: colar o report-windows.md -->
```

### MSIX no runner (`msix/build-msix.ps1` + `msix/test-msix.ps1`)

O passo é `continue-on-error`: o resultado é evidência nos dois sentidos.

```text
<!-- ORQUESTRADOR: colar a tabela "Summary" do test-msix.ps1 -->
```

| Verificação | Resultado |
|---|---|
| `makeappx` aceita o manifesto (rescap + desktop6 + alias console) | _a preencher_ |
| `Add-AppxPackage` instala | _a preencher_ |
| alias existe e executa com stdout capturado | _a preencher_ |
| (a) chaves no HKCU real (Chrome, Edge, Firefox) | _a preencher_ |
| (b) manifestos no caminho real | _a preencher_ |
| (c) `path` = alias, alias executa | _a preencher_ |
| (d) Chromium troca mensagem com o host pelo alias | _a preencher_ |

## O que o CI não prova

O runner é Windows Server, sem leitor, sem tokens, sem usuário na frente da tela. Falta provar
com hardware e pessoas:

- assinatura com SafeSign, SafeNet, ePass2003 e Watchdata, por CNG e por CAPI, e se algum CSP de
  terceiros é `PROV_RSA_FULL` sem SHA-2;
- A1 importado pelo assistente em Windows 10 e 11 (e com "proteção forte" ligada);
- o diálogo de PIN na frente do terminal e na frente do navegador;
- quantos PINs uma sessão pede (um por sessão? um por assinatura?);
- MSIX em Windows 10 22H2 e 11 24H2 de usuário comum; Chrome, Edge e Firefox **de verdade**
  iniciando o host pelo alias;
- pré-registro da extensão (`HKCU\Software\Google\Chrome\Extensions\<id>` e equivalentes do Edge);
  depende de a extensão estar publicada;
- janela egui em RDP (o kit ainda não tem janela; ver passo 8 do roteiro).

## Roteiro para os tokens reais

Anote tudo numa cópia da tabela do fim. **Cuidado com o PIN:** errar o PIN de propósito no máximo
uma vez por token, e em seguida acertar, para não bloquear.

### 0. Preparação

1. Windows 10 22H2 **e** Windows 11 (24H2 ou mais novo), usuário comum. Anote `winver`.
2. Binário: baixe o `websign-probe.exe` do CI ou compile (`cargo build --release -p websign-probe`
   em `docs/prototypes/kit`). Abra um terminal (Windows Terminal ou `cmd`) na pasta dele.
3. Por token: modelo, middleware instalado e versão (Painel de Controle → Programas).
4. Com o token conectado:
   ```powershell
   .\websign-probe.exe devices
   .\websign-probe.exe report --out relatorio-<token>.md
   ```
   O relatório não tem dados pessoais; anexe-o aqui.

### 1. Listar sem PIN

```powershell
.\websign-probe.exe list
.\websign-probe.exe list --every-path
```

Anote: o certificado aparece? Qual `provider` (`[CNG]` ou `[CAPI type …]`)? `hardware`? **Apareceu
algum pedido de PIN?** (não pode). Com `--every-path`, o mesmo certificado aparece também pelo
PKCS#11 como caminho alternativo?

### 2. Assinar pelo caminho que o app usa

```powershell
.\websign-probe.exe sign --cert <8+ dígitos da impressão digital> --hash all --pss
```

Anote por caso: OK/FAIL, `via NCryptSignHash` ou `via CryptSignHash`, tempo. E sobre o PIN:
quantas vezes foi pedido; **o diálogo abriu na frente do terminal?** (se abrir atrás ou só piscar na
barra de tarefas, anote); texto/visual do diálogo (é do Windows ou do middleware?).

### 3. Cancelar e errar o PIN

Repita o passo 2 só com SHA-256, clicando **Cancelar** no diálogo → esperado `cancelled by the user`.
Repita digitando o PIN errado **uma vez** → esperado `wrong PIN` (anote o código se vier
`… failed with 0x…`). Depois assine com o PIN certo.

### 4. Os outros modos de abrir a chave

```powershell
.\websign-probe.exe sign --cert <fp> --hash all --pss --ncrypt prefer
.\websign-probe.exe sign --cert <fp> --hash all --pss --ncrypt only
```

Anote se o caminho mudou (CAPI → CNG), se PSS passou a funcionar, e se o diálogo mudou.

### 5. O mesmo token pelo PKCS#11

```powershell
.\websign-probe.exe sign --cert <fp> --hash all --pss --every-path --module <dll>
```

DLLs usuais (confirme o caminho na máquina): SafeNet `C:\Windows\System32\eTPKCS11.dll`;
SafeSign `C:\Windows\System32\aetpkss1.dll`; ePass2003 `C:\Windows\System32\eps2003csp11.dll`;
Watchdata: procure `*pkcs11*.dll`/`WD*.dll` em `System32`; OpenSC
`C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll`.
Anote se o PIN foi pedido pelo terminal (esperado, PKCS#11) e se tudo verificou.

### 6. A1 instalado

1. Dê dois cliques num `.pfx` de teste → assistente com as opções padrão → "Usuário atual".
2. Passos 1 e 2. Esperado: `Microsoft Enhanced Cryptographic Provider v1.0 [CAPI type 1, AT_KEYEXCHANGE]`
   (ou o KSP de software) e `via CryptSignHash (PROV_RSA_AES)`.
3. Remova (`certmgr.msc`), reimporte marcando **"Habilitar proteção forte da chave privada"** e
   repita o passo 2: o diálogo de consentimento/senha abriu na frente?

### 7. MSIX e navegadores

Em PowerShell 7 **como administrador**, na pasta `docs/prototypes/kit`:

```powershell
$env:PROBE_EXE = "<caminho>\websign-probe.exe"
./msix/build-msix.ps1
./msix/test-msix.ps1
```

(Se o kit veio num ZIP, desbloqueie os scripts antes: `Get-ChildItem -Recurse *.ps1 | Unblock-File`.)
Anote a tabela final. Depois, **sem** privilégio de administrador:

1. Carregue a extensão de teste (`docs/prototypes/kit/extension`) sem empacotar no Chrome e no Edge
   (modo desenvolvedor) e como complemento temporário no Firefox (`about:debugging`).
2. Pela página/ação de teste da extensão, peça versão, lista e uma assinatura. Anote em cada
   navegador: conectou? a mensagem voltou? **o diálogo de PIN abriu na frente do navegador?**
3. Em Configurações → Aplicativos → Aliases de execução de aplicativo, **desligue** o alias do
   WebeSign Probe e repita: o host ainda abre (o manifesto aponta para a cópia na pasta da família
   do pacote)?
4. Desinstale (`Get-AppxPackage WebeSign.Probe | Remove-AppxPackage`) e anote o que sobrou em
   `HKCU\Software\Google\Chrome\NativeMessagingHosts` e em `%LOCALAPPDATA%\websign`.

### 8. egui em RDP

O kit ainda não tem janela. Até o app ter a janela de confirmação, use um exemplo mínimo do eframe
com `wgpu` (TODO(gustavo): decidir se entra um `websign-probe window` no kit) e meça, numa sessão
RDP e numa VM sem GPU, o tempo até a janela aparecer e se o fallback de software (WARP) funciona.

### Tabela para preencher

| Token / caso | Windows | Middleware (versão) | `list` sem PIN | Provedor mostrado | `allow` | `prefer` | `only` | PKCS#11 | PINs por sessão | Diálogo na frente (terminal / navegador) |
|---|---|---|---|---|---|---|---|---|---|---|
| SafeSign | | | | | | | | | | |
| SafeNet | | | | | | | | | | |
| ePass2003 | | | | | | | | | | |
| Watchdata | | | | | | | | | | |
| A1 (assistente) | | — | | | | | | — | | |
| A1 (proteção forte) | | — | | | | | | — | | |

| MSIX | Windows 10 | Windows 11 |
|---|---|---|
| `test-msix.ps1` sem FAIL | | |
| Chrome inicia o host e troca mensagem | | |
| Edge inicia o host e troca mensagem | | |
| Firefox inicia o host e troca mensagem | | |
| Alias desligado: host ainda abre | | |
| Sobras após desinstalar | | |

## Riscos conhecidos

- **CSP de terceiros `PROV_RSA_FULL` sem SHA-2:** CAPI não assina; `PREFER` só atravessa para CNG
  os provedores da Microsoft. Saída: PKCS#11 do mesmo token (a deduplicação já prefere o SO e cai
  no PKCS#11 como alternativa).
- **`CRYPT_ACQUIRE_COMPARE_KEY_FLAG`** exige ler a chave pública do contêiner; um CSP antigo que não
  exporte a pública falharia ao abrir. Se aparecer, remover a flag para esse caso.
- **Handle em cache e cartão removido:** tratado descartando o handle após erro nativo; confirmar
  com os tokens que o erro é nativo (e não, por exemplo, um diálogo de "insira o cartão").
- **Foco do diálogo:** mesmo com dono, o Windows pode negar o primeiro plano a um processo que não
  recebeu entrada (o diálogo pisca na barra de tarefas). O host é filho do navegador, que tem o foco,
  mas o dono que ele consegue é o console oculto (o `--parent-window` vem `0` de uma extensão MV3);
  medir no passo 7. No app, o dono será a janela de Confirmação, que já está na frente.
- **Desinstalar o MSIX deixa as chaves do HKCU e os manifestos** (sem virtualização o Windows não os
  limpa). O host some, o navegador reporta "host não encontrado" e a extensão mostra "falta o app".
  O app recria tudo a cada início.
- **Colisão de alias:** outro pacote com `websign-probe.exe`/`websign.exe` toma o alias da raiz;
  por isso o manifesto usa a cópia da pasta da família quando existe.
- **`unvirtualizedResources` é capacidade restrita:** a Microsoft pode recusar na certificação.
  Precisa de justificativa no Partner Center (integração de native messaging, precedentes de apps
  que registram hosts). É o ponto que mais pode derrubar o MSIX e só se prova submetendo.
- **Runtime do Visual C++:** um binário Rust MSVC comum depende de `vcruntime140.dll`, que uma máquina
  limpa pode não ter. O kit já compila com `+crt-static` (`kit/.cargo/config.toml`) e não depende dela;
  o app deve manter isso (ou declarar `Microsoft.VCLibs.140.00.UWPDesktop` no MSIX).
- **Ambientes corporativos** que bloqueiam apps da loja/aliases (AppLocker, políticas de Store) não
  terão o MSIX — o MSI continua necessário para eles no longo prazo, mesmo com o MSIX aprovado.
- **Runner é Windows Server:** qualquer sucesso no CI precisa ser repetido em Windows 10/11 cliente.

## Decisão proposta

**MSIX aprovado** se, e somente se:

1. `test-msix.ps1` sem FAIL no CI **e** em Windows 10 22H2 e Windows 11;
2. Chrome, Edge e Firefox iniciam o host pelo alias e trocam mensagem (passo 7);
3. a submissão de teste no Partner Center aceita `runFullTrust` + `unvirtualizedResources` com a
   nossa justificativa.

Qualquer um desses falhando → **MSI assinado pela SignPath** (grava as mesmas chaves no HKCU, host
em `%LOCALAPPDATA%\Programs`), mantendo o mesmo binário.

Independente do empacotamento, o caminho de assinatura do app é: `--ncrypt allow` + reabertura AES
para A1 em `PROV_RSA_FULL`, **desde que** os quatro tokens assinem SHA-256/384/512 por esse caminho
com o diálogo de PIN na frente. Se algum token só funcionar com `prefer`, o app passa a usar
`prefer` para esse provedor; se nenhum modo funcionar, o token fica com o PKCS#11.
