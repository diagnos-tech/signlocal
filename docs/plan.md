# Plano de implementação

> Para aprovar **antes** de escrever o produto. O que está aqui já foi exercitado com código real
> nas [provas de risco](prototypes/); o que ainda depende de hardware ou de conta em loja está
> marcado como pendente e tem roteiro.

## 0. Onde estamos

| Prova | Resposta até agora | Falta (com roteiro) |
|---|---|---|
| 1. Windows | **CI (Windows Server 2025):** o CNG assina RSA PKCS#1 v1.5, PSS e ECDSA P-256/384 nos 3 hashes. O CAPI legado assina PKCS#1 (o A1 em `PROV_RSA_FULL` é reaberto no CSP AES). Com `prefer`, as chaves dos CSPs da Microsoft passam pela ponte CNG e assinam **também PSS**. Nada abre diálogo com `--silent` ([1-windows.md](prototypes/1-windows.md)) | MSIX no runner (em ajuste) · 4 tokens reais · navegadores pelo alias num Windows cliente · aceite da capacidade restrita no Partner Center |
| 2. Mac | **CI (macOS 26.6, arm64):** a partir de um `.app` na sandbox com os entitlements da loja, o app grava os manifestos no home real (Chrome, Edge, Brave, Firefox). O host iniciado como o Chrome o inicia responde, e o Keychain assina PKCS#1 v1.5, PSS e ECDSA. A ponte do Safari está só desenhada ([2-mac.md](prototypes/2-mac.md)) | Mac real com Chrome e Safari · token no CryptoTokenKit · App Review |
| 3. Tokens no Mac | **CI:** um módulo PKCS#11 de fora do pacote **não carrega** dentro da sandbox (`file system sandbox blocked open()`). Logo, token que só funciona por PKCS#11 (ex.: DNIe) exige o complemento `.dmg`. SafeSign, OpenSC e Cartão de Cidadão expõem o token pelo CryptoTokenKit ([3-tokens-mac.md](prototypes/3-tokens-mac.md)) | Tokens reais: fechar quais middlewares ficam fora do CryptoTokenKit |
| 4. Linux | A cadeia inteira funciona aqui: Chromium → extensão → host → PKCS#11 (SoftHSM2 direto e via p11-kit) → assinatura verificada. RSA v1.5/PSS e ECDSA P-256/384/521 nos 3 hashes, inclusive chave com `CKA_ALWAYS_AUTHENTICATE`; mesmo certificado por dois módulos deduplicado ([4-linux.md](prototypes/4-linux.md)) | Token real via p11-kit · Firefox Snap (portal) num Ubuntu com interface gráfica |

O kit ([`docs/prototypes/kit`](prototypes/kit/)) roda no CI nos três sistemas com chaves de
software e publica os binários do `websign-probe` (e o `.msix`) como artefatos, para os testes com
tokens reais **sem compilar nada**.

## 1. Decisões que dependem de você

Cada uma tem uma proposta; sem resposta, sigo a proposta.

