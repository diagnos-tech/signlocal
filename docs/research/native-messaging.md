# Native messaging: onde cada navegador procura o host

Pesquisa para as provas da Fase 0 e para o `register` do app. Cobre caminhos e chaves de registro por
sistema × navegador, argumentos de lançamento, limites, o caso do Firefox Snap e do Flatpak, o Safari (que
não usa manifesto) e o pré-registro da extensão.

**Como ler a coluna "Base":**

| Sigla | Significa |
|---|---|
| **D** | documentação oficial do fornecedor (links no fim) |
| **C** | código-fonte do Chromium, lido em `raw.githubusercontent.com/chromium/chromium` |
| **B** | Bitwarden, `native-messaging.main.ts` (só para comparar caminhos) |
| **K** | KeePassXC, `NativeMessageInstaller.cpp` (idem) |
| **W** | web-eid-app (`CMakeLists.txt`, `web-eid.wxs`, modelos de manifesto) |
| **?** | não confirmei em fonte primária: tratar como hipótese até a prova |

Nada aqui foi copiado: as três bases de código foram lidas só para conferir caminhos.

---

## 1. Resumo do que decide o projeto

1. **O manifesto é por navegador, e o formato difere por família.** Chromium: `allowed_origins`
   (`chrome-extension://<id>/`, sem curinga). Firefox: `allowed_extensions` (ID Gecko). São dois arquivos.
2. **Windows não tem pasta: tem chave de registro.** `HKCU` basta (o Chrome consulta `HKCU` antes de `HKLM`) e
   o manifesto pode estar em qualquer pasta. **Edge, Brave, Vivaldi e Opera caem na chave do Chrome** quando
   não têm a própria, então uma chave do Chrome já cobre vários (§3.1).
3. **macOS e Linux usam `<pasta de dados do navegador>/NativeMessagingHosts/<host>.json`.** É por isso que o
   `--user-data-dir` do Chromium permite testar sem tocar no perfil real (§3.2, §3.3).
4. **No MV3 o host nunca recebe `--parent-window`**: o Chrome passa `0` quando quem conecta é um service
   worker (§2). Diálogos de PIN do sistema não podem depender desse handle: a janela do app precisa achar a
   janela do navegador por conta própria ou se pôr em primeiro plano.
5. **Firefox Snap não lê o manifesto: pede ao portal** (`org.freedesktop.portal.WebExtensions`, hoje;
   `org.freedesktop.NativeMessagingProxy`, em seguida). O host roda **fora** do Snap, iniciado pelo portal,
   com o ambiente do portal, não o do Firefox (§3.4).
6. **Safari não tem manifesto, registro nem `allowed_origins`.** A extensão vive dentro do app e fala com uma
   app extension (appex) por `runtime.sendNativeMessage` (§3.5).
7. **Pré-registro de extensão:** Chromium tem canal por arquivo/registro (Linux sem confirmação; Windows e
   macOS pedem que o usuário habilite). Firefox só por política do sistema (§4).

---

## 2. Como o navegador lança o host

### Argumentos

| Navegador | Argumentos, na ordem | Base |
|---|---|---|
| Chrome, Edge, Brave, Vivaldi, Opera (Linux, macOS) | `chrome-extension://<id>/` | D, C |
| Chrome, Edge, Brave, Vivaldi, Opera (Windows) | `chrome-extension://<id>/` `--parent-window=<HWND decimal>` | D |
| Firefox (todos os SOs) | caminho completo do manifesto, ID Gecko da extensão (desde o Firefox 55) | D |
| Safari | nenhum: não há processo filho, ver §3.5 | D |

- O primeiro argumento do Chromium é a **origem que o próprio navegador validou** contra `allowed_origins`.
  É a única identidade da extensão em que o host pode confiar; o `origin` que a extensão de teste manda no
  JSON é informativo.
- **`--parent-window` vale `0` quando o contexto chamador é um service worker** (Chrome e Edge, D). Toda
  extensão MV3 é service worker, então o handle não chega. O `probe` trata `0` como ausente
  (`nm/launch.rs`).
