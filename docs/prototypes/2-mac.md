# Prova 2 — Mac: sandbox da Mac App Store, native messaging, CryptoTokenKit e Safari

**Status:** provado em CI com chaves de software; falta Mac real com Chrome, Safari e token · **Resultado parcial:** SIM para sandbox + manifestos + host + Keychain; Safari só desenhado ·
**Decisão proposta:** ver [§8](#8-decisão-proposta)

## Objetivo

Responder com evidência, antes de construir o app:

1. O app **dentro da sandbox** da Mac App Store grava os manifestos de native messaging nas pastas
   dos navegadores (Chrome, Edge, Brave, Firefox…)?
2. O Chrome inicia esse binário sandboxed como host, e ele assina via Keychain/CryptoTokenKit?
3. A extensão do Safari assina (direto na appex ou abrindo o app)?
4. `SFSafariExtensionManager` informa o estado da extensão?

Se a sandbox barrar o PKCS#11, a resposta vem da [prova 3](3-tokens-mac.md) (complemento `.dmg`).

## Resposta curta (até agora)

| # | Pergunta | Resposta | Evidência | Falta |
|---|---|---|---|---|
| 1 | Sandbox grava os manifestos? | **SIM (CI)**, com `temporary-exception.files.home-relative-path.read-write` só nas pastas `NativeMessagingHosts` e o home real via `getpwuid_r` | §4: Chrome, Edge, Brave e Firefox gravados de dentro da sandbox | App Review aceitar as exceções (precedente: Bitwarden) |
| 2a | Navegador inicia o host sandboxed? | **SIM (simulado no CI)**: iniciado com a origem e o stdio do Chrome, o host sandboxed respondeu `ping` e `list` | §4; precedente `desktop_proxy` do Bitwarden | Chrome real (roteiro §7) |
| 2b | Assina via Keychain/CTK dentro da sandbox? | Keychain: **SIM (CI)**, PKCS#1 v1.5, PSS e ECDSA. Token CTK: **provável SIM**, a consulta roda na sandbox sem entitlement extra | §4 | Token real (roteiro §7, prova 3) |
| 3 | Safari assina? | **Arquitetura proposta** (§5): appex só repassa; o app mostra a confirmação e assina | web-eid-app (`src/mac/`), Bitwarden (socket no app group) | Tudo: exige projeto Xcode e Mac real |
| 4 | `SFSafariExtensionManager`? | API disponível ao app que contém a extensão; em Rust via `objc2-safari-services` (0.3.2) | web-eid-app `main.mm` usa `getStateOfSafariExtension` e `showPreferencesForExtension` | Prova no Mac real |

## 1. Como o kit prova

### Keystore do macOS (`probe/src/keystores/macos/`)

| Arquivo | Papel |
|---|---|
| `mod.rs` | Duas origens: `macos:keychain` (chaves em arquivos de keychain: A1 importado) e `macos:ctk` (chaves em tokens CryptoTokenKit: A3). Cada uma relata as próprias falhas. `locator` = SHA-256 do certificado |
| `identities.rs` | `SecItemCopyMatching(kSecClassIdentity, kSecMatchLimitAll, kSecReturnRef)`; a consulta de tokens acrescenta `kSecAttrAccessGroup = kSecAttrAccessGroupToken`. Item ilegível não derruba a lista |
| `token.rs` | `kSecAttrTokenID` da chave privada (`SecKeyCopyAttributes`). `provider` = só a parte do driver (`com.apple.pivtoken`, `…OpenSCToken`), sem a instância, que costuma ser o número de série do cartão |
| `sign.rs` | Confere o tamanho do digest → `SecKeyIsAlgorithmSupported` → `SecKeyCreateSignature` → ECDSA DER → `r‖s` (`probe_core::ecdsa::der_to_raw`, curva lida do certificado) |
| `algorithm.rs` | Só os algoritmos "Digest": `RSASignatureDigestPKCS1v15SHA*`, `RSASignatureDigestPSSSHA*`, `ECDSASignatureDigestX962SHA*` |
| `errors.rs` | `errSecUserCanceled` e `TKErrorCodeCanceledByUser` → `Cancelled`; `errSecAuthFailed` e `TKErrorCodeAuthenticationFailed` → `WrongPin`; resto → `Native { api, code, message }` com `SecCopyErrorMessageString` (ou domínio + descrição do `CFError`) |

Decisões e porquês:

- **Duas consultas, não uma.** O Chromium atual faz uma só consulta de identidades
  (`net/ssl/client_cert_store_mac.cc`, com `kSecAttrCanSign`). Se ela inclui os tokens depende do
  roteamento interno do Security.framework (keychain de arquivo × keychain "iOS"); a documentação da
  Apple manda consultar tokens pelo grupo de acesso: *"Use this access group identifier as the value
  for the `kSecAttrAccessGroup` attribute in a keychain query to access external tokens such as smart
  cards. Access to this group is granted by default and does not require an explicit entry in your
  app's `keychain-access-groups`."* Identidades de token que aparecerem na consulta simples são
  descartadas ali e contadas só em `macos:ctk` — nada aparece duas vezes.
- **`list` não pede PIN.** Só referências e atributos; o diálogo de PIN (do driver do token) ou de
  senha/permissão (do keychain) aparece dentro de `SecKeyCreateSignature`. Chaves de keychain de
  arquivo são `hardware: Some(false)`; de token, `Some(true)`.
- **Erros do token chegam no domínio `CryptoTokenKit`**, não como `OSStatus`: o `SecCTKKey.m` da Apple
  repassa o `NSError` do TK sem converter. Por isso o mapeamento olha o domínio.
  `TODO(gustavo)`: PIN bloqueado chega como `AuthenticationFailed` com zero tentativas no
  `userInfo`; mapear para `PinLocked` quando visto num token real.
- **Chamadas não serializadas.** O Chromium põe toda chamada ao Security.framework atrás de um lock
  (o código legado de keychain não é thread-safe); o app deve fazer o mesmo se usar várias threads.

### RSASSA-PSS: o salt da Apple é o tamanho do digest

Evidência, em três camadas:

1. `SecKey.h` (open source da Apple, `keychain/headers/SecKey.h`):
   *"`kSecKeyAlgorithmRSASignatureDigestPSSSHA256` — RSA signature with RSASSA-PSS padding according
   to PKCS#1 v2.1, input data must be SHA-256 generated digest. PSS padding is calculated using MGF1
   with SHA256 and saltLength parameter is set to 32 (SHA-256 output size)."* Idem 48 e 64.
2. Implementação (`OSX/sec/Security/SecKeyAdaptors.m`): os IDs são
   `algid:sign:RSA:digest-PSS:SHA256:SHA256:32` (hash, hash do MGF1, salt) e o encoder chama
   `ccrsa_emsa_pss_encode(di, di, di->output_size, salt, …)` — MGF1 com o mesmo hash, salt =
   `output_size` do digest.
3. Execução: o teste `algorithm.rs` confere, no Mac do CI, o ID de cada algoritmo em runtime; e o
   `ci-macos.sh` assina PSS e o `probe_core::verify` confere com salt = tamanho do digest.

### Scripts (`kit/macos/`)

| Arquivo | O que faz |
|---|---|
| `ci-macos.sh` | Gera com OpenSSL identidades RSA-2048, P-256 e P-384 (`.p12` com algoritmos legados, que o `security import` aceita), cria um keychain temporário em `~/Library/Keychains`, importa com `-A` e `set-key-partition-list` (Apple, `unsigned:` e o `cdhash` do probe, para nenhum diálogo travar o runner), põe na lista de busca, roda `list` e `sign --hash all --pss` e confere cada caso; gera o relatório; restaura a lista de busca e apaga o keychain |
| `sandbox/entitlements.mas.plist` | Entitlements do app da loja (tabela abaixo) |
| `sandbox/Info.plist` | Bundle mínimo; o `CFBundleIdentifier` (= `project.toml`) define o contêiner |
| `sandbox/sandbox-test.sh` | O experimento (§3). Imprime `RESULT <id> SIM/NÃO/N/A` e uma tabela no resumo do job |
| `sandbox/run-sandboxed.sh` | Roda **qualquer** comando do probe dentro da sandbox e mostra as negações do log; para o Mac real |
| `lib/common.sh`, `lib/bundle.sh` | Funções comuns (bash 3.2 do macOS: roda em qualquer Mac) |

`sign --all` não é usado no CI de propósito: o keychain System de todo Mac tem identidades
(`com.apple.systemdefault`, `com.apple.kerberos.kdc`) cujas chaves só serviços do sistema usam;
assinar com elas abre um diálogo de senha que ninguém responde no runner. O CI seleciona os
certificados de teste com `--cert`.

## 2. Entitlements do app da Mac App Store

| Entitlement | Por quê | Precedente |
|---|---|---|
| `com.apple.security.app-sandbox` | Obrigatório na loja. **Próprio, sem `com.apple.security.inherit`**: o host é iniciado pelo navegador, não por um pai sandboxed | Bitwarden `entitlements.desktop_proxy.plist` (app-sandbox + app group, sem inherit) |
| `com.apple.security.smartcard` | `TKSmartCardSlotManager` e PC/SC (leitores e ATR no diagnóstico) e módulos PKCS#11 que falam com o cartão de dentro do nosso processo. Itens de token no keychain **não** precisam dele | web-eid-app `web-eid-safari.entitlements`; doc da Apple: *"requires this entitlement for sandboxed applications that access smart cards using legacy PCSC framework APIs"*; SafeSign (prova 3) exige do app hospedeiro |
| `com.apple.security.device.usb` | Lista USB (VID:PID) no diagnóstico (`nusb`) | Bitwarden MAS |
| `com.apple.security.application-groups` = `TEAMID.dev.websign` | Socket da ponte do Safari (§5) e do complemento, se existir | Bitwarden (socket IPC no contêiner do grupo); web-eid (grupo compartilhado app↔appex) |
| `…temporary-exception.files.home-relative-path.read-write` | Só as pastas `NativeMessagingHosts` de Chrome (+Beta/Dev/Canary), Chromium, Edge (+Beta/Dev/Canary), Brave (+Beta/Nightly), Vivaldi, Opera (+Developer) e `Mozilla/` | Bitwarden MAS (mesma técnica; ele não lista Brave nem Opera); Brave e Vivaldi conferidos no KeePassXC; Opera pela regra do Chromium (`<user data dir>/NativeMessagingHosts`), **a confirmar** |
| `com.apple.application-identifier`, `com.apple.developer.team-identifier` | Exigidos pela loja; `TEAMID` é `TODO(gustavo)` | Bitwarden MAS |

Sem `network.client` (o app não usa rede), sem `cs.allow-jit` (não é Electron), sem
`files.user-selected` (o `.pfx` é importado pelo próprio macOS). **Arc** fica de fora até
confirmar a pasta num Mac real (`~/Library/Application Support/Arc/User Data/NativeMessagingHosts/`,
não verificado).

**Achado que muda o código do `register`:** dentro da sandbox, `$HOME` aponta para
`~/Library/Containers/<bundle id>/Data`. O Bitwarden resolve isso no Electron com
`os.userInfo().homedir` e documenta no Rust (`desktop_native/core/src/ipc/mod.rs`): *"While running
sandboxed, it's different: /Users/<user>/Library/Containers/com.bitwarden.desktop/Data"*. O
`std::env::home_dir()` lê `$HOME` → gravaria **dentro do contêiner**, onde nenhum navegador procura.
Por isso o `nm/register` do kit, no macOS, pega o home do banco de usuários
(`getpwuid_r(getuid())`, em `probe/src/nm/register/home.rs`), que a sandbox não redireciona. O
`sandbox-test.sh` detecta e aponta explicitamente o caso de o manifesto cair dentro do contêiner.

## 3. O experimento da sandbox (`sandbox-test.sh`)

Monta `WebeSign.app` com o `websign-probe` dentro, assina **ad hoc** com os entitlements acima
(menos os que exigem perfil de provisionamento) e, de dentro da sandbox:

| ID | Experimento | SIM significa |
|---|---|---|
| `sandbox` | Roda `--version` | O macOS criou `~/Library/Containers/dev.websign.app`: a sandbox foi aplicada |
| `register:<navegador>` | `register --browser chrome --browser edge --browser brave --browser firefox` (pastas dos navegadores criadas antes, para simular instalação) | O manifesto apareceu na pasta **real** e aponta para o binário sandboxed |
| `host-stdio` | Inicia o binário como o Chrome inicia (`chrome-extension://<dev_id>/` + mensagens com prefixo de 4 bytes) e manda `ping` e `list` | O host sandboxed respondeu as duas |
| `keychain-list`, `keychain-sign` | Keychain de teste em `~/Library/Keychains`, na lista de busca; `list` e `sign` (PKCS#1, PSS, ECDSA) | A sandbox enxerga keychains da lista de busca e usa as chaves (o resultado também mostra o mesmo teste fora da sandbox, para separar causa) |
| `ctk-query` | A consulta com `kSecAttrAccessGroupToken` | Rodou sem erro (o runner não tem token) |
| `pkcs11-*` | Ver [prova 3](3-tokens-mac.md#3-pkcs11-dentro-da-sandbox) | — |

No fim, o script imprime as negações que a sandbox registrou no log (`sender == "Sandbox"`) e
desfaz tudo: manifestos (restaurando os que existiam), pastas criadas, keychain e lista de busca, e o
contêiner se foi ele que o criou.

### O que um binário ad hoc prova — e o que não prova

| Prova | Não prova (só assinatura da Apple / TestFlight / loja) |
|---|---|
| As regras do perfil de sandbox para **estes** entitlements: arquivos fora do contêiner, keychain, `dlopen`, PC/SC | Que a **App Review aceita** as `temporary-exception` (precisam de justificativa por escrito) e o carregamento de módulos PKCS#11 externos (diretriz 2.5.2) |
| O contêiner e o `$HOME` redirecionado | O **app group** com Team ID (removido no teste: exige perfil) e o aviso do macOS 15 para grupos não autorizados por perfil |
| Que um binário sandboxed iniciado por processo não sandboxed funciona como host de stdio | O comportamento de um app instalado pela loja/TestFlight (quarentena, Gatekeeper, caminho em `/Applications`) |
| Partições do keychain para código ad hoc (`cdhash:`) | O diálogo de acesso ao keychain para o app com Team ID (`teamid:`), que é o que o usuário verá |
| — | Qualquer coisa do Safari: a extensão só roda assinada (ou com "Allow Unsigned Extensions" no Safari) |

## 4. O que foi provado em CI

Run [36629289998](https://github.com/diagnos-tech/web-esign/actions/runs/36629289998), job `macos`,
em 29/09/2026: **macOS 26.6.2 (25G83), arm64**, binário assinado ad hoc.

**Keychain** (`ci-macos.sh`, keychain descartável com 3 identidades de teste; todas as assinaturas
conferidas com `probe-core::verify`):

```
RSA-2048   SHA-256/384/512 × RSASSA-PKCS1-v1_5 e RSASSA-PSS   6 × OK via SecKeyCreateSignature (9–14 ms)
EC P-256   SHA-256/384/512 × ECDSA (DER → r‖s)                 3 × OK (5–7 ms)
EC P-384   SHA-256/384/512 × ECDSA (DER → r‖s)                 3 × OK (8–14 ms)
All required checks passed.
```

**Sandbox** (`sandbox/sandbox-test.sh`: `.app` com os entitlements da loja, assinado ad hoc):

| Experimento | Resultado | Detalhe |
|---|---|---|
| sandbox aplicada | **SIM** | o macOS criou o contêiner `~/Library/Containers/dev.websign.app/Data` |
| `register` grava no home real (Chrome, Edge, Brave, Firefox) | **SIM** | os quatro manifestos em `~/Library/Application Support/<navegador>/NativeMessagingHosts/` apontam para o binário sandboxed; navegadores sem pasta são ignorados |
| host sandboxed iniciado como o Chrome inicia (origem + stdio) | **SIM** | respondeu `pong` e `certificates`; log do host dentro do contêiner |
| keychain listado dentro da sandbox | **SIM** | as duas identidades de teste |
| assinatura dentro da sandbox (PKCS#1 v1.5, PSS, ECDSA) | **SIM** | três OK, conferidas |
| consulta CryptoTokenKit (`kSecAttrAccessGroupToken`) dentro da sandbox | **SIM** (sem token no runner) | a consulta roda sem entitlement extra |
| PKCS#11 (`dlopen` do SoftHSM2 em `/opt/homebrew`) | **NÃO** | `file system sandbox blocked open()`; o kernel registra `deny(1) file-read-data /opt/homebrew/Cellar/softhsm/…` |
| idem com *hardened runtime* | **NÃO** | mesma negação (a sandbox barra antes da validação de biblioteca) |

| Item | Resultado | Versão do macOS / runner |
|---|---|---|
| Keychain: RSA-2048 PKCS#1 v1.5 + PSS × SHA-256/384/512 | ✅ | 26.6.2 arm64 |
| Keychain: P-256 e P-384 ECDSA × SHA-256/384/512 (DER → `r‖s`) | ✅ | 26.6.2 arm64 |
| Sandbox aplicada | ✅ | 26.6.2 arm64 |
| Manifestos gravados pela sandbox (Chrome, Edge, Brave, Firefox) | ✅ | 26.6.2 arm64 |
| Host sandboxed responde por stdio | ✅ | 26.6.2 arm64 |
| Keychain lido e usado dentro da sandbox | ✅ | 26.6.2 arm64 |
| Módulo PKCS#11 fora do pacote carregado dentro da sandbox | ❌ | 26.6.2 arm64 |

## 5. Safari: arquitetura da ponte

A Apple exige que a extensão do Safari venha dentro de um app, com uma *app extension*
(`.appex`, ponto de extensão `com.apple.Safari.web-extension`) cuja classe
`SafariWebExtensionHandler` recebe cada `browser.runtime.sendNativeMessage()`. A appex é um
processo à parte, sandboxed, **sem interface**, que o Safari inicia e encerra quando quer.

```
página ──SDK──▶ content script ──▶ service worker da extensão (Safari)
                                     │ browser.runtime.sendNativeMessage({...,"origin": sender.origin})
                                     ▼
                     SafariWebExtensionHandler (.appex, sandboxed, sem UI)
                                     │ socket Unix em ~/Library/Group Containers/TEAMID.dev.websign/
                                     │ (mesmo enquadramento do native messaging: 4 bytes + JSON)
                                     ▼
                     WebeSign.app (Rust) — mesmo tratador do host do Chrome
                       janela de confirmação (egui) → SecKeyCreateSignature → PIN do sistema
```

**Assinar direto na appex?** Não. A appex conseguiria chamar `SecKeyCreateSignature` (o diálogo de
PIN é do sistema), mas a decisão 5 exige a janela de confirmação do app — site, certificado,
impressão digital, botão Assinar — e a appex não mostra janela. Além disso, duplicaria em Swift o
que o app faz em Rust. A appex fica burra: repassa bytes e não conhece o protocolo, então não muda
quando o protocolo mudar.

**Como a janela aparece?** A appex inicia o app (`NSWorkspace.openApplication`, `activates = false`)
se ele não estiver escutando; o app, já com o pedido, traz a janela de confirmação para a frente
(`NSApp.activate`), como faz quando o pedido vem do Chrome. O app fica aberto com fechamento por
ociosidade (decisão 6), então a partir do segundo pedido não há partida a frio.

**Por que socket no app group, e não o que o web-eid faz?** O web-eid-app usa
`NSDistributedNotificationCenter` (difusão para qualquer processo do usuário) + `UserDefaults` do
grupo, com espera ativa (`sleepForTimeInterval` em laço) e encerra o app a cada pedido. Funciona,
mas é lento, e o nome da notificação é público. XPC com *Mach service* exigiria registrar o serviço
no launchd (login item/`SMAppService`) e bindings de XPC/Objective-C no Rust. O socket Unix no
contêiner do grupo é o que o Bitwarden publica na loja (`~/Library/Group Containers/<grupo>/s.<nome>`),
é trivial dos dois lados e reaproveita o enquadramento do native messaging — o tratador do app é o
mesmo para Chrome, Firefox e Safari.

**Segurança da ponte.** Qualquer processo do usuário fora da sandbox pode tentar conectar no socket.
O app deve conferir o par: `getsockopt(LOCAL_PEERTOKEN)` → *audit token* →
`SecCodeCopyGuestWithAttributes` → `SecCodeCheckValidity` com o requisito
`anchor apple generic and certificate leaf[subject.OU] = "TEAMID" and identifier "dev.websign.app.extension"`.
A origem do site vem do Safari (`sender.origin`/`sender.url` no service worker), nunca do payload da
página — mesma regra do Chrome.

**Estado da extensão.** `SFSafariExtensionManager.getStateOfSafariExtension(withIdentifier:)` e
`SFSafariApplication.showPreferencesForExtension(withIdentifier:)` só funcionam no app que contém a
extensão. No app Rust, via `objc2-safari-services`; alimentam a aba Navegadores do diagnóstico e o
botão "ativar no Safari". Extensão desativada não envia ping, então essa consulta é a única fonte
para "instalada mas desligada".

### Proposta de código da appex (~110 linhas sem comentários)

```swift
// SafariWebExtensionHandler.swift — relay between the Safari web extension and the app.
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Safari hands every browser.runtime.sendNativeMessage() to this extension. It forwards the
// message unchanged to the app over a Unix socket in the shared app group container, framed
// like native messaging, and returns the app's reply. The app owns everything else: the
// confirmation window, the signing, the OS PIN dialog. This file never parses the protocol.

import AppKit
import Foundation
import SafariServices
import os

private let appBundleID = "dev.websign.app"
private let appGroupID = "TEAMID.dev.websign"  // TODO(gustavo): real Team ID
private let socketName = "s.host"  // short: sun_path holds 104 bytes
private let startTimeout: TimeInterval = 10
private let maxReplyBytes = 1 << 20  // native messaging's host-to-browser limit
private let log = Logger(subsystem: "dev.websign.app.extension", category: "relay")

final class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {
    /// One request at a time: the app shows one confirmation window at a time anyway.
    private static let queue = DispatchQueue(label: "dev.websign.relay")

    func beginRequest(with context: NSExtensionContext) {
        let item = context.inputItems.first as? NSExtensionItem
        let message = item?.userInfo?[SFExtensionMessageKey]
        Self.queue.async {
            let reply: Any
            do {
                reply = try Relay.exchange(message)
            } catch {
                log.error("relay failed: \(String(describing: error), privacy: .public)")
                // Same envelope as the host's own errors (nm/protocol.rs), so the SDK sees one shape.
                let id = (message as? [String: Any])?["id"] ?? NSNull()
                reply = ["v": 1, "id": id, "ok": false,
                         "error": ["code": "app_unavailable", "message": "\(error)"]] as [String: Any]
            }
            let response = NSExtensionItem()
            response.userInfo = [SFExtensionMessageKey: reply]
            context.completeRequest(returningItems: [response], completionHandler: nil)
        }
    }
}

enum RelayError: Error { case notJSON, noAppGroup, appNotFound, tooLarge, closed, posix(Int32) }

enum Relay {
    static func exchange(_ message: Any?) throws -> Any {
        guard let message, JSONSerialization.isValidJSONObject(message) else { throw RelayError.notJSON }
        let fd = try connectToApp()
        defer { close(fd) }
        let body = try JSONSerialization.data(withJSONObject: message)
        var length = UInt32(body.count)  // native byte order, like native messaging
        try writeAll(Data(bytes: &length, count: 4) + body, to: fd)
        let header = try readExactly(4, from: fd)
        let replyLength = Int(header.withUnsafeBytes { $0.loadUnaligned(as: UInt32.self) })
        guard replyLength <= maxReplyBytes else { throw RelayError.tooLarge }
        return try JSONSerialization.jsonObject(with: readExactly(replyLength, from: fd))
    }

    /// Connects to the app's socket, starting the app once if nobody is listening yet.
    private static func connectToApp() throws -> Int32 {
        guard let group = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroupID)
        else { throw RelayError.noAppGroup }
        let path = group.appendingPathComponent(socketName).path
        let deadline = Date().addingTimeInterval(startTimeout)
        var launched = false
        while true {
            let fd = socket(AF_UNIX, SOCK_STREAM, 0)
            guard fd >= 0 else { throw RelayError.posix(errno) }
            if connectUnix(fd, path) == 0 { return fd }
            let failure = errno
            close(fd)
            guard failure == ENOENT || failure == ECONNREFUSED, Date() < deadline else {
                throw RelayError.posix(failure)
            }
            if !launched {
                try launchApp()
                launched = true
            }
            Thread.sleep(forTimeInterval: 0.1)
        }
    }

    private static func launchApp() throws {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: appBundleID)
        else { throw RelayError.appNotFound }
        let configuration = NSWorkspace.OpenConfiguration()
        configuration.activates = false  // the app raises its own confirmation window
        configuration.addsToRecentItems = false
        NSWorkspace.shared.openApplication(at: url, configuration: configuration)
    }

    private static func connectUnix(_ fd: Int32, _ path: String) -> Int32 {
        var address = sockaddr_un()
        address.sun_family = sa_family_t(AF_UNIX)
        let bytes = Array(path.utf8)
        guard bytes.count < MemoryLayout.size(ofValue: address.sun_path) else {
            errno = ENAMETOOLONG
            return -1
        }
        withUnsafeMutableBytes(of: &address.sun_path) { $0.copyBytes(from: bytes) }
        return withUnsafePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
            }
        }
    }

    private static func writeAll(_ data: Data, to fd: Int32) throws {
        try data.withUnsafeBytes { raw in
            var offset = 0
            while offset < raw.count {
                let written = write(fd, raw.baseAddress! + offset, raw.count - offset)
                guard written > 0 else { throw RelayError.posix(errno) }
                offset += written
            }
        }
    }

    private static func readExactly(_ count: Int, from fd: Int32) throws -> Data {
        var data = Data(count: count)
        try data.withUnsafeMutableBytes { raw in
            var offset = 0
            while offset < count {
                let got = read(fd, raw.baseAddress! + offset, count - offset)
                guard got > 0 else { throw got == 0 ? RelayError.closed : RelayError.posix(errno) }
                offset += got
            }
        }
        return data
    }
}
```

Lado do app (Rust), fora do escopo desta prova: `UnixListener` em
`~/Library/Group Containers/TEAMID.dev.websign/s.host` (o `$HOME` da sandbox não serve; mesmo cálculo
do Bitwarden), verificação do par descrita acima e o **mesmo** laço de mensagens do host de stdio.

Riscos a medir no Mac real: o Safari pode encerrar a appex se o pedido demorar (usuário digitando
PIN por um minuto?) — medir com espera longa na janela de confirmação; e a primeira abertura do app
pela appex com `activates = false` precisa trazer a janela para frente mesmo assim.

## 6. O que falta

- [ ] Log do CI (§4) — keychain e sandbox ad hoc.
- [x] `register` usa o home real no macOS (`getpwuid_r`, achado do §2). Falta reexecutar o CI.
- [ ] Chrome real iniciando o host sandboxed e assinando com A1 do keychain e com token CTK.
- [ ] Firefox real (lê `~/Library/Application Support/Mozilla/NativeMessagingHosts/`).
- [ ] Safari: projeto Xcode com a appex acima, extensão carregada, assinatura ponta a ponta,
      `getStateOfSafariExtension` com a extensão ligada e desligada.
- [ ] TestFlight: o mesmo app assinado pela Apple (Team ID, app group, partição `teamid:` do keychain).
- [ ] App Review: enviar build com as exceções e ver se passa (é a única prova da aceitação).

## 7. Roteiro para o Gustavo (Mac real)

Pré-requisitos: macOS 14 ou 15 (anotar a versão exata e o chip), Xcode Command Line Tools, Rust,
Homebrew, Chrome e Firefox instalados **e abertos uma vez** (para criarem as pastas).

1. **Compilar:** `cd docs/prototypes/kit && cargo build --release -p websign-probe && export PROBE_EXE=$PWD/target/release/websign-probe`.
2. **Keychain com chaves de teste:** `bash macos/ci-macos.sh` → deve terminar em "All required
   checks passed." (usa um keychain temporário; não mexe no login).
3. **Sandbox:** `bash macos/sandbox/sandbox-test.sh` → copiar a tabela `RESULT` e as negações.
4. **A1 real (opcional):** importar um `.pfx` pelo Acesso às Chaves; `"$PROBE_EXE" list` → a linha deve
   dizer `macos:keychain (keychain, software; PIN by OS)`. `"$PROBE_EXE" sign --cert <16 hex> --hash all --pss`
   → o macOS pergunta se permite usar a chave: testar **Negar** (deve sair "cancelled by the user"),
   depois **Permitir**.
5. **Mesmo teste dentro da sandbox:** `bash macos/sandbox/run-sandboxed.sh list` e
   `bash macos/sandbox/run-sandboxed.sh sign --cert <16 hex> --hash sha256`.
   Anotar se o diálogo de permissão aparece e o que ele diz.
6. **Chrome iniciando o host sandboxed:**
   `bash macos/sandbox/run-sandboxed.sh register --browser chrome` (o bundle fica em
   `~/Library/Caches/dev.websign.sandbox-test/WebeSign.app`); conferir
   `cat ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/dev.websign.host.json`.
   Carregar a extensão de desenvolvimento (`kit/extension/`, ID fixo `nhnkdpljdgjflbflkhnkmfmcmodboeii`)
   em `chrome://extensions` (modo desenvolvedor → "Carregar sem compactação") e disparar `ping` e
   `list`. Evidência: resposta na extensão, o log do host (na sandbox o `TMPDIR` também vai para o
   contêiner: `find ~/Library/Containers/dev.websign.app -name '*-probe-host.log'`) e
   `log show --last 5m --predicate 'sender == "Sandbox"' --style compact | grep websign`.
7. **Token via CryptoTokenKit:** ver o roteiro da [prova 3](3-tokens-mac.md#4-roteiro-de-teste-para-o-gustavo)
   — rodar `list`/`sign` fora e dentro da sandbox (`run-sandboxed.sh`), e pelo Chrome (passo 6).
8. **Safari:** quando existir `safari/` (projeto Xcode com a appex do §5): Safari → Ajustes →
   Avançado → "Mostrar recursos para desenvolvedores"; Desenvolvedor → "Permitir extensões não
   assinadas"; ativar a extensão; assinar pela página de teste; desligar a extensão e conferir o
   estado no diagnóstico.
9. **Relatório:** `"$PROBE_EXE" report --run-signatures --cert <fp> --hash all --pss --out mac-real.md`
   e anexar aqui (não contém nomes, CPF nem números de série).

## 8. Decisão proposta

- **Um único binário na Mac App Store** = host de native messaging (Chrome, Edge, Brave, Vivaldi,
  Opera, Firefox) + janelas + assinador, com os entitlements de `entitlements.mas.plist`. O `register`
  calcula o home real (`getpwuid_r`), grava só em pastas de navegadores existentes e roda a cada
  abertura do app (e pelo esquema `websign://`, já que a loja não roda instalador).
- **Keychain + CryptoTokenKit primeiro**, pelas duas consultas deste kit. PKCS#11 dentro do app da
  loja só se a [prova 3](3-tokens-mac.md) mostrar que funciona **e** a App Review aceitar; senão,
  complemento `.dmg` (Developer ID, notarizado, Sparkle), chamado pelo mesmo socket do app group.
- **Safari:** appex de ~100 linhas que só repassa, socket Unix no app group, confirmação e assinatura
  no app; estado da extensão por `SFSafariExtensionManager` a partir do app.

Reverter se: a sandbox negar a escrita nas pastas mesmo com as exceções (→ o app não consegue se
registrar sozinho; alternativa seria o complemento registrar), o Chrome não conseguir iniciar o binário
sandboxed, ou o Safari encerrar a appex antes do usuário confirmar (→ appex responde "pendente" e a
extensão consulta o app de novo).

## Referências

- Bitwarden (GPL-3.0): `apps/desktop/resources/entitlements.mas.plist`,
  `entitlements.desktop_proxy.plist`, `src/main/native-messaging.main.ts` (`userInfo().homedir` no
  macOS), `desktop_native/core/src/ipc/mod.rs` (socket no app group).
- web-eid-app (MIT): `src/mac/safari-extension.mm`, `main.mm`, `shared.hpp`,
  `web-eid-safari*.entitlements`.
- Chromium: `net/ssl/client_cert_store_mac.cc`.
- Apple: documentação de `kSecAttrAccessGroupToken`, `com.apple.security.smartcard` e "Using
  cryptographic assets stored on a smart card"; open source `Security`: `keychain/headers/SecKey.h`,
  `OSX/sec/Security/SecKeyAdaptors.m`, `OSX/sec/Security/SecCTKKey.m`; `TKError.h`
  (`TKErrorCodeCanceledByUser = -4`, `TKErrorCodeAuthenticationFailed = -5`).
- KeePassXC: `src/browser/NativeMessageInstaller.cpp` (pastas de Brave, Vivaldi, Edge no macOS).