| # | Decisão | Proposta | Por quê |
|---|---|---|---|
| D1 | `sign()` recebe `prepare(certificate)` e devolve o digest para o certificado escolhido ([ux R1](ux.md#r1-uma-janela-por-assinatura-com-o-certificado-escolhido-nela)) | **Sim** | O PAdES precisa do certificado **antes** do digest (`signing-certificate-v2`). Assim basta uma janela por assinatura, e não duas. O adaptador `ExternalPdfSigner` do Diagnos chama `prepare` dentro do `sign` |
| D2 | `certificates()` devolve só o certificado escolhido, nunca a lista do computador ([ux R2](ux.md#r2-certificates-devolve-só-o-que-o-usuário-escolheu)) | **Sim** | Em clínica o computador é compartilhado: a lista inteira entregaria nome e CPF de outros médicos a qualquer site |
| D3 | Idioma de código e comentários | **Inglês** (docs em pt-BR, commits em pt-BR) | O projeto é internacional (eIDAS, América Latina); é o padrão para atrair contribuidores |
| D4 | Assinatura em lote (30 laudos/dia) | **Fora da v1**; a fila de pedidos já existe | Uma confirmação para N documentos muda o modelo de consentimento; melhor medir o uso antes |
| D5 | PIN do PKCS#11 por sessão ou a cada assinatura | **Por sessão** (até o token sair ou a conexão ociosa fechar), exceto chaves com `CKA_ALWAYS_AUTHENTICATE` | Mesmo comportamento do CNG/CryptoTokenKit, que o médico já conhece |
| D6 | Nome, domínio e IDs definitivos | Seguir com os provisórios do [`project.toml`](../project.toml) até a busca de marca | Trocar é editar um arquivo |
| D8 | Leitor de certificados | **Leitor DER próprio, tolerante** (~1 000 linhas, só resume; **não valida cadeia**), com fuzzing contínuo no CI | A pilha `x509-cert`/`der` fazia certificados reais sumirem da lista (§3). Validar cadeia continua sendo do site |
| D7 | Assinatura dos commits | Os commits têm você como autor e `Signed-off-by`. **A chave SSH configurada neste ambiente não é a sua** (é a do ambiente de execução) | Para commits assinados com a sua chave, reassine ao integrar (`git rebase --exec 'git commit --amend --no-edit -S'`) |

Pendências menores de UX estão em [ux.md §17](ux.md#17-pendências).

## 2. Arquitetura

### 2.1 Fluxo de uma assinatura (D1 aprovado)

```
site (SDK)            extensão (MV3)             app (host de native messaging)          SO / PKCS#11
   │ sign({hash,prepare})  │                              │                                      │
   │──── postMessage ─────▶│── origem vem do navegador ──▶│ abre a Confirmação (lista, origem)   │
   │                       │                              │── lista (cache; PC/SC invalida) ────▶│
   │◀── prepare(cert) ─────│◀───── need_digest(cert) ─────│ usuário escolhe o certificado        │
   │──── digest ──────────▶│──────── digest ─────────────▶│ mostra o código de conferência       │
   │                       │                              │ [Assinar] → PIN (SO ou nosso campo)──▶│
   │◀── {cert, chain,      │◀──────── signature ──────────│◀──────────── assinatura crua ────────│
   │     algorithm, sig} ──│                              │                                      │
```

A extensão mantém a porta de native messaging aberta enquanto houver uso e a fecha após um tempo
ocioso. Assim, drivers PKCS#11 são carregados uma vez por sessão. "Abrir diagnóstico" dispara **outro processo** do mesmo binário, para a janela sobreviver ao fechamento da porta.

### 2.2 Repositório

```
Cargo.toml                 workspace (exclui docs/prototypes/kit)
package.json               bun workspaces: sdk, extension, site, e2e
project.toml               nomes e IDs provisórios (fonte única)
crates/
  websign-core/            lógica pura, promovida do probe-core + código de conferência, origem, nome do titular
  websign-protocol/        mensagens serde versionadas → gera sdk/src/generated/protocol.ts (ts-rs)
  websign-keystores/       Windows (CNG/CAPI), macOS (Keychain/CTK), PKCS#11 — promovidos do kit
  websign-devices/         USB (nusb), PC/SC (pcsc), dicas do devices.json
  websign-host/            motor da sessão: framing, despacho, consentimento, fila, ociosidade, cache
app/                       binário `websign`: main, janelas egui, registro nos navegadores, plataforma
  src/ui/                  theme.rs, fontes, ícones, i18n, confirm/, diagnostics/, widgets/
  i18n/                    pt-BR.toml, en.toml (+ pt-PT, es, fr)
extension/                 WXT: background, content, popup (TS + CSS puros)
sdk/                       @websign/sdk — Apache-2.0, zero dependências
safari/                    projeto Xcode: appex de ~100 linhas que só repassa
e2e/                       Playwright: página → extensão → app → chave de software
devices.json               + devices.schema.json + LICENSE-CC0
packaging/                 windows/msix (e wix só se a prova 1 reprovar), macos/, linux/nfpm
site/                      GitHub Pages: download, privacidade, /ativar
xtask/                     `cargo xtask gen|e2e|package` — uma porta de entrada para tudo
```

**Por que vários crates:** o motor da sessão e os keystores são testáveis sem compilar o egui/wgpu;
cada crate tem uma responsabilidade e um `SPEC.md`; o `websign-core` continua puro (o que o TDD cego exige).

### 2.3 Protocolo

- Definido **uma vez** em Rust (`websign-protocol`), com `"v": 1`. Os tipos TypeScript são **gerados**,
  e o CI falha se o arquivo gerado estiver desatualizado.
- **Extensão ↔ app:** `hello` (negocia versões; a extensão carrega `MIN_APP_VERSION`), `choose`, `sign.begin`,
  `sign.need_digest`, `sign.digest`, `sign.result`, `cancel`, `status`, `open_diagnostics`.
  Os códigos de erro são os da [ux.md §15](ux.md#15-erros).
- **Página ↔ extensão:** a extensão se anuncia (`announce`) e faz o repasse, validando formato, tamanho e origem.
  A página nunca fala com o app diretamente.

### 2.4 Segurança (o que cada peça garante)

| Ameaça | Defesa | Onde |
|---|---|---|
| Página maliciosa se passa por outra | Origem vem do `MessageSender` do navegador, nunca do payload; `http:` não local recusado | extensão, host |
| Outra extensão fala com o app | `allowed_origins`/`allowed_extensions` só com os IDs das nossas extensões | manifestos |
| Clique induzido / janela imitada | Janela do app (a página não desenha nela); botão Assinar arma após 600 ms com foco; foco inicial nunca no Assinar | app/ui |
| Digest adulterado ou truncado | Tamanho exato do hash declarado (32/48/64); código de conferência mostrado nos dois lados | core, host, SDK |
| Vazamento de PIN | PIN só na janela do app (PKCS#11) ou do SO; buffer zerado; `EnableSecureEventInput` no Mac | app, keystores |
| Rastreamento entre sites | `certificates()` só devolve o escolhido; consentimento por origem, revogável | host, SDK |
| Protocolo antigo/estranho | Versão explícita; campo desconhecido rejeitado; limite de tamanho | protocol, host |
| Dados pessoais em logs | Logs só com tipos, tamanhos e códigos; "copiar diagnóstico" com conteúdo definido | host, app |
| Verificador fraco | A verificação do kit rejeita `s ≥ n` no PSS (achado do TDD) | core |

## 3. Como trabalhamos (TDD cego em três papéis)

1. **Especificação** (Opus): `SPEC.md` por módulo com a API pública em código (corpos `todo!()`),
   comportamento, erros e exemplos. Vetores de teste vêm de normas e da [ux.md §16](ux.md#16-casos-de-teste-de-referência).
2. **Testes e implementação às cegas, em paralelo** (Sonnet, cada um numa git worktree isolada criada do
   mesmo commit): um escreve os testes, o outro o código; nenhum lê o trabalho do outro.
3. **Revisão crítica** (Opus): junta, roda, decide cada divergência pela especificação e pelas normas,
   fixa a regra no `SPEC.md`, faz revisão de segurança e qualidade.

Na Fase 0 o método já se pagou no `probe-core`:
- 286 de 290 testes cegos passaram de primeira.
- As 4 divergências mostraram que a pilha `x509-cert`/`der` **descartava certificados reais**: UTCTime anterior
  a 1970, `UniversalString` e OIDs com arcos acima de 32 bits.
- As 24 dúvidas levantadas pelos dois lados viraram regras explícitas no `SPEC.md`.
- A implementação achou uma maleabilidade no verificador PSS do crate `rsa` (aceitava `s + n`).
- Hoje: 391 testes, 460 mil entradas mutadas sem pânico, nenhum arquivo acima de ~210 linhas.

Onde o TDD cego não cabe inteiro:

- **Adaptadores de SO** (Win32, Security.framework, PKCS#11): uma **suíte de contrato** única contra
  o trait `Keystore`, escrita antes, que roda no CI de cada SO com chaves de software (KSP de software,
  keychain de teste, SoftHSM2). Tokens reais entram na matriz manual [`docs/compatibility.md`](compatibility.md).
- **Interface:** a máquina de estados da Confirmação é pura (TDD cego normal). As telas são testadas com
  `egui_kittest` (consultas pela árvore do AccessKit: "botão Assinar desabilitado nos primeiros 600 ms")
  mais snapshots visuais revisados pelo revisor.

Portões de qualidade no CI (todos obrigatórios para integrar):

- Rust: `fmt`, `clippy -D warnings`, testes, `cargo-deny` (licenças compatíveis com GPL-3.0, advisories,
  duplicatas), `cargo-llvm-cov` (core e host ≥ 90%), `cargo-mutants` semanal no core, `cargo-fuzz` no leitor
  DER, no framing de native messaging e no parser do protocolo.
- TypeScript: `tsc` estrito, Biome, Vitest, limite de tamanho (SDK < 5 KB gzip, popup < 15 KB).
- Protocolo: TS gerado atualizado. E2E: Chromium (Linux/SoftHSM e Windows/CNG) a cada PR; macOS diário.
- Release: binário sem a feature de teste `auto-confirm` (checado no CI).

Agentes: **Opus** para especificações, adaptadores de SO, interface e revisões; **Sonnet** para testes cegos,
módulos puros bem especificados, SDK/extensão, scripts e empacotamento. Uma trilha por fronteira de arquivos.

## 4. Fases

Cada trilha lista **arquivos** (fronteira), **depende de** e **aceite**. Trilhas da mesma fase rodam em paralelo.

### Fase 1 — Núcleo (começa com a aprovação deste plano; roda enquanto os tokens são testados)

| Trilha | Arquivos | Depende de | Aceite |
|---|---|---|---|
| 1.1 Workspace e portões | `Cargo.toml`, `package.json`, `xtask/`, `.github/workflows/ci.yml`, `deny.toml`, `biome.json`, `tsconfig.base.json`, `rust-toolchain.toml` | — | CI verde vazio; `cargo-deny` barra uma dependência GPL-incompatível de teste |
| 1.2 `websign-core` | `crates/websign-core/**` | 1.1 | O `probe-core` promovido, mais o código de conferência, a formatação da origem (domínio registrável via PSL) e o nome do titular. Todos os vetores da ux §16 passam; cobertura ≥ 90%; mutantes mortos ≥ 85% |
| 1.3 `websign-protocol` | `crates/websign-protocol/**`, `sdk/src/generated/**` | 1.1 | Ida e volta serde; versão desconhecida → erro tipado; TS gerado conferido no CI |
| 1.4 `websign-keystores` | `crates/websign-keystores/**` | 1.2 | Suíte de contrato verde nos 3 SOs com chaves de software; `list()` nunca pede PIN (teste); sessão PKCS#11 reaproveitada |
| 1.5 `websign-devices` | `crates/websign-devices/**`, `devices.schema.json` | 1.1 | USB/PC/SC sem serial; eventos de token entrando/saindo invalidam o cache; dicas do devices.json por VID:PID/ATR |
| 1.6 `websign-host` | `crates/websign-host/**` | 1.2–1.5 | Cenários com keystore e UI falsos: fluxo D1 com troca de certificado (novo digest), negação, tempo esgotado, fila cheia, versão errada, consentimento lembrado/revogado |
| 1.7 SDK | `sdk/**` | 1.3 | API `status/certificates/sign/installUrl/onChange/fingerprint`; erros tipados = ux §15; zero dependências; < 5 KB |
| 1.8 Extensão | `extension/**` | 1.3 | Anúncio na página; porta com fechamento por ociosidade; ping em `onStartup/onInstalled`; popup com os 6 estados; < 15 KB |
| 1.9 E2E | `e2e/**` | 1.6–1.8 | Página → extensão → app (`auto-confirm`) → assinatura verificada, no Chromium Linux (SoftHSM2) e Windows (CNG) |

### Fase 2 — Entrega 1: Windows + Chrome/Edge

| Trilha | Arquivos | Depende de | Aceite |
|---|---|---|---|
| 2.1 Base da interface | `app/src/ui/{theme,fonts,icons,i18n,widgets}/**`, `app/i18n/**` | 1.1 | Tokens da ux §11 em `theme.rs`; Inter embutida; Phosphor; claro/escuro do sistema; chave i18n inexistente não compila |
| 2.2 Confirmação | `app/src/ui/confirm/**` | 2.1, 1.6 | Máquina de estados da ux §4.8 (TDD cego) + testes `egui_kittest` de cada estado; abre < 300 ms (medido) |
| 2.3 Diagnóstico | `app/src/ui/diagnostics/**` | 2.1, 1.4, 1.5 | Quatro abas da ux §8. O "copiar diagnóstico" produz exatamente o conteúdo da ux §8.7 (teste de ouro) |
| 2.4 Plataforma Windows | `app/src/platform/windows/**`, `app/src/registration/**` | 1.6 | HKCU para todos os navegadores; alias MSIX; `websign:`; janela de PIN com dono (HWND da Confirmação); `+crt-static`; fallback de software no RDP |
| 2.5 Pacotes | `packaging/windows/msix/**`, build das extensões Chrome/Edge | 2.4 | MSIX instalável por sideload no Windows 10/11. Os zips das lojas ficam prontos, **sem publicar** |

**Aceite da entrega:** você assina um PAdES no Diagnos de homologação com dois tokens A3 (o SafeNet
e o SafeSign), no Windows 11, pelo Chrome e pelo Edge, instalando pelo MSIX. As linhas do
`docs/compatibility.md` precisam estar preenchidas.

### Fase 3 — Entrega 2: Mac (Safari + Chrome)

| Trilha | Arquivos | Depende de | Aceite |
|---|---|---|---|
| 3.1 Plataforma macOS | `app/src/platform/macos/**` | 1.6 | Home real (`getpwuid_r`); registro a cada abertura e por `websign:`; `EnableSecureEventInput` no campo de PIN; estado da extensão do Safari |
| 3.2 Ponte do Safari | `safari/**`, `app/src/platform/macos/safari_socket.rs` | 3.1 | A appex só repassa por um socket no app group; o Safari assina com a confirmação no app |
| 3.3 Pacote da Mac App Store | `packaging/macos/**` | 3.1, 3.2 | Binário universal; entitlements da prova 2; build do TestFlight aprovado (com a sua conta) |
| 3.4 Complemento `.dmg` | `packaging/macos/complement/**` | resultado da prova 3 | **Só se** o critério de [3-tokens-mac.md §1](prototypes/3-tokens-mac.md) mandar |

### Fase 4 — Entrega 3: Firefox + Linux

| Trilha | Arquivos | Depende de | Aceite |
|---|---|---|---|
| 4.1 Firefox | `extension/` (alvo gecko), pacote para a AMO | 1.8 | Firefox 121+ no Windows, Mac e Linux (inclusive o Snap, pelo portal) |
| 4.2 Linux | `packaging/linux/**`, CI do repositório APT/YUM | 1.6 | Instaladores `.deb` e `.rpm` com manifestos de sistema, `pcscd` e `p11-kit`. O repositório estático é assinado com GPG, e `apt upgrade`/`dnf upgrade` atualizam o app |

### Fase 5 — Site, dispositivos e documentação

`site/` (download com detecção do SO, privacidade, `/ativar`, acabamento Stripe), `devices.json` com a
semente de [research/tokens.md](research/) e validação por schema no CI, `docs/compatibility.md`.

## 5. O que preciso de você para as provas

1. Rodar os roteiros com os tokens: [1-windows.md](prototypes/1-windows.md#roteiro-para-os-tokens-reais), [2-mac.md](prototypes/2-mac.md), [3-tokens-mac.md](prototypes/3-tokens-mac.md#4-roteiro-de-teste-para-o-gustavo), [4-linux.md](prototypes/4-linux.md). Os binários prontos ficam nos artefatos do CI (`websign-probe-*`).
2. Contas, **quando chegar a hora** (nada será criado nem pago sem a sua confirmação): Apple Developer
   (US$ 99/ano, para o TestFlight e a Mac App Store), Microsoft Partner Center (para testar a aceitação de
   `unvirtualizedResources`), Chrome Web Store (US$ 5).
3. Aprovar este plano, com D1–D7 respondidas ou a aceitação das propostas.