- O diretório de trabalho do host é a pasta do executável (C, `LaunchContext`).
- O ambiente é o do navegador (confirmado no e2e: `WEBSIGN_PROBE_PIN` e `SOFTHSM2_CONF` exportados antes do
  Chromium chegam ao host). Exceção: hosts iniciados pelo portal do Firefox Snap (§3.4).

### Protocolo (igual nos dois sentidos)

`u32` com o tamanho em **ordem de bytes nativa** + JSON UTF-8. Nas plataformas que nos interessam (x86-64,
ARM64) é little-endian, mas o código usa `from_ne_bytes`/`to_ne_bytes` como a documentação manda.

| Direção | Limite | Base |
|---|---|---|
| host → navegador | **1 MB** (o navegador derruba a conexão se passar) | D (Chrome, Edge, Firefox) |
| navegador → host | 64 MiB (Chrome, doc atual); 4 GB (Edge, Firefox) | D |

O `probe` recusa reply acima de 1 MiB e aceita no máximo 1 MiB de entrada (as mensagens são pequenas).

### Ciclo de vida

- `runtime.connectNative()` mantém o processo até a porta fechar; `sendNativeMessage()` inicia **um processo
  por mensagem** e só a primeira resposta vale (D).
- **Chrome 105+: uma porta `connectNative` mantém o service worker vivo.** Se o host morre, a porta fecha e o
  worker termina depois dos timers; a doc recomenda chamar `connectNative()` de novo no `onDisconnect`
  (D, ciclo de vida do service worker). A extensão de teste fecha a porta por ociosidade (60 s) e reconecta
  no próximo pedido.
- O `stderr` do host: o Firefox o redireciona ao Browser Console (D). O Chrome não o mostra em interface
  alguma (nos sistemas POSIX o host herda o `stderr` do processo do navegador, visível só se ele foi aberto
  num terminal; C, `launch_context_posix.cc`). Por isso o `probe` escreve `<temp>/websign-probe-host.log`.

### Mensagens de erro típicas (para o diagnóstico)

| Navegador | Texto | Causa provável |
|---|---|---|
| Chrome/Edge | `Specified native messaging host not found.` | manifesto ou chave ausente; nome diferente |
| Chrome/Edge | `Access to the specified native messaging host is forbidden.` | origem fora de `allowed_origins` |
| Chrome/Edge | `Native host has exited.` | o processo terminou (visto quando o host caía por `todo!()`) |
| Firefox | `No such native application <nome>` | manifesto/chave não encontrados |
| Firefox | `This extension does not have permission to use native application <nome>` | ID fora de `allowed_extensions` |
| Firefox | `File at path <path> does not exist, or is not executable` | manifesto achado, `path` errado |

---

## 3. Matriz de registro

`<host>` = `dev.websign.host` (`project.toml`, `[ids].native_host`; o Chrome só aceita `[a-z0-9_.]`).

### 3.1 Windows

O manifesto pode ficar **em qualquer pasta**; o valor padrão da chave é o caminho dele. O `path` do manifesto
pode ser relativo à pasta do manifesto (D).

| Navegador | Chave `HKCU` (valor padrão = caminho do manifesto) | Base |
|---|---|---|
| Chrome (e Beta/Dev/Canary) | `Software\Google\Chrome\NativeMessagingHosts\<host>` | D, C |
| Chromium | `Software\Chromium\NativeMessagingHosts\<host>` (lida primeiro; depois a do Chrome) | C |
| Edge | `Software\Microsoft\Edge\NativeMessagingHosts\<host>`; fallback: Chromium, depois Chrome | D |
| Brave | `Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\<host>`; KeePassXC usa a do Chrome | B, K |
| Vivaldi | `Software\Vivaldi\NativeMessagingHosts\<host>`; KeePassXC usa a do Chrome | B, K |
| Opera | a do Chrome (fórum da Opera; sem chave própria conhecida) | ? |
| Firefox | `Software\Mozilla\NativeMessagingHosts\<host>` | D |

Detalhes do Chromium (C, `launch_context_win.cc`):

- Ordem de busca: `HKCU` (se a política `NativeMessagingUserLevelHosts` não proíbe) e depois `HKLM`; em cada
  raiz, visão de 32 bits antes da de 64. A **primeira chave achada vale**: o Edge documenta que, se a extensão
  está na Edge Add-ons e na Chrome Web Store, os **dois IDs** precisam estar no mesmo `allowed_origins`.
- Só builds com `CHROMIUM_BRANDING` leem `Software\Chromium` antes; todos os outros leem só
  `Software\Google\Chrome`. Por isso o `probe` grava as chaves próprias **e** a do Chrome, e Opera/Chrome
  compartilham uma.
- O host é iniciado por `cmd.exe /d /s /c "<comando>" < pipe > pipe`, a menos que a política
  `NativeHostsExecutablesLaunchDirectly` (ou o feature `LaunchWindowsNativeHostsDirectly`) esteja ligada.
  Nos dois modos `start_hidden = true`, **exceto** quando o executável é do subsistema GUI e o lançamento é
  direto. **Risco a provar:** um host de console iniciado com `SW_HIDE` pode ter a primeira janela criada
  oculta. A janela de confirmação deve chamar `ShowWindow(SW_SHOW)` explicitamente ou o binário deve ser do
  subsistema Windows.
- **MSIX:** a doc da Microsoft diz que, no Windows 10 1903+, arquivos **novos** criados em
  `AppData\Local`, `Local\Microsoft`, `Roaming` e `Roaming\Microsoft` vão para um local privado do pacote, e
  toda escrita em `HKCU` é copy-on-write privada. Sem `desktop6:FileSystemWriteVirtualization` e
  `desktop6:RegistryWriteVirtualization` desligados (com `rescap:unvirtualizedResources`), nem o manifesto nem
  a chave chegam ao navegador. O `register` tem `--manifest-dir` para apontar outra pasta. Ver
  `docs/prototypes/1-windows.md`.
- `register` no MSIX grava o **alias de execução** como `path` (`platform::windows::msix_alias_path`), porque
  arquivos em `WindowsApps` não são executáveis por outros processos.

### 3.2 macOS

Usuário: `~/Library/Application Support/<pasta>/NativeMessagingHosts/<host>.json`.

| Navegador | `<pasta>` | Base |
|---|---|---|
| Chrome | `Google/Chrome` (Beta: `Google/Chrome Beta`; Dev: `Google/Chrome Dev`; Canary: `Google/Chrome Canary`) | D, B |
| Chrome for Testing (146+) | `Google/ChromeForTesting` (antes do 146 usava a pasta do Chrome) | D |
| Chromium | `Chromium` | D, K |
| Edge | `Microsoft Edge` (+ ` Beta`, ` Dev`, ` Canary`) | D, B |
| Brave | `BraveSoftware/Brave-Browser` (+ `-Beta`, `-Nightly`) | K; Beta/Nightly por analogia (?) |
| Vivaldi | `Vivaldi` | B, K |
| Opera | `com.operasoftware.Opera` | ? |
| Firefox | `Mozilla/NativeMessagingHosts` (nota: `Mozilla`, sem subpasta de perfil) | D |

Sistema (todos os usuários): `/Library/Google/Chrome/NativeMessagingHosts/`,
`/Library/Microsoft/Edge/NativeMessagingHosts/`, `/Library/Application Support/Chromium/NativeMessagingHosts/`,
`/Library/Application Support/Mozilla/NativeMessagingHosts/` (D). O web-eid instala o de Chrome e o de Firefox
nesses caminhos (W).

**Sandbox (Mac App Store).** Só as pastas `NativeMessagingHosts/` precisam de escrita, via
`com.apple.security.temporary-exception.files.home-relative-path.read-write` (o Bitwarden lista exatamente
essas pastas na build MAS; B, `entitlements.mas.plist`). O caminho é relativo à **home real**, mas um processo
sandboxed vê `$HOME` dentro do contêiner (`<home>/Library/Containers/<bundle>/Data`); o `register` reconstrói
a home real cortando em `/Library/Containers/` (o Bitwarden usa `os.userInfo().homedir`, que consulta o banco
de usuários). O binário chamado pelo navegador roda com `com.apple.security.inherit` no Bitwarden (B,
`entitlements.desktop_proxy*.plist`).

### 3.3 Linux

Usuário: `<pasta de configuração>/NativeMessagingHosts/<host>.json` (Firefox: `~/.mozilla/native-messaging-hosts/`).
No Chromium isso é `DIR_USER_DATA/NativeMessagingHosts`, ou seja, **vale para qualquer `--user-data-dir`**
(C, `chrome_paths.cc`: `DIR_USER_NATIVE_MESSAGING`; só compilado para Linux, ChromeOS, macOS e Android, não
para Windows).

| Navegador | Pasta sob `~/.config/` (usuário) | Sistema | Base |
|---|---|---|---|
| Chrome | `google-chrome`, `google-chrome-beta`, `google-chrome-unstable` | `/etc/opt/chrome/native-messaging-hosts/` | D, B |
| Chrome for Testing (146+) | `google-chrome-for-testing` | `/etc/opt/chrome_for_testing/native-messaging-hosts/` | D |
| Chromium | `chromium` | `/etc/chromium/native-messaging-hosts/` | D, C |
| Edge | `microsoft-edge`, `-beta`, `-dev` | `/etc/opt/edge/native-messaging-hosts/` | D |
| Brave | `BraveSoftware/Brave-Browser` (+ `-Beta`, `-Nightly`) | `/etc/chromium/native-messaging-hosts/`? | B, K; sistema ? |
| Vivaldi | `vivaldi`, `vivaldi-snapshot` | `/etc/chromium/native-messaging-hosts/` (fórum) | B, K |
| Opera | `opera`, `opera-beta`, `opera-developer` | ? | ? (fórum de 2014) |
| Firefox | `~/.mozilla/native-messaging-hosts/` | `/usr/lib/mozilla/native-messaging-hosts/` e `/usr/lib64/...` | D |

O web-eid (pacote de sistema) instala em `/usr/lib/mozilla/native-messaging-hosts/` (Debian; `${LIBDIR}` nas
demais), `/etc/chromium/native-messaging-hosts/` e `/etc/opt/chrome/native-messaging-hosts/`, com `path`
absoluto `/usr/bin/web-eid` (W). O `.deb` do nosso app fará o mesmo; o `register` por usuário é a rota do
"complemento" e dos testes.

**Só registrar onde o navegador existe.** O `register` só grava quando a pasta de configuração do navegador
já existe (mesma regra do Bitwarden, que avisa "not found, skipping"). Criar `~/.config/vivaldi` para quem não
tem Vivaldi suja a home e engana ferramentas que tratam a pasta como prova de instalação. O preço: um
navegador instalado e nunca aberto é ignorado até o próximo registro; o app final registra a cada início e
fecha a lacuna. No Windows não há pasta a testar, então as chaves de `HKCU` são gravadas sempre.

**Snap (Chromium).** O Chromium do Snap lê `~/snap/chromium/common/chromium/NativeMessagingHosts/` (o
`register` grava aqui) e roda com `/tmp` privado (o log do host fica no `/tmp` do Snap). Se o confinamento
deixa o host executar, não está provado: **prova 4**.

**Flatpak.** Cada navegador Flatpak enxerga só o próprio `~/.var/app/<id>/`: manifesto em
`~/.var/app/<id>/config/<pasta>/NativeMessagingHosts/` (Firefox: `.../.mozilla/native-messaging-hosts/`) e o
binário **copiado para lá**, já que o `/usr` do sandbox é o do runtime (mesma técnica do Bitwarden, que faz
hard link). IDs: `com.google.Chrome`, `org.chromium.Chromium`, `com.microsoft.Edge`, `com.brave.Browser`,
`com.vivaldi.Vivaldi`, `com.opera.Opera`, `org.mozilla.firefox`. **Limite conhecido:** dentro do sandbox o host
não vê o `pcscd` nem os módulos PKCS#11 do sistema, então ele responde `ping`, mas não assina. Resolver exige
o portal (§3.4) ou `flatpak-spawn --host`; fora do escopo do spike.

### 3.4 Firefox Snap e Flatpak: o portal

O Firefox confinado (Snap do Ubuntu; Flatpak) **não consegue** ler manifestos nem executar hosts. Em vez
disso, delega a um serviço D-Bus que fora do sandbox acha o manifesto, valida o ID da extensão contra
`allowed_extensions`, **pergunta ao usuário uma vez por par extensão × aplicação** e inicia o processo,
devolvendo descritores de `stdin`/`stdout`/`stderr`.

| | Portal WebExtensions | Native Messaging Proxy |
|---|---|---|
| Nome D-Bus | `org.freedesktop.portal.WebExtensions` | `org.freedesktop.NativeMessagingProxy` |
| Métodos | `CreateSession`, `GetManifest`, `Start` (+ sinal `Response`), `GetPipes`, `Close` | `GetManifest`, `Start`, `Close` |
| Preferência do Firefox | `widget.use-xdg-desktop-portal.native-messaging` | `widget.use-xdg-desktop-portal.native-messaging-proxy` |
| Valores | 0 desligado, 1 ligado, 2 autodetecção | idem |
| Situação (setembro de 2026) | patch da distribuição no Ubuntu desde o 22.04 (não entrou no `xdg-desktop-portal` upstream); é o que o Firefox Snap **estável** usa hoje | substituto, previsto para o Ubuntu 26.04. Bugzilla 1955255: `RESOLVED FIXED` no Firefox 157 (entrou no mozilla-central em 10/09/2026), preferência padrão 0; em 29/09/2026 a Canonical acabara de habilitá-la só no Snap `nightly` |
| Usuário é perguntado | sim | não |

Fontes: doc de design do Firefox e Bugzilla 1955255 (links no fim). Consequências para o app:

1. **O manifesto tem de estar no disco do host**, em `~/.mozilla/native-messaging-hosts/` ou
   `/usr/lib/mozilla/native-messaging-hosts/`, e o `path` tem de ser um caminho do host (nunca dentro de
   `~/snap/`). O `register` já grava esse arquivo; nada específico de Snap é necessário.
2. **O web-eid não tem tratamento específico de Snap** nos instaladores que li (`CMakeLists.txt`, `web-eid.wxs`):
   instala o manifesto de sistema com `path` absoluto e deixa o portal fazer o resto. É a mesma aposta que
   fazemos.
3. **Quem inicia o host é o portal, não o Firefox**: o ambiente (variáveis, `DISPLAY`/`WAYLAND_DISPLAY`,
   `XDG_*`) é o do serviço `xdg-desktop-portal` da sessão. `WEBSIGN_PROBE_PIN` não chega por essa rota, e uma
   janela egui depende de o portal ter o ambiente gráfico. Ambos são pontos da **prova 4**.
4. Na primeira conexão o usuário vê um diálogo do portal ("permitir que <extensão> inicie <aplicação>"):
   entra no texto de ajuda do popup.
5. Módulos PKCS#11 do Firefox Snap **não** entram aqui: o portal só cobre native messaging.

### 3.5 Safari

Não há manifesto, chave de registro, `allowed_origins` nem processo filho. A Apple (D):

- A extensão web tem três partes que rodam isoladas: o **app** (macOS/iOS), o **JavaScript** da extensão e uma
  **app extension (appex)** que faz a mediação. Os sandboxes são separados; o que os une são *app groups*.
- O background script (ou uma página da extensão) chama `browser.runtime.sendNativeMessage(app, mensagem)`;
  **o primeiro parâmetro é ignorado** e a mensagem vai sempre para a appex do app que contém a extensão,
  em `beginRequest(with:)` (`NSExtensionRequestHandling`). **Content scripts não podem** falar com a appex,
  então a ponte do site passa pelo background, como no Chrome.
- O caminho contrário (app → JS) usa `SFSafariApplication.dispatchMessage` e uma `runtime.connectNative`
  cuja porta só conecta ao app que contém a extensão.
- O web-eid usa a appex só como ponte: ela **abre o app** (`NSWorkspace.launchApplication`), guarda a
  mensagem em um `UserDefaults` compartilhado por app group e avisa por `NSDistributedNotificationCenter`; a
  resposta volta pelo mesmo caminho (W, `src/mac/`). É o desenho da prova 2 (a appex assina direto ou abre o
  app).
- Estado da extensão: `SFSafariExtensionManager.getStateOfSafariExtension(withIdentifier:)`, do app.

---

## 4. Pré-registro de extensão

"Pré-registrar" faz o navegador oferecer/instalar a extensão sem o usuário ir à loja. Vale só para a
extensão de produção; a de teste é carregada com `--load-extension`.

| Navegador | SO | Onde | Como o usuário vê | Base |
|---|---|---|---|---|
| Chrome | Windows | `HKLM\Software\Wow6432Node\Google\Chrome\Extensions\<id>` (32 bits: sem `Wow6432Node`), valor `update_url` = `https://clients2.google.com/service/update2/crx`. O código lê **também** `HKCU\Software\Google\Chrome\Extensions` (usa `HKCU` só quando a chave em `HKLM` não abre) | precisa habilitar num diálogo | D, C |
| Chrome | macOS | `~/Library/Application Support/Google/Chrome/External Extensions/<id>.json` (usuário) ou `/Library/Application Support/Google/Chrome/External Extensions/` (todos: só lido se os donos e permissões da árvore forem root/admin) | diálogo | D, W |
| Chrome | Linux | `/opt/google/chrome/extensions/<id>.json` ou `/usr/share/google-chrome/extensions/<id>.json` | **instala sozinho** | D |
| Chromium | Linux | `/usr/share/chromium/extensions/<id>.json` | idem | W |
| Edge | Windows | `HKLM\Software\Microsoft\Edge\Extensions\<id>` (`update_url` = `https://edge.microsoft.com/extensionwebstorebase/v1/crx`) | diálogo | D, W |
| Edge | macOS | `~/Library/Application Support/Microsoft Edge/External Extensions/` ou `/Library/Application Support/Microsoft/Edge/External Extensions/` | diálogo | D |
| Edge | Linux | `~/.config/microsoft-edge/External Extensions/` ou `/usr/share/microsoft-edge/extensions/` | automático | D |
| Firefox | todos | **não há arquivo por usuário.** Política `ExtensionSettings` (`installation_mode: force_installed` + `install_url`) em `policies.json` (`distribution/`, `/etc/firefox/policies/`) ou no registro `HKLM\Software\Policies\Mozilla\Firefox`. A doc só cita `HKLM` | instala sem perguntar | D |
| Firefox | Linux (pacote) | `/usr/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/<id>.xpi` (o web-eid empacota assim; o GUID é o ID do aplicativo Firefox) | o Firefox oferece a instalação | W |
| Safari | macOS | a extensão vem **dentro do app**; o usuário liga em Safari > Configurações > Extensões | ligar manualmente | D |

Arquivo externo do Chromium: `{"external_update_url": "https://clients2.google.com/service/update2/crx"}`.
No Windows e no macOS o Chrome só aceita instalação externa vinda da Chrome Web Store (desde o Chrome 33 e
44, respectivamente).

**Consequência para o MSIX (HKCU-only):** o pré-registro do Chrome por `HKCU` é plausível pelo código, mas a
documentação só fala em `HKLM`; o do Firefox exige política de sistema, impossível sem elevação. A **prova 1**
tem de dizer se `HKCU\Software\Google\Chrome\Extensions\<id>` funciona de fato e, se não, o popup da extensão
"Baixar" e a página da loja são o único caminho para Firefox.

Política que pode bloquear tudo isso e que o diagnóstico deve saber explicar (Chrome/Edge, D):
`NativeMessagingAllowlist`, `NativeMessagingBlocklist` e `NativeMessagingUserLevelHosts` (quando `false`, o
Chrome ignora `HKCU` e as pastas de usuário, e só lê o local do sistema).

---

## 5. O que o kit implementa

- `websign-probe register [--browser <all|chrome|chromium|edge|brave|vivaldi|opera|firefox>]... [--uninstall]
  [--user-data-dir DIR] [--dry-run] [--extension-id ID]... [--manifest-dir DIR]` grava os manifestos das
  tabelas acima (`probe/src/nm/register/`). `allowed_origins` = ID de desenvolvimento + IDs de loja não
  vazios + `--extension-id`. Firefox: `allowed_extensions` = `firefox_id`.
- `--user-data-dir DIR` grava só `DIR/NativeMessagingHosts/<host>.json`, que o Chromium lê com
  `--user-data-dir=DIR` no Linux e no macOS; é o que o `nm-e2e` usa.
- O host (`probe/src/nm/`) reconhece o lançamento pelos argumentos da §2, fala o protocolo `"v": 1` e registra
  em `<temp>/websign-probe-host.log` só: horário, pid, família, ID da extensão, tipo e tamanho das mensagens,
  resultado. Nunca PIN, digest, assinatura, nome, certificado, impressão digital ou site.
- Variáveis só de teste: `WEBSIGN_PROBE_PIN` (PIN PKCS#11) e `WEBSIGN_PROBE_MODULES` (módulos extras, lista
  no formato do `PATH`), porque o navegador não passa argumentos.

### O que continua sem prova

| Item | Por quê | Prova |
|---|---|---|
| Firefox (todos os SOs) e Safari | não há Firefox nem Mac neste ambiente; o `web-ext lint` aprovou o manifesto (só o aviso esperado de `service_worker`) | 2, 4 |
| Edge, Brave, Vivaldi, Opera no Windows/Linux/macOS | os caminhos vêm de documentação e de terceiros (B, K); só o Chromium foi executado | 1, 2 |
| Opera (todos), Brave Beta/Nightly | sem fonte primária | 1, 2 |
| Chromium Snap, Flatpak, Firefox Snap | sem esses ambientes aqui | 4 |
| `--parent-window` real no Windows e janela oculta por `SW_HIDE` | só existe no Windows | 1 |
| Pré-registro por `HKCU` (Chrome/Edge) | a doc só cita `HKLM` | 1 |

---

## 6. Fontes

**Documentação**

- Chrome, Native messaging: <https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging>
- Chrome, ciclo de vida do service worker (`connectNative` mantém vivo): <https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle>
- Chrome, instalar extensões por arquivo/registro: <https://developer.chrome.com/docs/extensions/how-to/distribute/install-extensions>
- Edge, Native messaging: <https://learn.microsoft.com/en-us/microsoft-edge/extensions/developer-guide/native-messaging> (fonte: <https://raw.githubusercontent.com/MicrosoftDocs/edge-developer/main/microsoft-edge/extensions/developer-guide/native-messaging.md>)
- Edge, distribuição alternativa: <https://raw.githubusercontent.com/MicrosoftDocs/edge-developer/main/microsoft-edge/extensions/developer-guide/alternate-distribution-options.md>
- MDN, Native messaging: <https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging>
- MDN, Native manifests: <https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests>
- Firefox, native messaging em navegador confinado: <https://firefox-source-docs.mozilla.org/toolkit/components/extensions/webextensions/native-messaging-portal-design.html>
- Bugzilla 1955255 (native messaging proxy, Snap/Flatpak; estado lido pela API REST em 29/09/2026): <https://bugzilla.mozilla.org/show_bug.cgi?id=1955255>
- Ubuntu, chamada de testes do native messaging no Snap do Firefox: <https://discourse.ubuntu.com/t/call-for-testing-native-messaging-support-in-the-firefox-snap/29759>
- Firefox, política `ExtensionSettings`: <https://mozilla.github.io/policy-templates/>
- Apple, mensagens entre o app e o JavaScript de uma extensão do Safari: <https://developer.apple.com/documentation/safariservices/messaging-between-the-app-and-javascript-in-a-safari-web-extension>
- Microsoft, como apps desktop empacotados (MSIX) rodam, com virtualização de arquivos e registro: <https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes>

**Código lido (não copiado)**

- Chromium, busca de manifestos no Windows e lançamento: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/extensions/api/messaging/launch_context_win.cc>
- Chromium, busca no POSIX: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/extensions/api/messaging/launch_context_posix.cc>
- Chromium, `DIR_USER_NATIVE_MESSAGING`: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/common/chrome_paths.cc>
- Chromium, pré-registro por registro: <https://raw.githubusercontent.com/chromium/chromium/main/chrome/browser/extensions/external_registry_loader_win.cc>
- Bitwarden, caminhos por SO: <https://raw.githubusercontent.com/bitwarden/clients/main/apps/desktop/src/main/native-messaging.main.ts>
- web-eid-app, instalação: <https://raw.githubusercontent.com/web-eid/web-eid-app/main/src/app/CMakeLists.txt> e <https://raw.githubusercontent.com/web-eid/web-eid-app/main/install/web-eid.wxs>
- web-eid-app, ponte do Safari: `src/mac/` (mesmo repositório)
- KeePassXC, instalador de manifestos: `src/browser/NativeMessageInstaller.cpp`
