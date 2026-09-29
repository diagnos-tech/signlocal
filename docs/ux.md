# WebeSign — Especificação de UX/UI

> Documento de referência para quem implementa `app/` (egui), `extension/` (popup), `sdk/` e `site/`.
> Mockups navegáveis: [`docs/ux/mockups.html`](ux/mockups.html) (claro e escuro; os tokens de lá são
> exatamente os da [§11](#11-tokens-de-design)).
>
> Convenções: cada decisão traz **Por quê:** em uma linha. Pendências do mantenedor aparecem como
> `TODO(gustavo)`. Dimensões em **px lógicos** (pontos do egui, antes da escala de DPI). Textos de interface
> aparecem entre aspas com a chave i18n ao lado quando ela existe (ex.: `confirm.eyebrow`).

## Sumário

1. [Princípios](#1-princípios)
2. [Vocabulário: do termo técnico à palavra na tela](#2-vocabulário)
3. [Requisitos de UX para as outras peças](#3-requisitos-de-ux-para-as-outras-peças)
4. [Janela de Confirmação](#4-janela-de-confirmação)
5. [Lista de certificados](#5-lista-de-certificados)
6. [Possíveis certificados](#6-possíveis-certificados)
7. [Complemento (macOS)](#7-complemento-macos)
8. [Janela de Diagnóstico](#8-janela-de-diagnóstico)
9. [Popup da extensão](#9-popup-da-extensão)
10. [Primeira execução por sistema](#10-primeira-execução-por-sistema)
11. [Tokens de design](#11-tokens-de-design)
12. [Ícones](#12-ícones)
13. [Textos e i18n](#13-textos-e-i18n)
14. [Acessibilidade](#14-acessibilidade)
15. [Erros](#15-erros)
16. [Casos de teste de referência](#16-casos-de-teste-de-referência)
17. [Pendências](#17-pendências)

---

## 1. Princípios

| # | Princípio | Regra que sai dele |
|---|-----------|--------------------|
| P1 | **Confiança vem do que é verificável.** | A janela só mostra o que o app sabe por conta própria: o site vem do navegador, o certificado vem do sistema, o código vem do digest. Nenhum texto enviado pelo site aparece na janela. |
| P2 | **Clareza para quem não é da área.** | Nenhum termo técnico na primeira camada (ver [§2](#2-vocabulário)). Termos técnicos só em "Detalhes" e no Diagnóstico, sempre ao lado da versão leiga. |
| P3 | **Rápido no dia 100, não só no dia 1.** | O médico assina dezenas de laudos por dia: certificado pré-selecionado, PIN com Enter, janela que fecha sozinha no sucesso. Cada segundo a mais se multiplica. |
| P4 | **Anti-phishing por padrão.** | Domínio registrável em destaque, alerta para IP, punycode e site local, `http` bloqueado, botão Assinar armado só após 600 ms com a janela em foco. |
| P5 | **Acessível sem modo especial.** | Contraste AA em ambos os temas, tudo operável por teclado, AccessKit ligado, estado nunca comunicado só por cor. |
| P6 | **Ajuda acionável, nunca beco sem saída.** | Todo estado vazio ou erro diz o que fazer e oferece um botão para isso (baixar driver, reparar, abrir diagnóstico). |
| P7 | **Privacidade visível.** | O usuário vê o que sai do computador: "O PIN fica neste computador", "O site vai receber nome, tipo, emissor e validade", pré-visualização exata do "Copiar diagnóstico". |

---

## 2. Vocabulário

A primeira camada da interface usa só a coluna "Na tela". O termo técnico pode aparecer em "Detalhes" e no
Diagnóstico.

| Termo técnico | Na tela (pt-BR) | Na tela (en) |
|---------------|-----------------|--------------|
| digest / hash | código de conferência | verification code |
| origin | site | site |
| consentimento por origem | permissão do site / "Lembrar este site" | site permission / "Remember this site" |
| PKCS#11 / cryptoki / módulo | driver do token | token driver |
| CNG, CAPI, Keychain, CryptoTokenKit | Windows / Mac / sistema | Windows / Mac / system |
| slot, token PKCS#11 | token | token |
| smart card | cartão | card |
| leitor PC/SC | leitor de cartão | card reader |
| keyUsage / EKU incompatível | "não serve para assinar" / "feito para login" | "can't sign" / "made for login" |
| certificado A1 | "Neste computador" | "On this computer" |
| certificado A3 em hardware | "Token …" / "Cartão no leitor …" | "Token …" / "Card in reader …" |
| native messaging host não registrado | "O navegador não encontra o app" | "The browser can't find the app" |
| protected authentication path | teclado do leitor | reader keypad |
| complemento (helper fora da sandbox) | Complemento | Add-on |
| CKR_PIN_LOCKED | PIN bloqueado | PIN locked |
| PUK / SO PIN | PUK (com explicação) | PUK (with explanation) |

**Por quê:** o público principal é médico, não TI; o suporte de TI encontra os termos técnicos no Diagnóstico.

---

## 3. Requisitos de UX para as outras peças

Estes requisitos nascem da UX e precisam ser respeitados pelas trilhas de SDK, extensão, app e devices.json.
Se o nome de uma API divergir, a trilha dona do código decide o nome; o comportamento é o que importa.

### R1. Uma janela por assinatura, com o certificado escolhido nela

O PAdES exige o certificado **antes** do digest (o atributo `signing-certificate-v2` entra nos atributos
assinados). Por isso o `sign()` do SDK recebe uma função que prepara o digest para o certificado escolhido:

```ts
const result = await websign.sign({
  hash: "SHA-256",
  prepare: async (certificate) => buildSignedAttrsDigest(certificate), // Uint8Array de 32/48/64 bytes
});
// result: { certificate, chain?, signatureAlgorithm, signature }
```

Fluxo: a janela abre → o certificado pré-selecionado vai para a página → a página devolve o digest → a janela
mostra o código de conferência. Se o usuário trocar de certificado, o app pede um novo digest (`prepare` roda
de novo) e o código muda.
**Por quê:** uma janela só por assinatura; o padrão "escolher certificado numa janela e assinar em outra" dobra
os cliques de quem assina 30 laudos por dia.

### R2. `certificates()` devolve só o que o usuário escolheu

- Site **não lembrado**: `certificates()` abre a Janela de Confirmação em modo **Escolher** ([§4.10](#410-modo-escolher-e-permissão-do-site))
  e devolve apenas o certificado escolhido.
- Site **lembrado**: devolve sem janela os certificados já usados nesse site (normalmente um).
- Nunca devolve a lista completa do computador.

**Por quê:** em clínica o computador é compartilhado; a lista completa entregaria nome e CPF de outros médicos
a qualquer site.

### R3. `fingerprint(digest)` no SDK

O SDK exporta `fingerprint(digest): { text: string; colorIndex: number; cells: boolean[] }` com o mesmo
algoritmo da [§4.4](#44-código-de-conferência), para o site mostrar o mesmo código e o mesmo desenho perto do
seu botão "Assinar". `TODO(gustavo)`: o Diagnos mostra o código ao lado de "Aguardando confirmação no WebeSign".

### R4. O site não põe texto na janela

O protocolo **não** tem campo de "motivo", "nome do documento" ou similar exibido na janela.
**Por quê:** texto do site dentro da nossa janela ganharia a nossa credibilidade (P1); o contexto do documento
fica na página do site, ligado pelo código de conferência.

### R5. A extensão informa origem, moldura e navegador

Cada pedido chega ao app com: origem do frame que chamou o SDK (do `MessageSender`, nunca do payload), origem
da aba de nível superior, nome e versão do navegador. Origens `http:` não locais, `file:`, `data:` e de
extensões são recusadas pela extensão com `InsecureOrigin` antes de chegar ao app.

### R6. Filtro opcional pedido pelo site

O pedido pode trazer `accept: { algorithms?: ("ECDSA"|"RSA-PKCS1"|"RSA-PSS")[] }`. Certificados incompatíveis
aparecem desabilitados com "Não compatível com este pedido" ([§5.8](#58-o-que-entra-na-lista)).
`TODO(gustavo)`: filtro por política (ex.: só ICP-Brasil) fica para depois?

### R7. Campos de devices.json que a interface consome

```jsonc
{
  "id": "safenet-etoken-5110",
  "name": "SafeNet eToken 5110",          // nome comercial exibido
  "kind": "token",                         // "token" | "card" | "reader"
  "match": { "usb": ["0529:0620"], "atr": [] },
  "driver": {
    "name": "SafeNet Authentication Client",
    "download": { "windows": "https://…", "macos": "https://…", "linux": "https://…" },
    "pkcs11": { "windows": "eTPKCS11.dll", "macos": "/usr/local/lib/libeTPkcs11.dylib", "linux": "/usr/lib/libeTPkcs11.so" }
  },
  "macos": { "cryptotokenkit": false },    // false → precisa do Complemento (ver §7)
  "pinUnlock": { "tool": "SafeNet Authentication Client" }   // onde desbloquear com o PUK
}
```

Campo ausente nunca bloqueia nada; só troca a dica específica pela genérica.

### R8. O app sabe mapear certificado → dispositivo

Para dizer "Token SafeNet eToken 5110" ou "Cartão no leitor Identiv", o app liga cada certificado ao
dispositivo: no Windows pelo `NCRYPT_READER_PROPERTY`/nome do leitor, no Mac pelo `kSecAttrTokenID`, no
PKCS#11 pela descrição do slot. Sem mapeamento seguro, o texto cai para "Token ou cartão".

---

## 4. Janela de Confirmação

### 4.1 Quando abre, tamanho e posição

| Item | Decisão | Por quê |
|------|---------|---------|
| Tamanho | **480 × 600** px lógicos, fixa, sem redimensionar | Cabe em 1366×768 a 100% (área útil ~728 px) com a barra de título; 480 dá largura para nome + selo na mesma linha. |
| Tela pequena | Se a área útil for menor que 640 px de altura, a janela usa `área útil − 40` de altura; cabeçalho e rodapé ficam fixos e o corpo rola | Nunca esconder os botões. |
| Posição | Centralizada no monitor onde está o ponteiro | É onde o usuário acabou de clicar "Assinar" no site; centralizar (e não abrir sob o ponteiro) evita que o botão nasça debaixo do cursor. |
| Primeiro plano | Sempre no topo enquanto espera decisão; opaca; sem minimizar nem maximizar; fechar (X) = Cancelar | Pedido de assinatura não pode ficar perdido atrás do navegador. |
| Título da janela (SO) | "Assinar para {site} — WebeSign" (`confirm.window_title`) | Leitor de tela anuncia o site ao focar; a barra de tarefas mostra quem pediu. |
| Nome do navegador | Nome curto: Chrome, Edge, Firefox, Brave, Safari, Chromium ("pelo Chrome") | Cabe na sobrancelha ao lado do chip; o nome completo fica no nome acessível. |
| Tema | Segue o sistema (claro/escuro) | Consistência com o SO. |
| Fonte da verdade do site | Origem enviada pela extensão ([R5](#r5-a-extensão-informa-origem-moldura-e-navegador)) | P1. |

`TODO(gustavo)`: provar na Fase 0 se o Windows entrega o foco para a janela quando o host já está rodando
(conexão mantida aberta). Se não entregar: janela no topo, `FlashWindowEx`, e o armamento do botão só começa
quando o usuário clicar na janela (o primeiro clique só foca, não aciona nada).

### 4.2 Anatomia (modo Assinar)

```
 480 px ──────────────────────────────────────────────────────────────
┌────────────────────────────────────────────────────────────────────┐ barra do SO: "Assinar para app.diagnos.health — WebeSign"
│ CABEÇALHO · bg-surface · padding 20/24/16 · borda inferior border   │
│  [signature 16] Pedido de assinatura · pelo Chrome   [✓ Site com permissão] │ text-caption fg-muted · chip de permissão à direita (22 px)
│  https://app.diagnos.health                                         │ text-headline: esquema+subdomínio fg-subtle 400, domínio fg 600
│  quer que você assine um documento.                                 │ text-body, fg-muted
│  (linha de alerta da origem, só quando houver — §4.3)               │ aviso compacto warning-soft, largura total
├────────────────────────────────────────────────────────────────────┤
│ CORPO · bg-canvas · padding 16/24 · gap 16 · rola se faltar espaço  │
│ ┌──────────────────────────────────────────────────────────────┐   │ cartão do código: bg-surface, border, radius-lg, padding 12/16
│ │ [identicon 40]  Código de conferência                SHA-256 │   │
│ │                 7F3A 9C21 E0B4 55D8                          │   │ text-code
│ │                 Confira se o site mostra o mesmo código.     │   │ text-small, fg-subtle
│ └──────────────────────────────────────────────────────────────┘   │
│  Assinar com                                                        │ text-caption, fg-muted (rótulo acessível da lista)
│ ┌──────────────────────────────────────────────────────────────┐   │ lista: bg-surface, border, radius-lg
│ │ (•) Ana Beatriz Souza                       [ICP-Brasil A3]   │   │ linha 72 px (ver §5.1)
│ │     CPF •••.456.789-•• · AC SOLUTI Multipla v5                │   │
│ │     [identification-card] Cartão no leitor · Vence em 23 dias │   │
│ ├──────────────────────────────────────────────────────────────┤   │
│ │ ( ) Ana Beatriz Souza                       [ICP-Brasil A1]   │   │
│ │ …                                                             │   │
│ ├──────────────────────────────────────────────────────────────┤   │
│ │ [caret-right] Não podem assinar (1)                           │   │ linha 40 px
│ └──────────────────────────────────────────────────────────────┘   │
│  (bloco de PIN aqui, só para chave via driver — §4.6)               │
├────────────────────────────────────────────────────────────────────┤
│ RODAPÉ · bg-surface · 64 px · padding 0/24 · borda superior         │
│  O Windows vai pedir o PIN.                  [Cancelar] [Assinar]   │ botões control-lg (36); Assinar min 112 px
└────────────────────────────────────────────────────────────────────┘
```

Regras de layout:

- Cabeçalho e rodapé são fixos; só o corpo rola. A lista tem altura máxima de **3 linhas** (216 px) e rola
  dentro de si; com o bloco de PIN visível, o máximo cai para **2 linhas** e a linha selecionada é mantida
  visível.
- **Ordem dos botões segue a plataforma.** Windows: `[Assinar] [Cancelar]` alinhados à direita. macOS e Linux:
  `[Cancelar] [Assinar]`. **Por quê:** o usuário clica por memória muscular do próprio sistema; inverter causa
  clique errado. Os mockups têm um seletor "Windows / macOS e Linux" que troca a ordem.
- O chip de permissão ("Site com permissão" / "Site novo") fica na linha da sobrancelha, à direita; alertas
  da origem ficam numa linha própria logo abaixo da frase, com largura total. **Por quê:** o alerta é mais
  importante que o status e não pode ser cortado; o status é curto e cabe na sobrancelha.
- Fila: a sobrancelha vira "Pedido de assinatura 1 de 3" (`confirm.eyebrow_queue`) e o título da janela
  ganha "(1 de 3)".
- O texto à esquerda do rodapé é a **dica de próximo passo** (ex.: "O Windows vai pedir o PIN.",
  `pin.os_prompt`) ou, nos últimos 30 s antes do tempo esgotar, "Este pedido expira em 28 s"
  (`footer.expires_in`).
- Tudo no mockup é reproduzível em egui: sem gradiente, sem desfoque, uma sombra por moldura.

### 4.3 Origem do site

A origem é o elemento mais importante da janela. Ela vem da extensão ([R5](#r5-a-extensão-informa-origem-moldura-e-navegador))
e é formatada assim:

1. **Separar** esquema, subdomínios, domínio registrável (eTLD+1 pela Public Suffix List, crate `psl`) e porta.
2. **Pintar**: `https://` e subdomínios em `fg-subtle` peso 400; domínio registrável em `fg` peso 600; porta
   (só se não for a padrão) em `fg-subtle`. Tudo em `text-headline` (20/26).
3. **Nunca cortar o domínio registrável.** Se o host não couber em 2 linhas, cortar subdomínios pela
   **esquerda** com "…" (`…secure.login.app.diagnos.health`). Hosts com mais de 32 caracteres usam
   `text-title` (16/22). **Por quê:** o golpe clássico é `diagnos.health.cadastro-medico.com`; o que
   importa (`cadastro-medico.com`) fica no fim.
4. **IDN**: se qualquer rótulo não for ASCII, mostrar a forma **punycode** (`xn--…`) como principal e a forma
   Unicode abaixo em `fg-subtle` ("Aparece como diаgnos.health", `origin.shown_as`), com alerta.
   **Por quê:** domínios com acento são raros no nosso público; ser estrito custa pouco e mata homógrafos.
5. **Moldura**: se o pedido vem de um iframe cuja origem difere da aba, acrescentar uma linha de alerta
   "Dentro da página de {top_site}" (`confirm.inside_frame`).

Variantes e alertas (o chip de status fica na sobrancelha; o alerta, numa linha própria abaixo de "quer que você assine…"):

| Caso | Exemplo | Tratamento | Chave |
|------|---------|------------|-------|
| https normal, site lembrado | `https://app.diagnos.health` | chip success `check-circle` "Site com permissão" | `confirm.site_remembered` |
| https normal, site novo | `https://app.diagnos.health` | chip neutro `info` "Site novo" (nome acessível: "Primeira vez que este site pede algo neste computador") + caixa "Lembrar este site" | `confirm.site_new` |
| Subdomínio enganoso | `https://diagnos.health.cadastro-medico.com` | sem alerta extra; o destaque do domínio faz o trabalho; se for novo, chip "Primeira vez" | — |
| IDN / punycode | `https://xn--dignos-4nf.health` | alerta warning `warning` "Endereço com caracteres especiais. Confira letra por letra." | `origin.warn_idn` |
| IP público | `https://203.0.113.7` | alerta warning "Endereço numérico, sem nome de site" | `origin.warn_ip` |
| IP de rede local | `https://192.168.0.20:8443` | alerta warning "Endereço numérico da rede local" | `origin.warn_ip_local` |
| Site local | `http://localhost:5173`, `127.0.0.1`, `[::1]` | alerta warning `terminal-window` "Site local de desenvolvimento" | `origin.warn_localhost` |
| http não local | `http://laudos.exemplo.com` | **bloqueado** pela extensão (`InsecureOrigin`); se chegar ao app por bug, estado de erro sem botão Assinar | `origin.blocked_http` |

**Por quê bloquear http:** mesma regra do WebAuthn (só contexto seguro); qualquer um na rede poderia injetar
um pedido. Sistemas hospitalares em IP de intranet continuam funcionando se usarem https.

Alertas warning **não** bloqueiam e não mudam o botão; eles são lidos pelo leitor de tela junto com a origem.
Com qualquer alerta, a caixa "Lembrar este site" fica **desmarcada e desabilitada** para IP e punycode
(o usuário ainda pode assinar, mas não pode lembrar). **Por quê:** endereços difíceis de conferir não devem
ganhar acesso silencioso.

### 4.4 Código de conferência

**Decisão:** mostrar os **8 primeiros bytes do digest** em hexadecimal maiúsculo, em 4 grupos de 4
(`7F3A 9C21 E0B4 55D8`, fonte mono), acompanhados de um **identicon 5×5 espelhado** derivado dos mesmos bytes.
Nada de emoji nem lista de palavras.

| Opção avaliada | Resultado | Motivo |
|----------------|-----------|--------|
| Hex completo (64 caracteres) | ✗ | Ninguém confere 64 caracteres. |
| Emoji-hash | ✗ | O emoji do egui (fonte embutida) e o do navegador (fonte do SO) têm desenhos diferentes; o usuário compararia desenhos que não batem. |
| Lista de palavras | ✗ | Depende do idioma; o site e o app poderiam estar em idiomas diferentes; lista por idioma pesa no SDK. |
| **Hex curto em grupos + identicon** | ✓ | O identicon permite conferir "de relance" (forma + cor); o hex permite conferir com exatidão; ambos se reproduzem no SDK com poucas linhas e sem fonte especial. |

Algoritmo (idêntico no Rust e no SDK, `R3`):

```text
b          = digest[0..8]
text       = hex_upper(b) em grupos de 4, separados por espaço     → "7F3A 9C21 E0B4 55D8"
colorIndex = b[0] >> 5                                             → 0..7 (paleta identicon, §11.1)
bits       = (b[1] << 8) | b[2]                                    → usa os 15 bits de baixo
célula(linha r ∈ 0..4, coluna c ∈ 0..2) acesa ⇔ bit (r*3 + c) de bits = 1
coluna 3 = coluna 1; coluna 4 = coluna 0                           (espelho)
```

Desenho: quadro 40×40 com fundo `bg-sunken`, borda 1 px `border`, `radius-sm`; células de 6 px com 1 px de
respiro, cor da paleta identicon. As 8 cores têm contraste ≥ 3:1 contra `bg-surface` nos dois temas, então o
mesmo desenho vale no claro e no escuro (e no site).

Texto de apoio: "Confira se o site mostra o mesmo código." (`code.help`). Sem certificado selecionado (lista
vazia ou só com desabilitados) não há digest e o cartão do código **não aparece**; ele entra com
`motion-base` quando um certificado é selecionado. **Por quê:** um cartão vazio ocupa o espaço de que o estado
vazio precisa para a dica acionável. Selo `SHA-256`/`SHA-384`/`SHA-512`
em `text-caption` à direita do rótulo. Enquanto o site prepara o digest ([R1](#r1-uma-janela-por-assinatura-com-o-certificado-escolhido-nela)):
esqueleto do código + "Preparando o documento…" (`code.preparing`), mostrado só se demorar mais de 150 ms.

Acessibilidade: o identicon é decorativo (oculto do AccessKit); o código é texto selecionável e é lido
caractere a caractere ("7 F 3 A, 9 C 2 1, …", `code.a11y`).

### 4.5 Seção de certificado

Toda a especificação da lista está na [§5](#5-lista-de-certificados). Na janela de confirmação:

- Rótulo "Assinar com" (`certs.label_sign`); em modo Escolher, "Escolha o certificado" (`certs.label_select`).
- **Um certificado**: a linha aparece sem o círculo de seleção (o único já está escolhido); o resto igual.
- **Vários**: grupo de rádio; ↑/↓ troca a seleção.
- **Nenhum**: estado vazio + [Possíveis certificados](#6-possíveis-certificados).
- Trocar de certificado **re-arma** o botão Assinar (600 ms) e pede novo digest ao site (R1).

### 4.6 PIN

| Origem da chave | Quem pede o PIN | O que a nossa janela mostra |
|-----------------|-----------------|-----------------------------|
| Windows (CNG/CAPI) ou Mac (Keychain/CryptoTokenKit) | A janela do sistema ou do middleware | Dica no rodapé: "O Windows vai pedir o PIN numa janela própria." (`pin.os_prompt`). Depois do clique: "Digite o PIN na janela do Windows." (`pin.os_prompt_now`). Nossa janela deixa de ser "sempre no topo" enquanto o sistema pede o PIN (a janela do PIN é filha dela via `NCRYPT_WINDOW_HANDLE_PROPERTY`). |
| Driver do token (PKCS#11), teclado comum | Nosso campo de PIN | Bloco de PIN (abaixo). |
| Driver do token com teclado no leitor (`CKF_PROTECTED_AUTHENTICATION_PATH`) | O teclado do leitor | Sem campo. Antes do clique: "Depois de clicar em Assinar, digite o PIN no teclado do leitor." (`pin.pinpad_before`). Depois: cartão com `dots-nine` e "Digite o PIN no teclado do leitor." (`pin.pinpad_now`). |
| Driver do token já autenticado nesta sessão (e chave sem `CKA_ALWAYS_AUTHENTICATE`) | Ninguém | Chip `lock-key-open` "Token desbloqueado nesta sessão" (`pin.unlocked_session`). |

Bloco de PIN (driver do token):

```
 PIN do token                                         4 a 16 caracteres     ← text-caption fg-muted / text-small fg-subtle
┌────────────────────────────────────────────────────────────────[eye]┐     ← campo control-md 32, bg-sunken, borda border-strong
│ ••••••                                                              │
└─────────────────────────────────────────────────────────────────────┘
 [shield-check] O PIN fica neste computador e não passa pelo navegador.     ← text-small fg-subtle (some quando há erro)
```

- Rótulo: "PIN do token" ou "PIN do cartão" (pelo `kind` do dispositivo); limites vindos de
  `CK_TOKEN_INFO.ulMinPinLen/ulMaxPinLen` ("4 a 16 caracteres", `pin.length_hint`). Assinar só habilita com o
  tamanho dentro do limite.
- Botão `eye`/`eye-slash` para mostrar o PIN (`pin.show`/`pin.hide`), desligado por padrão.
  **Por quê:** PIN errado bloqueia o token e custa uma ida à AC; ver o que digitou reduz bloqueios.
- Enter no campo = Assinar (se armado e com tamanho válido).
- Segurança: buffer zerado logo após `C_Login` (crate `zeroize`); no Mac, `EnableSecureEventInput` enquanto o
  campo tem foco; o valor nunca vai para o AccessKit (papel `PasswordInput`, valor oculto); sem copiar/colar
  para fora do campo.
- **PIN incorreto** (`CKR_PIN_INCORRECT`): campo esvaziado e focado, borda `danger`, mensagem abaixo com
  `x-circle`. O PKCS#11 não informa o número exato de tentativas; usamos as flags do token:

  | Flag após o erro | Mensagem | Chave |
  |------------------|----------|-------|
  | nenhuma | "PIN incorreto." | `pin.incorrect` |
  | `CKF_USER_PIN_COUNT_LOW` | "PIN incorreto. Restam poucas tentativas antes de o token bloquear." | `pin.incorrect_low` |
  | `CKF_USER_PIN_FINAL_TRY` | "PIN incorreto. Última tentativa: se errar de novo, o token bloqueia." (texto em `danger`, peso 600) | `pin.incorrect_final` |

  **Por quê:** prometer "3 tentativas restantes" seria inventar um número que o padrão não dá.
- **PIN bloqueado** (`CKR_PIN_LOCKED` ou `CKF_USER_PIN_LOCKED`): o campo some; aparece um aviso `danger` com
  `lock-key` (fill): título "PIN bloqueado" e texto "O token bloqueou depois de muitas tentativas erradas.
  Desbloqueie com o PUK no SafeNet Authentication Client ou procure a AC SOLUTI." (`pin.locked_body`, usando
  `pinUnlock.tool` do devices.json e o emissor). A linha do certificado ganha o motivo "PIN bloqueado" e fica
  desabilitada; Assinar desabilita. Se o usuário cancelar nesse estado, o SDK recebe `PinLocked`.
- Chave do sistema com PIN errado: o sistema mostra o próprio erro; se voltar
  `SCARD_W_WRONG_CHV`, mostramos "PIN incorreto." no rodapé e o usuário clica Assinar de novo; se voltar
  `SCARD_W_CHV_BLOCKED`, estado PIN bloqueado.
- Não há aviso de Caps Lock: o egui/winit não expõe o estado da tecla de forma confiável.

### 4.7 Botões, armamento e prevenção de clique acidental

| Regra | Valor | Por quê |
|-------|-------|---------|
| Armamento do botão Assinar | **600 ms** contados a partir do momento em que a janela está visível **e** focada | Maior que o intervalo de duplo clique padrão (500 ms no Windows e no macOS): o segundo clique do "Assinar" do site nunca cai no nosso botão. |
| Re-armar | Sempre que a janela perde e recupera o foco, o certificado muda, o digest muda ou um pedido da fila assume | Conteúdo novo sob o cursor exige nova leitura. |
| Entrada durante o armamento | Teclas e cliques são **descartados** (exceto Esc) | Tecla Enter que o usuário estava digitando no site não vira assinatura. |
| Clique válido | Pressionar **e** soltar dentro do botão, com o pressionar depois do armamento | Evita "pressiona antes, solta depois". |
| Visual do armamento | Botão desabilitado que transiciona para habilitado em `motion-base` (180 ms) ao armar; sem barra de progresso | Informa sem chamar atenção. |
| Duplo clique numa linha | Só seleciona | Nenhum gesto de lista assina. |
| Janela | Opaca, no topo, posição definida pelo app | O site não controla nada da janela. |

Estados do botão primário: `Assinar` (desabilitado → armado) · `Assinando…` com `Spinner` do egui ·
em erro recuperável, `Tentar de novo`. Em modo Escolher: `Usar este certificado` (`action.use_cert`).

Cancelar fica habilitado o tempo todo, exceto durante a assinatura via sistema ou teclado do leitor (nenhuma
das APIs permite abortar); nesse caso mostra `Aguarde…`.

### 4.8 Máquina de estados

| Estado | O que mostra | Saídas |
|--------|--------------|--------|
| `loading_certs` | Esqueleto de 2 linhas após 150 ms; "Procurando certificados…"; após 2 s, "Ainda lendo {device}. Drivers de token podem levar alguns segundos." | → `choosing`, `empty` |
| `empty` | Sem cartão do código; estado vazio + possíveis certificados ([§6](#6-possíveis-certificados)); "Abrir diagnóstico" à esquerda do rodapé; Assinar desabilitado; foco em Cancelar | → `choosing` (token inserido), Cancelar → `NoCertificates` |
| `choosing` | Lista; digest pendente mostra "Preparando o documento…" | → `ready` (digest chegou), Cancelar |
| `ready` | Código visível; Assinar arma em 600 ms | → `signing`, Cancelar → `UserCancelled` |
| `pin_error` | Bloco de PIN com mensagem (§4.6) | → `signing`, `pin_locked`, Cancelar |
| `pin_locked` | Aviso de bloqueio; linha desabilitada | Escolher outro → `ready`; Cancelar → `PinLocked` |
| `signing` | Botão "Assinando…"; dica do sistema/teclado do leitor; lista e PIN desabilitados | → `success`, `pin_error`, `error` |
| `success` | Corpo troca por `check-circle` 48 px (success), "Assinado", "A assinatura foi enviada para {site}."; resultado já foi enviado ao site | Fecha sozinha após **900 ms** |
| `error` | Aviso `danger` acima da lista com título, texto, ação e "Detalhes técnicos" recolhido (código copiável) | Ação da [§15](#15-erros); Cancelar → código do erro |
| `site_cancelled` | "{site} cancelou o pedido." (a aba fechou ou navegou) | Fecha após 1,5 s |
| `timeout` | Após **5 min** sem decisão | Fecha; SDK recebe `Timeout` |
| `blocked_origin` | Só por bug (R5): erro `InsecureOrigin`, sem Assinar | Fechar |

**Por quê fechar no sucesso:** o próximo passo do usuário está no site; 900 ms bastam para ver que deu certo.
O resultado é enviado antes da animação, então o site não espera por ela.

Mudanças ao vivo (eventos PC/SC):

- **Token inserido**: nova linha entra no fim do grupo utilizável com animação de altura (`motion-base`);
  a linha selecionada **não se move**; o botão re-arma. Leitor de tela anuncia "Certificado encontrado: {nome}"
  (região viva educada).
- **Token removido** com a linha selecionada: a linha fica no lugar, desabilitada, com "Removido. Conecte o
  token de novo." (`cert.reason.removed`); Assinar desabilita. Ao voltar, a linha volta selecionada.
- **Token removido durante `signing`**: erro `TokenRemoved`.

### 4.9 Teclado e foco

| Tecla | Efeito |
|-------|--------|
| Esc | Cancelar, sempre (inclusive durante o armamento). |
| Tab / Shift+Tab | Lista → "Detalhes" da linha selecionada → PIN → mostrar PIN → "Lembrar este site" → Cancelar → Assinar (na ordem visual da plataforma). |
| ↑ / ↓, Home / End | Movem a seleção na lista, pulando linhas desabilitadas (que continuam focáveis para leitura). |
| Enter numa linha | Não assina. Move o foco para o PIN (se a chave pede PIN nosso) ou para o botão Assinar. |
| Enter no campo de PIN | Assina (armado + tamanho válido). |
| Enter / Espaço no botão | Aciona o botão focado. |
| Ctrl/⌘+C no código | Copia o código (texto selecionável). |

Foco inicial: (1) campo de PIN, se o certificado pré-selecionado pede PIN nosso; (2) senão, a linha
selecionada; (3) sem certificados, Cancelar. **Nunca** no botão Assinar.
**Por quê:** a janela aparece enquanto o usuário pode estar digitando; foco em Assinar transformaria um Enter
perdido em assinatura.

### 4.10 Modo Escolher e permissão do site

Um site **novo** (sem permissão lembrada) sempre vê a caixa:

```
[ ] Lembrar este site neste computador                          ← checkbox, control-sm
    Ele poderá saber qual certificado você usa sem perguntar.    ← text-small fg-subtle
    Cada assinatura continua pedindo sua confirmação.
```

- **Desmarcada por padrão.** **Por quê:** lembrar dá um poder novo ao site; o padrão seguro é não conceder.
- Aparece no modo Assinar (abaixo da lista) e no modo Escolher.
- Desabilitada para IP e punycode (§4.3).
- O que "lembrar" concede: `certificates()` sem janela para os certificados já usados no site, e
  pré-seleção do último certificado usado no site. Assinar **sempre** pede confirmação.
- Revogar: Diagnóstico › Navegadores › "Sites com permissão" ([§8.3](#83-aba-navegadores)).

**Modo Escolher** (`certificates()` de site não lembrado, R2): mesma janela, com estas diferenças:

- Título da janela: "Escolher certificado para {site} — WebeSign"; sobrancelha "Pedido de certificado";
  frase "quer saber com qual certificado você vai assinar." (`confirm.asks_select`).
- No lugar do cartão do código: aviso neutro `info` "O site vai receber nome, tipo, emissor e validade do
  certificado escolhido. Nada é assinado agora." (`consent.select_shares`).
- Não há PIN.
- Botão primário "Usar este certificado"; mesmo armamento de 600 ms.
- Sucesso: "Certificado enviado para {site}" por 900 ms e fecha.

### 4.11 Fila, tempo esgotado e site que desistiu

- **Fila**: pedidos que chegam com a janela aberta entram numa fila (máx. 10; acima disso, `Busy`). A
  sobrancelha mostra "Pedido de assinatura 1 de 3" (`confirm.eyebrow_queue`). Ao concluir o atual, o próximo assume com transição de
  `motion-base` e re-armamento. Pedidos de sites diferentes nunca se misturam.
- **Tempo esgotado**: 5 min sem decisão → fecha com `Timeout`. Contagem regressiva no rodapé nos últimos 30 s.
- **Site desistiu**: a porta da extensão fecha (aba fechada ou navegação) → estado `site_cancelled`.

`TODO(gustavo)`: assinatura em lote (vários laudos, uma confirmação com N códigos) conflita com "confirmação a
cada assinatura"; decidir se entra e com qual limite.

---

## 5. Lista de certificados

É o coração da UX: o médico precisa reconhecer o **seu** certificado em um segundo, mesmo num computador
compartilhado.

### 5.1 Anatomia da linha (72 px)

```
┌──────────────────────────────────────────────────────────────────────┐
│ (•)  Ana Beatriz Souza                              [ICP-Brasil A3]    │  linha 1: text-body-strong fg · selo à direita
│      CPF •••.456.789-•• · AC SOLUTI Multipla v5                        │  linha 2: text-small fg-muted
│      [identification-card] Cartão no leitor · Vence em 23 dias  Detalhes│  linha 3: text-small; validade colorida; "Detalhes" só na selecionada
└──────────────────────────────────────────────────────────────────────┘
 padding 10/16 · linhas 20 + 16 + 16 sem espaço extra (= 72) · rádio 16 px · gap 12 · linha inteira é o alvo do clique
```

- Selecionada: fundo `accent-soft`, rádio preenchido `accent` com ponto `on-accent`. Sem barra lateral colorida.
- Hover: `bg-hover`. Foco de teclado: anel `focus` de 2 px por dentro da linha.
- Desabilitada: textos em `fg-subtle`, sem rádio, motivo na linha 3 no lugar da validade (ex.: `x-circle`
  "Venceu em 10/05/2026" em `danger`).
- Texto que não cabe é truncado com "…" no fim (nome e emissor); o texto completo fica na dica ao passar o
  mouse e no nome acessível.
- Na linha 3, **a validade nunca é cortada**: quem encolhe é o local ("Token SafeNet eToken 5110 · pelo
  dri…"). **Por quê:** a validade é estado e decide a escolha; o local é identificação secundária.

### 5.2 Nome do titular

1. Pegar o `CN` do sujeito. Se não houver, `givenName + surname`; se não houver, `O`.
2. ICP-Brasil: remover o sufixo `:{11 ou 14 dígitos}` (`ANA BEATRIZ SOUZA:12345678909` → `ANA BEATRIZ SOUZA`).
3. Se o nome estiver todo em maiúsculas, converter para "Título": partículas `da, das, de, di, do, dos, du, e,
   del, la, van, von, y` em minúsculas (exceto no início); sufixos societários `LTDA → Ltda`, `S.A.`, `ME`,
   `EPP`, `EIRELI` preservados como grafia oficial. Senão, manter como está.
4. Nunca inventar título profissional: o certificado não tem "Dra."; a linha mostra "Ana Beatriz Souza".

### 5.3 Tipo (selo)

Um único selo por linha, neutro (`bg-sunken`, `fg-muted`, `radius-sm`, `text-caption`). **Por quê neutro:** cor
fica reservada para estado (validade, erro); tipo não é estado.

Primeira regra que casar:

| # | Condição | Selo (pt-BR / en) |
|---|----------|-------------------|
| 1 | Emissor do Cartão de Cidadão (issuer `CN` contém "Cartão de Cidadão") | "Cartão de Cidadão" / "Cartão de Cidadão" |
| 2 | Emissor do DNIe (issuer `O` = "DIRECCION GENERAL DE LA POLICIA") | "DNIe" / "DNIe" |
| 3 | Política ICP-Brasil `2.16.76.1.2.{1,2,3,4}.*` | "ICP-Brasil A1…A4" |
| 4 | Política ICP-Brasil `2.16.76.1.2.{101..104}.*` | "ICP-Brasil S1…S4" |
| 5 | Sujeito com `O=ICP-Brasil` sem política reconhecida | "ICP-Brasil" |
| 6 | `qcStatements` com `QcCompliance` (0.4.0.1862.1.1) **e** `QcSSCD` (0.4.0.1862.1.4) | "Qualificado eIDAS" / "Qualified eIDAS" |
| 7 | `QcCompliance` sem `QcSSCD` | "eIDAS" / "eIDAS" |
| 8 | Qualquer outro | "Certificado" / "Certificate" |

`TODO(gustavo)`: conferir a tabela de OIDs contra o DOC-ICP-04 vigente antes de liberar.
A tabela de emissores nacionais (linhas 1–2) vive no app como dado (`cert_kinds.rs`), não no devices.json,
porque descreve certificados e não dispositivos.

### 5.4 Emissor curto

`CN` do emissor; na falta, `O`. Sem abreviações inventadas ("AC SOLUTI Multipla v5", "AC Certisign RFB G5").
Truncado no fim.

### 5.5 Documento (CPF/CNPJ)

| Fonte | Exibição | Por quê |
|-------|----------|---------|
| ICP-Brasil PF: `otherName 2.16.76.1.3.1` (data de nascimento 8 dígitos + CPF 11 dígitos + …) | `CPF •••.456.789-••` | Máscara padrão do governo (mostra do 4º ao 9º dígito): distingue pessoas sem expor o CPF inteiro. |
| ICP-Brasil PJ: `otherName 2.16.76.1.3.3` (CNPJ 14 dígitos) | `CNPJ 12.345.678/0001-90` completo | CNPJ é dado público da Receita. |
| `serialNumber` do sujeito com prefixo ETSI (`PNOPT-`, `IDCPT-`, `IDCES-`, …) | `Documento •••••123` (3 últimos) | Documento nacional é dado pessoal. |
| Nenhum | Linha 2 só com o emissor | — |

Leitor de tela: "CPF parcialmente oculto, 456 789" (`cert.doc_cpf_a11y`). O CPF completo nunca aparece em
lugar nenhum da interface, dos logs ou do diagnóstico.

### 5.6 Validade

| Situação | Texto | Cor | Ícone |
|----------|-------|-----|-------|
| Mais de 30 dias | "Válido até 14/03/2027" | `fg-muted` | — |
| 8 a 30 dias | "Vence em 23 dias" | `warning` | `clock` |
| 2 a 7 dias | "Vence em 5 dias" | `danger` | `clock` |
| Amanhã / hoje | "Vence amanhã" / "Vence hoje" | `danger` | `clock` |
| Vencido | "Venceu em 10/05/2026" (linha desabilitada) | `danger` | `x-circle` |
| Ainda não vale | "Válido a partir de 01/10/2026" (linha desabilitada) | `warning` | `clock` |

Datas no fuso local; formato por idioma (`dd/mm/aaaa` em pt; `14 Mar 2027` em en).
**Por quê 30 dias:** é o prazo prático para renovar um A3 (agendar validação na AC).

### 5.7 Onde está o certificado (origem em linguagem leiga)

| Situação técnica | Texto | Ícone |
|------------------|-------|-------|
| Chave de software no repositório do sistema (A1 importado) | "Neste computador" | `desktop` |
| Chave em hardware, dispositivo conhecido (devices.json, `kind: token`) | "Token SafeNet eToken 5110" | `usb` |
| Chave em cartão num leitor | "Cartão no leitor" (+ nome do leitor em Detalhes) | `identification-card` |
| Chave em hardware sem mapeamento seguro | "Token ou cartão" | `usb` |
| Qualquer um dos acima, acessado pelo driver (PKCS#11) | acrescenta " · pelo driver" | — |

**Por quê o "· pelo driver":** explica por que o campo de PIN aparece na nossa janela (e não a do sistema);
fora isso, o caminho técnico fica em Detalhes.

### 5.8 O que entra na lista

**Ocultos** (nunca na lista; contados em Diagnóstico › Certificados):

1. Sem chave privada associada.
2. `basicConstraints cA=true`.
3. `keyUsage` presente sem `digitalSignature` nem `nonRepudiation`.
4. `extendedKeyUsage` presente e só com propósitos alheios (`serverAuth`, `codeSigning`, `timeStamping`,
   `OCSPSigning`). Passam: ausência de EKU, `anyExtendedKeyUsage`, `clientAuth`, `emailProtection`,
   `documentSigning` (1.3.6.1.5.5.7.3.36), MS Document Signing (1.3.6.1.4.1.311.10.3.12).
5. Certificado de autenticação irmão: mesmo titular **e** mesmo dispositivo que um certificado com
   `nonRepudiation`, tendo ele próprio só `digitalSignature` (é o caso do Cartão de Cidadão e do DNIe, que
   trazem um certificado de login e um de assinatura). **Por quê:** evita que o médico escolha o de login e
   gere uma assinatura não qualificada.

**Desabilitados** (visíveis no grupo "Não podem assinar", com motivo): vencido, ainda não vale, PIN bloqueado,
incompatível com o algoritmo pedido pelo site (R6), removido durante a janela.
**Por quê mostrar o vencido:** "meu certificado sumiu" é o chamado de suporte mais comum, e quase sempre é
vencimento; mostrar com o motivo resolve sem suporte.

### 5.9 Ordenação e seleção inicial

1. Último certificado usado **neste site** (só para site lembrado).
2. Demais utilizáveis por uso mais recente em qualquer site.
3. Nunca usados: A3/qualificado antes de A1/software; depois nome (A→Z); depois validade mais longa.
4. Grupo "Não podem assinar ({n})" no fim, recolhido.

Seleção inicial: item 1; senão o primeiro utilizável; senão nenhum. Enquanto a janela está aberta, novas
linhas entram no fim e **nada reordena** (anti clique errado).

### 5.10 Agrupamento

Na Confirmação **não** há grupos por origem técnica; só o grupo recolhido "Não podem assinar".
**Por quê:** o médico pensa em "meu certificado", não em "repositório do Windows vs driver".
No Diagnóstico › Certificados há grupos por origem (é lá que TI olha).

### 5.11 Deduplicação

Chave: SHA-256 do DER do certificado. O mesmo certificado visto pelo sistema e pelo driver vira **uma**
linha, usando o caminho do sistema (decisão 2 do brief). Detalhes mostra "Também acessível pelo driver do
token". Se a assinatura pelo sistema falhar com erro de driver (não PIN, não cancelamento), o erro oferece
"Tentar pelo driver do token" (`errors.driver_failure.alt_path`).

### 5.12 Detalhes do certificado

O link "Detalhes" (só na linha selecionada) expande a linha em `motion-base` com pares rótulo/valor em
`text-small`:

| Rótulo | Valor |
|--------|-------|
| Titular | CN completo como está no certificado |
| Emissor | DN do emissor (CN, O, C) |
| Validade | "03/03/2025 a 22/10/2026" |
| Uso | "Assinatura digital, Não repúdio" |
| Chave | "RSA 2048" / "ECDSA P-256" |
| Impressão digital (SHA-256) | 16 grupos de 4 hex em `text-mono`, com botão `copy` |
| Acesso | "Windows (repositório de certificados)" / "Keychain do macOS" / "Driver do token: C:\Windows\System32\eTPKCS11.dll" |
| (se dedup) | "Também acessível pelo driver do token" |

Botão "Ver no sistema" (`cert.details.view_in_system`): abre o visualizador do SO
(`CryptUIDlgViewContext` no Windows, `SFCertificatePanel` no Mac, `gcr-viewer` no Linux se existir).

### 5.13 Muitos certificados

Com mais de 6 utilizáveis, aparece acima da lista um campo "Filtrar por nome, CPF ou emissor"
(`certs.filter_placeholder`, ícone `magnifying-glass`). Filtra por nome, emissor e dígitos visíveis do CPF.
Sem resultado: "Nenhum certificado com “{query}”".

---

## 6. Possíveis certificados

Dispositivo detectado que parece token ou cartão, mas que não trouxe certificado para a lista.

### 6.1 O que conta como "possível certificado"

| Fonte | Entra se | Sai se |
|-------|----------|--------|
| USB (`nusb`) | VID:PID está no devices.json **ou** tem interface de classe `0x0B` (CCID) | algum certificado listado foi mapeado a ele (R8) |
| Leitor PC/SC | há cartão presente (ATR lido) | idem |

Dispositivos USB comuns (mouse, teclado) nunca aparecem. Quando o mapeamento certificado → dispositivo for
incerto, a dica **não** aparece na Confirmação (evita alarme falso) e aparece no Diagnóstico como
"Não sabemos se tem certificados".

### 6.2 Na Confirmação

**Lista vazia** — o estado vazio vira um cartão acionável (bg-surface, border, radius-lg, padding 16):

```
 [usb em círculo warning-soft 32]  SafeNet eToken 5110 conectado
                                   Nenhum certificado apareceu neste token. Para usá-lo,
                                   instale o SafeNet Authentication Client.
                                   [download-simple Baixar para Windows ↗]  [arrow-clockwise Já instalei, procurar de novo]
```

- Botão primário do cartão abre o link do SO atual no navegador padrão; secundário relê drivers e
  repositório (`common.refresh`).
- Sem link para o SO: "Procure o programa no site da autoridade certificadora que emitiu o seu certificado."
  (`possible.no_link`).
- Cartão desconhecido num leitor: "Há um cartão no leitor Identiv uTrust 2700 R, mas não o reconhecemos. Se ele
  tem certificado, instale o programa do fabricante do cartão." + [Abrir diagnóstico].
- No Mac, se o dispositivo precisa do Complemento ([§7](#7-complemento-macos)), o cartão vira dois passos:
  "1. Instale o SafeNet Authentication Client" e "2. Instale o Complemento WebeSign".
- Sem nenhum dispositivo detectado: só o estado vazio "Nenhum certificado encontrado / Conecte o token ou
  insira o cartão no leitor. A lista atualiza sozinha." + link "Abrir diagnóstico".

**Lista com certificados** — uma linha compacta de 40 px no fim da lista:
`[usb] SafeNet eToken 5110 conectado sem certificados · Como resolver` → expande o mesmo cartão dentro da lista.
**Por quê:** o caso real é "renovei e ganhei token novo, mas o driver não está instalado" enquanto o A1 antigo
continua aparecendo.

### 6.3 No Diagnóstico

Aba Dispositivos, na linha do dispositivo, com o mesmo texto e botão ([§8.4](#84-aba-dispositivos)).

---

## 7. Complemento (macOS)

Só existe se a prova 3 mostrar que a sandbox da App Store barra o PKCS#11. Tudo abaixo vale apenas para o
build da Mac App Store.

**Nome na tela:** "Complemento WebeSign" (en: "WebeSign Add-on"). Ícone `puzzle-piece`.
Nunca usar "sandbox", "Mach service", "helper", "segundo plano" ou "daemon" na interface.

Quando sugerir (qualquer um):

1. Dispositivo detectado cujo devices.json diz `macos.cryptotokenkit: false`.
2. Usuário adicionou um driver manualmente no Diagnóstico.
3. Dispositivo conhecido conectado, sem certificados, com o driver do fabricante instalado (arquivo `pkcs11.macos` existe).

Texto base (`complement.body_needed`): "Apps da App Store não podem usar alguns drivers de token. O Complemento
WebeSign, gratuito e do mesmo projeto, faz essa ponte. Ele só funciona quando o WebeSign pede."
**Por quê esse texto:** explica a causa sem culpar ninguém, diz que é do mesmo projeto (confiança) e responde
de antemão o medo de "programa rodando escondido".

| Estado | Onde | Visual | Ação |
|--------|------|--------|------|
| Ausente e necessário | Confirmação (cartão de possível certificado, passo 2) e Diagnóstico › Dispositivos (cartão no topo) | aviso `warning` | [Baixar Complemento] → página do site com o `.dmg` |
| Instalado e atual | Diagnóstico › Dispositivos | linha `success` "Complemento ativo · versão 1.2.0" | — |
| Desatualizado | Diagnóstico › Dispositivos; Confirmação só se o erro vier dele | aviso `warning` "Atualize o Complemento (1.1.0 instalado, precisa da 1.2.0)" | [Atualizar Complemento] → abre o Complemento, que dispara a checagem do Sparkle |
| Instalado, sem resposta | Diagnóstico › Dispositivos | aviso `warning` "O Complemento está instalado, mas não respondeu." | [Abrir Complemento] |
| Não necessário | — | nada é mostrado | — |

---

## 8. Janela de Diagnóstico

### 8.1 Estrutura

| Item | Decisão | Por quê |
|------|---------|---------|
| Tamanho | **760 × 540** px lógicos, fixa; minimizar permitido, maximizar não | Cabe em 1366×768 a 100%; largura para barra lateral + conteúdo de 560. |
| Abertura | Ícone do app no menu Iniciar/Launchpad/menu de apps; "Abrir diagnóstico" no popup e nos erros | Porta de entrada única para "algo não funciona". |
| Aba inicial | A primeira com semáforo vermelho; senão amarelo; senão Navegadores. Vindo de "nenhum certificado", Dispositivos | Abre onde está o problema. |
| Layout | Barra lateral 200 px (bg-surface, borda direita) + conteúdo (bg-canvas, padding 24, rola) | 4 abas com status ficam legíveis na vertical. |

Barra lateral, de cima para baixo:

1. Marca (`seal-check` fill em quadrado `accent` 24 px, `radius-md`) + "WebeSign" (`text-title`) + versão (`text-small`, `fg-subtle`).
2. **Semáforo geral** (chip): verde "Pronto para assinar", amarelo "Precisa de atenção", vermelho "Ainda não dá para assinar".
3. Abas: ícone + rótulo + ícone de status à direita. Altura 36, `radius-md`, selecionada em `accent-soft` com texto `accent-fg`.
4. Rodapé: botão secundário largo [copy Copiar diagnóstico].

Cabeçalho do conteúdo: título da aba (`text-headline`) + subtítulo (`text-body`, `fg-muted`) + ação à direita
[arrow-clockwise Procurar de novo] (F5 / Ctrl+R).

**Semáforo** — cor + forma + texto, nunca só cor:

| Nível | Ícone (fill) | Cor | Significa |
|-------|--------------|-----|-----------|
| Verde | `check-circle` | `success` | Funciona. |
| Amarelo | `warning` | `warning` | Algo a resolver, mas dá para assinar (ou é só aviso). |
| Vermelho | `x-circle` | `danger` | Impede assinar. |
| Cinza | `circle-dashed` (regular) | `fg-subtle` | Não se aplica / sem dados. |

Regras por aba:

| Aba | Verde | Amarelo | Vermelho |
|-----|-------|---------|----------|
| Navegadores | ≥ 1 navegador com extensão conectada e todos os instalados sem problema | ≥ 1 conectado, mas outro sem extensão/desatualizado | Nenhum navegador conectado, ou registro do app ausente em todos |
| Dispositivos | Todo dispositivo detectado trouxe certificado (ou nenhum detectado) | Dispositivo sem certificado com dica; driver adicionado que não carrega; Complemento desatualizado | `pcscd` parado no Linux |
| Certificados | ≥ 1 utilizável e nenhum utilizável vence em ≤ 30 dias | Algum utilizável vence em ≤ 30 dias | Nenhum utilizável |
| Ajuda | sem semáforo | — | — |

Geral = pior entre as abas.

### 8.2 Primeiros passos

Enquanto não estiverem todos verdes (e até o primeiro teste de assinatura), uma faixa aparece acima do
conteúdo de qualquer aba, com `list-checks` e "Primeiros passos":

`[✓] App instalado · [!] Extensão no navegador · [✓] Certificado encontrado · [ ] Teste de assinatura [Testar assinatura]`

"Testar assinatura" abre `https://<site>/teste` no navegador padrão: a página gera 32 bytes aleatórios, chama
`sign()`, mostra o **mesmo código de conferência** e o resultado ("Funcionou. Assinado com Ana Beatriz Souza,
ICP-Brasil A3."). **Por quê:** ensina o hábito de conferir o código antes do primeiro laudo real.
"Ocultar" (`onboarding.dismiss`) esconde a faixa para sempre.

### 8.3 Aba Navegadores

Subtítulo: "Onde a extensão está instalada e conectada ao app."

**Seção "Navegadores neste computador"** — uma linha por navegador instalado (Chrome, Edge, Firefox, Brave,
Chromium, Safari; detectados por caminho de instalação/registro/LaunchServices, nunca pelas pastas de perfil):

| Estado | Linha | Ação |
|--------|-------|------|
| Conectado | `check-circle` "Extensão 1.4.2 conectada · há 3 min" | — |
| Extensão não detectada | `warning` "Extensão não detectada" + dica "Se o Chrome mostrar “Nova extensão adicionada”, clique em Ativar." (Chrome/Edge) | [Instalar extensão ↗] abre a loja certa **naquele** navegador |
| Extensão desatualizada | `warning` "Extensão 1.2.0 · precisa da 1.3.0 ou mais nova" + "Reinicie o Chrome para atualizar." | — |
| Registro do app ausente | `x-circle` "O Chrome não encontra o app WebeSign" | [Reparar] regrava o manifesto; depois "Reparado. Reinicie o Chrome." |
| Safari com extensão desligada | `warning` "Extensão desativada no Safari" | [Abrir ajustes do Safari] (`SFSafariApplication.showPreferencesForExtension`) |
| Firefox Snap (Linux) | `info` "Firefox instalado como Snap: na primeira assinatura, permita o acesso quando o sistema perguntar." | — |

"Conectado" usa os pings registrados (decisão 8 do brief) com horário relativo ("há 3 min", "ontem, 17:40").
Vazio: "Nenhum navegador compatível encontrado. O WebeSign funciona com Chrome, Edge, Firefox, Brave e Safari."

**Seção "Sites com permissão"** — origem com destaque do domínio registrável, "Lembrado em 12/09/2026 · último
uso hoje, 14:02", botão [Revogar] (secundário). Revogar pede confirmação inline (o botão vira "Confirmar
revogação" por 4 s). Vazio: "Nenhum site lembrado. Quando você marcar “Lembrar este site” numa assinatura, ele
aparece aqui."

### 8.4 Aba Dispositivos

Subtítulo: "Tokens, cartões e drivers que o WebeSign enxerga agora." A lista atualiza ao vivo (PC/SC).

1. **(Só Mac App Store)** cartão do Complemento no topo, quando relevante ([§7](#7-complemento-macos)).
2. **(Só Linux)** linha "Serviço de cartões (pcscd)": `check-circle` "Ativo" ou `x-circle` "Parado" com o
   comando copiável `sudo systemctl enable --now pcscd.socket`.
3. **Tokens e cartões** — nome (devices.json ou "Dispositivo de cartão inteligente"), `USB 0529:0620` em
   `text-mono` `fg-subtle`, e status: `check-circle` "2 certificados" · `warning` "Sem certificados · instale o
   SafeNet Authentication Client" [Baixar para Windows ↗] · `circle-dashed` "Não sabemos se tem certificados".
4. **Leitores de cartão** — nome do leitor; "Sem cartão" (cinza) ou "Cartão: G&D StarSign · 1 certificado" com
   o ATR em `text-mono` (truncado no meio, botão `copy`).
5. **Drivers de token** — nome do módulo (`C_GetInfo.libraryDescription`), caminho em `text-mono`, status
   (`check-circle` "Carregado · 1 token", `circle-dashed` "Carregado · nenhum token conectado", `x-circle`
   "Não carregou: arquivo não encontrado"), origem ("Encontrado automaticamente" / "Adicionado por você"
   [Remover]). Abaixo: [plus Adicionar driver…] com "Use só se o fabricante do token pedir." — abre seletor de
   arquivo (`.dll`/`.dylib`/`.so`), carrega na hora e mostra o resultado na própria linha.

Vazio: "Nenhum token ou leitor conectado. Conecte o token na porta USB; esta lista atualiza sozinha."

### 8.5 Aba Certificados

Subtítulo: "Todos os certificados que este computador oferece, inclusive os que não servem para assinar."

- Barra de ação: [file-plus Importar arquivo .pfx…] + texto "O Windows importa o arquivo e guarda o
  certificado com segurança. Você vai precisar da senha do arquivo."
  - Windows: `CryptUIWizImport` com a nossa janela como mãe (o assistente do Windows faz tudo).
  - Mac: `NSOpenPanel` → abre o arquivo no Acesso às Chaves, que pede a senha e importa no chaveiro de login.
  - Linux: botão oculto; texto "No Linux, use o certificado em token ou cartão." `TODO(gustavo)`: caminho para A1 no Linux.
  - Ao voltar o foco para a janela, a lista é relida.
- Grupos por origem: "No Windows" / "No Keychain do Mac" / "Pelo driver do token". Linhas iguais às da
  Confirmação (sem rádio), com "Detalhes" em todas.
- Grupo recolhido "Não servem para assinar ({n})" com o motivo de cada um: "Vencido em …", "Ainda não vale",
  "Sem chave privada neste computador", "Feito para login, não para assinatura".
- Vazio: "Nenhum certificado neste computador. Conecte o token, insira o cartão ou importe um arquivo .pfx."

### 8.6 Aba Ajuda

1. **Perguntas comuns** (acordeão; a primeira aberta):
   - "Meu certificado não aparece" → checklist: token conectado? driver instalado (link para Dispositivos)?
     certificado vencido (link para Certificados)? A1 importado?
   - "Errei o PIN / o token bloqueou" → explica PUK, onde desbloquear, procurar a AC.
   - "O que o WebeSign envia para os sites?" → só a assinatura do código de conferência e o certificado
     escolhido; nunca o PIN; nunca o documento.
   - "Atalhos de teclado" → tabela da §4.9 e da §8.8.
2. **Relatar um problema**: pré-visualização exata do diagnóstico (caixa `bg-sunken`, `text-mono`, 8 linhas,
   rolagem) + [copy Copiar diagnóstico] + [arrow-square-out Abrir página de suporte ↗] (issues do projeto).
   Texto: "Isto é exatamente o que será copiado. Não inclui nomes, CPF, sites nem números de série."
3. **Sobre**: "WebeSign 1.4.0 · protocolo 1 · GPL-3.0-or-later" + [Código-fonte ↗] [Privacidade ↗].

### 8.7 "Copiar diagnóstico": conteúdo exato

Texto puro, em inglês (é para o suporte e para issues públicas), gerado por uma função pura testável.

```text
WebeSign diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  chrome 129.0 · extension 1.4.2 · host registered · last ping 2026-09-29T14:02Z
  edge 129.0 · extension not seen · host registered
  firefox 131.0 · extension 1.4.2 · host registered · last ping 2026-09-28T20:40Z
devices:
  usb 0529:0620 safenet-etoken-5110 · certs 0
  reader "Identiv uTrust 2700 R" · atr 3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A · certs 1
pkcs11:
  %ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll · loaded · slots 1 · tokens 1
  %USERPROFILE%\Downloads\wdpkcs_icp.dll · failed: file not found (user-added)
certificates:
  usable 3 (os 3, pkcs11 0, deduplicated 1) · hidden 2 (expired 1, login-only 1)
  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 3
  expiring<=30d 2
complement: n/a
recent errors (last 20):
  2026-09-29T14:05Z sign PinIncorrect pkcs11 CKR_PIN_INCORRECT
```

**Entra:** versões, SO, idioma, escala, backend de render, navegadores e estado da extensão, VID:PID, nome do
leitor, ATR, caminhos de módulos com a pasta do usuário trocada por variável (`%USERPROFILE%`, `~`), contagens
de certificados por tipo/algoritmo/situação, códigos de erro técnicos.
**Não entra:** nome, CPF, CNPJ, e-mail, número de série ou impressão digital de certificado, número de série
de token/USB, nome do computador, nome do usuário, sites (inclusive os lembrados), digests, códigos de
conferência. **Por quê:** o texto vai parar em issue pública; ATR e VID:PID identificam modelo, não pessoa.

### 8.8 Teclado no Diagnóstico

Ctrl/⌘+1…4 troca de aba · ↑/↓ na barra lateral · F5 ou Ctrl/⌘+R procura de novo · Ctrl/⌘+Shift+C copia o
diagnóstico · Ctrl/⌘+W fecha.

---

## 9. Popup da extensão

TS puro + CSS, sem framework, **< 15 KB** no total (HTML + CSS + JS + ícones SVG inline).
Largura **320 px**, altura pelo conteúdo (máx. 480). Fonte: pilha do sistema (`--ws-font-popup`).
**Por quê a fonte do sistema:** embutir a Inter custaria ~50 KB, mais que o popup inteiro, e o popup vive
dentro da interface do navegador, onde a fonte do sistema parece nativa.

Anatomia: cabeçalho (marca 20 px + "WebeSign") → cartão de estado (ícone 32 px + título `text-title` +
texto `text-body` `fg-muted`) → botão primário largo (`control-lg`) → ação secundária em texto → rodapé
(`text-small` `fg-subtle`): "Extensão 1.4.2 · App 1.4.0" e link "Privacidade".

| Estado | Detecção | Ícone | Título | Texto | Primário | Secundário |
|--------|----------|-------|--------|-------|----------|------------|
| `checking` | conexão em andamento (mostrado só após 150 ms) | spinner | "Verificando…" | — | — | — |
| `ready` | host respondeu com versão ≥ `MIN_APP_VERSION` | `check-circle` success | "Tudo pronto para assinar" | "O app WebeSign 1.4.0 está conectado a este navegador." | Abrir diagnóstico | — |
| `missing` | `connectNative` falhou ("host not found") | `download-simple` accent | "Falta instalar o app WebeSign" | "A extensão precisa do app no computador para assinar." | Baixar para Windows | "Já instalei? Ativar o app" |
| `outdated` | versão < `MIN_APP_VERSION` ou protocolo antigo | `warning` warning | "Atualize o app WebeSign" | "Você tem a versão 1.1.0. Este navegador precisa da 1.3.0 ou mais nova." | Atualizar na Microsoft Store | — |
| `error` | host iniciou mas não respondeu em 3 s / fechou | `x-circle` danger | "O app não respondeu" | "Tente de novo. Se continuar, reinicie o computador." + código técnico em `text-mono` | Tentar de novo | Baixar de novo |
| `unsupported` | `runtime.getPlatformInfo().os` ∉ {win, mac, linux} | `info` | "O WebeSign ainda não funciona neste sistema" | "Use um computador com Windows, macOS ou Linux." | — | — |

**App instalado mas nunca aberto (host não registrado):** para a extensão isso é igual a "não instalado"
(o `connectNative` falha do mesmo jeito). Resolução:

1. O pacote registra o esquema de URL `websign:` **na instalação** (MSIX `windows.protocol`, `CFBundleURLTypes`
   no Mac, `.desktop` com `x-scheme-handler/websign` no Linux). Isso não exige abrir o app.
2. "Já instalei? Ativar o app" abre `https://<site>/ativar` numa aba. Essa página chama `websign:activate`
   (o navegador pergunta "Abrir WebeSign?"), o app grava os manifestos de todos os navegadores e mostra a
   janela de Diagnóstico com "Primeiros passos".
3. A página `/ativar` usa o anúncio da extensão na página para mostrar ao vivo "Procurando o app… → Pronto".

**Por quê uma página e não o popup:** o popup fecha quando o navegador mostra a pergunta do protocolo;
a página continua aberta e acompanha o resultado.

Outras regras: "Abrir diagnóstico" manda `{type: "openDiagnostics"}` pelo native messaging e fecha o popup;
links de download escolhem o SO por `runtime.getPlatformInfo()` (Windows → Microsoft Store; Mac → Mac App
Store; Linux → página de download do site com .deb/.rpm); o ícone da extensão ganha o selo "!" (`warning`)
em `missing`/`outdated`/`error` e o perde em `ready`.

---

## 10. Primeira execução por sistema

| SO | O que acontece na instalação | Primeira abertura | Passos que o usuário vê |
|----|------------------------------|-------------------|-------------------------|
| **Windows** (Microsoft Store, MSIX) | Registra alias de execução e o esquema `websign:`; nada roda | A Store mostra "Abrir". O app grava HKCU (host em todas as chaves `NativeMessagingHosts` + pré-registro da extensão) e abre o Diagnóstico com "Primeiros passos" | 1. Extensão: Chrome/Edge mostram "Nova extensão adicionada" → "clique em Ativar"; Firefox → [Instalar extensão ↗]. 2. Certificado encontrado. 3. [Testar assinatura]. |
| **macOS** (Mac App Store) | Registra o esquema `websign:`; nada roda | O app grava os manifestos (Chrome, Edge, Brave, Firefox) e abre o Diagnóstico | 1. Safari: [Abrir ajustes do Safari] para ligar a extensão; ao usar no site, escolher "Permitir sempre neste site". Chrome/Edge/Firefox: [Instalar extensão ↗]. 2. Certificado (e Complemento, se preciso — §7). 3. [Testar assinatura]. |
| **Linux** (.deb/.rpm) | `postinst` grava manifestos de sistema (`/etc/opt/chrome/native-messaging-hosts`, `/usr/lib/mozilla/native-messaging-hosts`, …) e ativa `pcscd.socket` | Pelo menu de apps, abre o Diagnóstico | 1. [Instalar extensão ↗]; Firefox Snap: aviso do portal. 2. `pcscd` ativo? 3. Certificado. 4. [Testar assinatura]. |

Regra comum: o app **nunca** abre sozinho no login e não fica residente (decisão 4 do brief). O Diagnóstico
é a única "tela de boas-vindas".

---

## 11. Tokens de design

Fonte única para `app/src/ui/theme/` (egui), `design/tokens.css` (popup e site) e para os mockups.
Nomes: `kebab-case` no documento e no CSS (`--ws-bg-canvas`), `snake_case` no Rust (`tokens.bg_canvas`).
Um teste do app lê `design/tokens.css` com `include_str!` e compara cada `--ws-*` com a constante Rust.
**Por quê um teste e não um gerador:** mantém os dois arquivos legíveis à mão e custa um teste de 40 linhas.

Direção visual: neutros frios levemente puxados para o azul da marca; um único acento, "azul-tinta"
(a cor de caneta de assinatura), usado só em ação primária, seleção e foco. Cor semântica (verde, âmbar,
vermelho) é separada do acento e reservada para estado.

### 11.1 Cores

| Token | Claro | Escuro | Uso |
|-------|-------|--------|-----|
| `bg-canvas` | `#F5F6F9` | `#0E1016` | Fundo das janelas (corpo) |
| `bg-surface` | `#FFFFFF` | `#161922` | Cabeçalho, rodapé, cartões, lista, barra lateral |
| `bg-sunken` | `#EDEFF4` | `#0B0D12` | Campos, poço do identicon, selos, caixa de diagnóstico |
| `bg-hover` | `#F1F3F8` | `#1E222D` | Hover de linhas e botões secundários |
| `accent-soft` | `#EEF0FD` | `#1C2242` | Linha/aba selecionada |
| `border` | `#DCE0E8` | `#2A2F3C` | Divisórias e contorno de cartões (decorativo) |
| `border-strong` | `#848D9F` | `#6B7488` | Contorno de campo, rádio, checkbox (≥ 3:1) |
| `fg` | `#141722` | `#E7E9EF` | Texto principal |
| `fg-muted` | `#545C6D` | `#A3AAB9` | Texto secundário |
| `fg-subtle` | `#636B7D` | `#8C94A6` | Metadados, placeholders, esquema/subdomínio |
| `accent` | `#3346D1` | `#4E5EE4` | Fundo do botão primário, rádio marcado |
| `accent-hover` | `#2A3BB8` | `#4555DA` | Hover do primário |
| `accent-pressed` | `#2332A0` | `#3E4ECC` | Pressionado |
| `accent-fg` | `#3346D1` | `#9DA8FF` | Links, ícones de acento, texto da aba selecionada |
| `on-accent` | `#FFFFFF` | `#FFFFFF` | Texto sobre `accent` |
| `focus` | `#3346D1` | `#9DA8FF` | Anel de foco (2 px) |
| `success` | `#127A41` | `#5BCB8D` | Texto/ícone de sucesso |
| `success-soft` | `#E6F5EC` | `#10281B` | Fundo de chip/aviso de sucesso |
| `success-border` | `#A8DCBE` | `#1F5C3B` | Borda de aviso de sucesso |
| `warning` | `#935300` | `#EFB35E` | Texto/ícone de atenção |
| `warning-soft` | `#FFF2D9` | `#2A1F0E` | Fundo de chip/aviso de atenção |
| `warning-border` | `#EFCB8A` | `#5C4418` | Borda de aviso de atenção |
| `danger` | `#BE242B` | `#FF868B` | Texto/ícone de erro |
| `danger-soft` | `#FDEBEC` | `#321519` | Fundo de aviso de erro |
| `danger-border` | `#F2B3B6` | `#6B2429` | Borda de aviso de erro; borda de campo com erro usa `danger` |
| `shadow-color` | `#1417221F` | `#0000008C` | Cor da sombra de popover/tooltip |

Paleta do identicon (igual nos dois temas, índice 0–7): `id-0 #D6453A` · `id-1 #C4610A` · `id-2 #3B8A3B` ·
`id-3 #12857B` · `id-4 #2D78D2` · `id-5 #5A55D6` · `id-6 #9549C4` · `id-7 #C63E7B`. Todas ≥ 3:1 contra
`bg-surface` e `bg-sunken` nos dois temas.

Contraste medido (WCAG 2.x):

| Par | Claro | Escuro | Mínimo |
|-----|-------|--------|--------|
| `fg` sobre `bg-surface` | 17,9 | 14,5 | 4,5 |
| `fg-muted` sobre `bg-canvas` | 6,2 | 8,2 | 4,5 |
| `fg-subtle` no pior fundo (claro: `bg-sunken`; escuro: `accent-soft`) | 4,65 | 5,1 | 4,5 |
| `accent-fg` sobre `accent-soft` | 6,3 | 7,0 | 4,5 |
| `on-accent` sobre `accent` | 7,2 | 5,2 | 4,5 |
| `on-accent` sobre `accent-hover` | 8,7 | 5,9 | 4,5 |
| `success` / `warning` / `danger` sobre o `*-soft` | 4,8 / 5,5 / 5,3 | 7,7 / 8,7 / 7,2 | 4,5 |
| `border-strong` sobre `bg-surface` | 3,3 | 3,7 | 3,0 (componente) |
| `focus` sobre `bg-surface` | 7,2 | 7,9 | 3,0 |

### 11.2 Tipografia

| Família | Token CSS | Arquivos embutidos | Por quê |
|---------|-----------|--------------------|---------|
| Inter | `--ws-font-ui: "Inter", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif` | Inter Regular 400, Medium 500, SemiBold 600 (OFL) | Decisão do brief; ótima em tamanhos pequenos. |
| JetBrains Mono | `--ws-font-mono: "JetBrains Mono", ui-monospace, "SF Mono", Consolas, monospace` | Regular 400 e Medium 500, **subconjunto** ASCII (~20 KB) (OFL) | Distingue 0/O e 1/l no código de conferência; o egui não tem algarismos tabulares na Inter. |
| Sistema | `--ws-font-popup: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", sans-serif` | nenhum | Só no popup ([§9](#9-popup-da-extensão)). |

| Token | Tamanho/linha (px) | Peso | Família | Uso |
|-------|--------------------|------|---------|-----|
| `text-caption` | 12/16 | 500 | ui | Rótulos de seção, selos, chips |
| `text-small` | 12/16 | 400 | ui | Metadados, ajudas, rodapé do popup |
| `text-body` | 14/20 | 400 | ui | Texto padrão |
| `text-body-strong` | 14/20 | 600 | ui | Nome do titular, nome de navegador/dispositivo |
| `text-button` | 14/20 | 500 | ui | Botões |
| `text-title` | 16/22 | 600 | ui | Títulos de seção, título do popup, origem longa |
| `text-headline` | 20/26 | 600 | ui | Origem na Confirmação, título da aba |
| `text-code` | 18/24 | 500 | mono | Código de conferência (espaçamento extra de 0,5 px entre letras) |
| `text-mono` | 12/16 | 400 | mono | VID:PID, ATR, caminhos, impressão digital |

Sem caixa alta em rótulos (pt-BR em caixa alta fica pesado e o leitor de tela soletra siglas).
**Por quê corpo 14:** o público inclui médicos mais velhos em monitores de consultório; 12 é pouco.

### 11.3 Espaçamento, tamanhos e raios

| Token | Valor | | Token | Valor |
|-------|-------|-|-------|-------|
| `space-1` | 4 | | `control-sm` | 28 (checkbox, botões de linha) |
| `space-2` | 8 | | `control-md` | 32 (campos, botões comuns) |
| `space-3` | 12 | | `control-lg` | 36 (botões do rodapé e do popup) |
| `space-4` | 16 | | `row-cert` | 72 |
| `space-5` | 20 | | `row-compact` | 40 |
| `space-6` | 24 | | `row-diag` | 56 |
| `space-8` | 32 | | `icon-sm` / `icon-md` / `icon-lg` | 16 / 20 / 32 |
| `radius-sm` | 4 (selos, identicon) | | `identicon` | 40 |
| `radius-md` | 6 (botões, campos, linhas, abas) | | `sidebar` | 200 |
| `radius-lg` | 10 (cartões, lista, avisos, popup) | | `window-confirm` | 480 × 600 |
| `radius-full` | metade da altura (chips) | | `window-diagnostics` | 760 × 540 |
| | | | `popup-width` | 320 |

Sombras: só `shadow-popover` = deslocamento 0/4, desfoque 16, espalhamento 0, cor `shadow-color` (tooltips e
menus). Cartões usam borda, não sombra. **Por quê:** a janela já tem a sombra do SO; sombra interna em tudo
achata a hierarquia.

### 11.4 Movimento

| Token | Valor | Uso |
|-------|-------|-----|
| `motion-fast` | 120 ms | hover, pressionar, anel de foco |
| `motion-base` | 180 ms | expandir/recolher, entrada de linha, armar botão, troca de aba |
| `motion-slow` | 260 ms | check de sucesso |
| `easing` | cubic-out (`emath::easing::cubic_out`) | todas |
| `delay-arming` | 600 ms | botão Assinar |
| `delay-loading` | 150 ms | só mostra carregando se passar disso |
| `delay-slow-hint` | 2000 ms | "Ainda lendo {device}…" |
| `hold-success` | 900 ms | sucesso antes de fechar |
| `hold-site-cancelled` | 1500 ms | "site cancelou" antes de fechar |
| `timeout-request` | 300 s | pedido sem decisão |

Movimento reduzido: se o SO pedir (`SPI_GETCLIENTAREAANIMATION` no Windows,
`accessibilityDisplayShouldReduceMotion` no Mac, `enable-animations` do GNOME, `prefers-reduced-motion` no
popup), todas as durações de animação viram 0. Temporizadores de segurança (`delay-arming`) e de estado não
mudam. O egui está em modo reativo: animação só repinta enquanto `ctx.animate_*` estiver em curso.

### 11.5 Mapa para o egui (`theme.rs`)

| Campo egui | Token |
|------------|-------|
| `Visuals::dark_mode` | tema do sistema (`ctx.options.theme_preference = System`) |
| `panel_fill` | `bg-canvas` |
| `window_fill` | `bg-surface` |
| `extreme_bg_color` (fundo de `TextEdit`) | `bg-sunken` |
| `faint_bg_color` | `bg-hover` |
| `code_bg_color` | `bg-sunken` |
| `hyperlink_color` | `accent-fg` |
| `warn_fg_color` / `error_fg_color` | `warning` / `danger` |
| `selection.bg_fill` / `selection.stroke` | `accent-soft` / 1 px `accent-fg` |
| `text_cursor.stroke` | 2 px `accent-fg` |
| `widgets.noninteractive` | `bg_fill` `bg-surface`, `bg_stroke` 1 px `border`, `fg_stroke` `fg` |
| `widgets.inactive` | `weak_bg_fill` `bg-surface`, `bg_stroke` 1 px `border-strong`, `fg_stroke` `fg`, raio `radius-md` |
| `widgets.hovered` | `weak_bg_fill` `bg-hover`, `bg_stroke` 1 px `border-strong`, `expansion` 0 |
| `widgets.active` | `weak_bg_fill` `bg-sunken`, `bg_stroke` 1 px `fg-subtle`, `expansion` 0 |
| `widgets.open` | igual a `hovered` |
| `window_corner_radius` / `menu_corner_radius` | `radius-lg` / `radius-md` |
| `window_shadow` / `popup_shadow` | nenhuma (SO desenha) / `shadow-popover` |
| `window_stroke` | 1 px `border` |
| `Spacing::item_spacing` | (`space-2`, `space-2`) |
| `Spacing::button_padding` | (`space-3`, 0) com altura pelo `control-*` |
| `Spacing::interact_size` | (32, `control-md`) |
| `Spacing::icon_width` / `icon_spacing` | `icon-sm` / `space-2` |
| `Spacing::indent` | `space-4` |
| `TextStyle::Small / Body / Button / Heading / Monospace` | `text-small / text-body / text-button / text-headline / text-mono` |
| `TextStyle::Name("caption" \| "body-strong" \| "title" \| "code")` | os tokens de mesmo nome |

Pesos: registrar famílias `FontFamily::Name("inter-medium")` e `FontFamily::Name("inter-semibold")`
(o egui não sintetiza peso). Altura de linha via `TextFormat::line_height`. O botão primário, os chips, o
rádio, o identicon e o anel de foco são widgets próprios em `app/src/ui/widgets/` (um por arquivo), porque o
egui não tem esses visuais prontos.

---

## 12. Ícones

Phosphor 2.1 via `egui-phosphor` (constantes em `SCREAMING_SNAKE_CASE`, ex.: `regular::SEAL_CHECK`). No
popup e no site, SVG inline dos mesmos arquivos (`@phosphor-icons/core`). Peso **regular** por padrão;
**fill** só para status. Tamanho 16 em linha de texto, 20 em botões de ícone e abas, 32 em cartões de estado.

| Conceito | Ícone | Peso |
|----------|-------|------|
| Marca provisória do app (`TODO(gustavo)`: logo definitivo) | `seal-check` | fill |
| Pedido de assinatura (sobrancelha) | `signature` | regular |
| Pedido de certificado (modo Escolher) | `certificate` | regular |
| Site com https | `lock-simple` | regular |
| Site local de desenvolvimento | `terminal-window` | regular |
| Código de conferência (rótulo acessível / Detalhes) | `hash` | regular |
| Certificado (genérico, aba Certificados) | `certificate` | regular |
| Neste computador | `desktop` | regular |
| Token USB / aba Dispositivos | `usb` | regular |
| Cartão no leitor / leitor de cartão | `identification-card` | regular |
| Driver do token (PKCS#11) | `plug` | regular |
| Complemento (macOS) | `puzzle-piece` | regular |
| PIN (rótulo do campo, leitor de tela) | `password` | regular |
| Teclado do leitor (PIN pad) | `dots-nine` | regular |
| Token desbloqueado nesta sessão | `lock-key-open` | regular |
| PIN bloqueado | `lock-key` | fill |
| Privacidade do PIN / Privacidade | `shield-check` | regular |
| Mostrar / ocultar PIN | `eye` / `eye-slash` | regular |
| Vence em breve | `clock` | regular |
| Sucesso / semáforo verde | `check-circle` | fill |
| Atenção / semáforo amarelo | `warning` | fill |
| Erro / semáforo vermelho | `x-circle` | fill |
| Informação / site novo | `info` | fill |
| Não se aplica / sem dados | `circle-dashed` | regular |
| Navegadores (aba) | `browsers` | regular |
| Navegador (linha) | `browser` | regular |
| Sites com permissão | `globe` | regular |
| Certificados (aba) | `certificate` | regular |
| Ajuda (aba) | `question` | regular |
| Diagnóstico (botões "Abrir diagnóstico") | `stethoscope` | regular |
| Primeiros passos | `list-checks` | regular |
| Baixar | `download-simple` | regular |
| Link externo (sufixo) | `arrow-square-out` | regular |
| Copiar | `copy` | regular |
| Procurar de novo / tentar de novo | `arrow-clockwise` | regular |
| Adicionar driver | `plus` | regular |
| Importar .pfx | `file-plus` | regular |
| Filtrar | `magnifying-glass` | regular |
| Expandir / recolher | `caret-right` / `caret-down` | regular |
| Fechar | `x` | regular |
| Atalhos de teclado | `keyboard` | regular |

**Por quê ícone genérico para navegadores:** o Phosphor não tem Edge nem Brave; misturar logos de marca com
ícone genérico fica desigual, e o nome do navegador já identifica.

---

## 13. Textos e i18n

### 13.1 Formato

- **Um arquivo TOML por idioma** em `i18n/`: `en.toml`, `pt-BR.toml`, `pt-PT.toml`, `es.toml`, `fr.toml`.
  **Por quê TOML:** aceita comentários para tradutores (JSON não), é legível e o Rust lê sem esforço.
- `en.toml` é a **referência**: define o conjunto de chaves. CI falha se `pt-BR` ou `en` tiver chave faltando,
  sobrando ou com placeholders diferentes; `pt-PT`, `es`, `fr` podem atrasar (caem no fallback).
- Chaves: seções por área (`[confirm]`, `[cert]`, …), nomes em `snake_case`; a chave completa é o caminho com
  pontos (`confirm.window_title`).
- Placeholders `{nome}`. Plural como subtabela com categorias CLDR `one`/`many`/`other` (só `other` é
  obrigatória). O app implementa as regras de plural dos 5 idiomas em uma função pura (sem dependência).

```toml
# i18n/pt-BR.toml
[confirm]
# Título da janela no SO. {site} é o host, ex.: app.diagnos.health
window_title = "Assinar para {site} — WebeSign"

[cert.expires_in]
one = "Vence em {count} dia"
other = "Vence em {count} dias"
```

- **Rust:** `build.rs` lê `i18n/*.toml` e gera constantes (`k::CONFIRM_WINDOW_TITLE`) e as tabelas por
  idioma; chave errada é erro de compilação. Uso: `tr(k::CERT_EXPIRES_IN).count(23)`.
- **Extensão:** um passo do build do WXT converte as seções `[popup]` e `[store]` para
  `_locales/<pt_BR|pt_PT|es|fr|en>/messages.json` (nome `popup.ready_title` → `popup_ready_title`;
  `{version}` → `$version$` com `placeholders`). O popup usa `chrome.i18n.getMessage`, e a loja fica
  traduzida de graça. Regra: strings do popup não têm plural (o `chrome.i18n` não suporta).
- **SDK:** a seção `[site.errors]` gera `sdk/src/messages.gen.ts`, exportado num ponto de entrada separado
  (`@websign/sdk/messages`), para o SDK continuar com zero dependências e o site só levar se quiser.
- **Idioma:** app pelo idioma do SO (`sys-locale`); popup pelo idioma do navegador. Cadeia de fallback:
  `pt-BR → pt-PT → en`, `pt-PT → pt-BR → en`, `es → en`, `fr → en`, outros → `en`.
- Datas e números: formatados por função própria por idioma (`dd/mm/aaaa`, `14 Mar 2027`), sem ICU.

### 13.2 Strings (pt-BR e en)

`{…}` são placeholders. Plurais indicados como `one / other`.

#### Comuns

| Chave | pt-BR | en |
|-------|-------|----|
| `common.cancel` | Cancelar | Cancel |
| `common.close` | Fechar | Close |
| `common.copy` | Copiar | Copy |
| `common.copied` | Copiado | Copied |
| `common.retry` | Tentar de novo | Try again |
| `common.wait` | Aguarde… | Please wait… |
| `common.details` | Detalhes | Details |
| `common.technical_details` | Detalhes técnicos | Technical details |
| `common.open_diagnostics` | Abrir diagnóstico | Open diagnostics |
| `common.refresh` | Procurar de novo | Scan again |
| `common.download_for` | Baixar para {os} | Download for {os} |
| `common.learn_more` | Saiba mais | Learn more |
| `common.new_window_hint` | (abre no navegador) | (opens in your browser) |
| `os.windows` / `os.macos` / `os.linux` | Windows / macOS / Linux | Windows / macOS / Linux |
| `time.just_now` | agora há pouco | just now |
| `time.minutes_ago` | há {count} min | {count} min ago |
| `time.today_at` | hoje, {time} | today, {time} |
| `time.yesterday_at` | ontem, {time} | yesterday, {time} |

#### Confirmação: cabeçalho e origem

| Chave | pt-BR | en |
|-------|-------|----|
| `confirm.window_title` | Assinar para {site} — WebeSign | Sign for {site} — WebeSign |
| `confirm.window_title_select` | Escolher certificado para {site} — WebeSign | Choose certificate for {site} — WebeSign |
| `confirm.eyebrow` | Pedido de assinatura | Signature request |
| `confirm.eyebrow_select` | Pedido de certificado | Certificate request |
| `confirm.via_browser` | pelo {browser} | via {browser} |
| `confirm.asks_sign` | quer que você assine um documento. | wants you to sign a document. |
| `confirm.asks_select` | quer saber com qual certificado você vai assinar. | wants to know which certificate you will sign with. |
| `confirm.site_remembered` | Site com permissão | Allowed site |
| `confirm.site_new` | Site novo | New site |
| `confirm.site_new_a11y` | Primeira vez que este site pede algo neste computador | First time this site asks for anything on this computer |
| `confirm.eyebrow_queue` | Pedido de assinatura {current} de {total} | Signature request {current} of {total} |
| `confirm.inside_frame` | Dentro da página de {top_site} | Inside a page from {top_site} |
| `origin.warn_idn` | Endereço com caracteres especiais. Confira letra por letra. | Address with special characters. Check it letter by letter. |
| `origin.shown_as` | Aparece como {unicode} | Displays as {unicode} |
| `origin.warn_ip` | Endereço numérico, sem nome de site | Numeric address with no site name |
| `origin.warn_ip_local` | Endereço numérico da rede local | Local network numeric address |
| `origin.warn_localhost` | Site local de desenvolvimento | Local development site |
| `origin.blocked_http` | Este site não usa conexão segura (https). Por segurança, o WebeSign não assina para ele. | This site doesn't use a secure connection (https). For your safety, WebeSign won't sign for it. |

#### Confirmação: código, lista e certificado

| Chave | pt-BR | en |
|-------|-------|----|
| `code.label` | Código de conferência | Verification code |
| `code.help` | Confira se o site mostra o mesmo código. | Check that the site shows the same code. |
| `code.preparing` | Preparando o documento… | Preparing the document… |
| `code.a11y` | Código de conferência: {spelled} | Verification code: {spelled} |
| `certs.label_sign` | Assinar com | Sign with |
| `certs.label_select` | Escolha o certificado | Choose a certificate |
| `certs.loading` | Procurando certificados… | Looking for certificates… |
| `certs.loading_slow` | Ainda lendo {device}. Drivers de token podem levar alguns segundos. | Still reading {device}. Token drivers can take a few seconds. |
| `certs.empty_title` | Nenhum certificado encontrado | No certificates found |
| `certs.empty_body` | Conecte o token ou insira o cartão no leitor. A lista atualiza sozinha. | Plug in your token or insert your card. The list updates by itself. |
| `certs.filter_placeholder` | Filtrar por nome, CPF ou emissor | Filter by name, ID number or issuer |
| `certs.filter_empty` | Nenhum certificado com “{query}” | No certificate matches “{query}” |
| `certs.unusable_group` | Não podem assinar ({count}) | Can't sign ({count}) |
| `certs.found_a11y` | Certificado encontrado: {name} | Certificate found: {name} |
| `cert.doc_cpf` | CPF {masked} | CPF {masked} |
| `cert.doc_cpf_a11y` | CPF parcialmente oculto, {visible} | CPF partially hidden, {visible} |
| `cert.doc_cnpj` | CNPJ {cnpj} | CNPJ {cnpj} |
| `cert.doc_generic` | Documento {masked} | ID {masked} |
| `cert.kind.icp` | ICP-Brasil {class} | ICP-Brasil {class} |
| `cert.kind.icp_plain` | ICP-Brasil | ICP-Brasil |
| `cert.kind.eidas_qscd` | Qualificado eIDAS | Qualified eIDAS |
| `cert.kind.eidas` | eIDAS | eIDAS |
| `cert.kind.pt_cc` | Cartão de Cidadão | Cartão de Cidadão |
| `cert.kind.es_dnie` | DNIe | DNIe |
| `cert.kind.generic` | Certificado | Certificate |
| `cert.where.computer` | Neste computador | On this computer |
| `cert.where.token_named` | Token {device} | Token {device} |
| `cert.where.card_in_reader` | Cartão no leitor | Card in reader |
| `cert.where.unknown_hw` | Token ou cartão | Token or card |
| `cert.via_driver` | pelo driver | via driver |
| `cert.valid_until` | Válido até {date} | Valid until {date} |
| `cert.expires_in` (one / other) | Vence em {count} dia / Vence em {count} dias | Expires in {count} day / Expires in {count} days |
| `cert.expires_tomorrow` | Vence amanhã | Expires tomorrow |
| `cert.expires_today` | Vence hoje | Expires today |
| `cert.expired_on` | Venceu em {date} | Expired on {date} |
| `cert.valid_from` | Válido a partir de {date} | Valid from {date} |
| `cert.reason.incompatible` | Não compatível com este pedido | Not compatible with this request |
| `cert.reason.pin_locked` | PIN bloqueado | PIN locked |
| `cert.reason.removed` | Removido. Conecte o token de novo. | Removed. Plug the token back in. |
| `cert.reason.no_key` | Sem chave privada neste computador | No private key on this computer |
| `cert.reason.login_only` | Feito para login, não para assinatura | Made for login, not for signing |
| `cert.details.subject` | Titular | Holder |
| `cert.details.issuer` | Emissor | Issuer |
| `cert.details.validity` | Validade | Validity |
| `cert.details.validity_range` | {from} a {to} | {from} to {to} |
| `cert.details.usage` | Uso | Usage |
| `cert.details.usage_sign` | Assinatura digital | Digital signature |
| `cert.details.usage_nonrep` | Não repúdio | Non-repudiation |
| `cert.details.key` | Chave | Key |
| `cert.details.fingerprint` | Impressão digital (SHA-256) | Fingerprint (SHA-256) |
| `cert.details.access` | Acesso | Access |
| `cert.details.access_windows` | Windows (repositório de certificados) | Windows (certificate store) |
| `cert.details.access_macos` | Keychain do macOS | macOS Keychain |
| `cert.details.access_driver` | Driver do token: {path} | Token driver: {path} |
| `cert.details.also_via_driver` | Também acessível pelo driver do token | Also available through the token driver |
| `cert.details.view_in_system` | Ver no sistema | View in system |

#### Confirmação: PIN, permissão, botões e estados

| Chave | pt-BR | en |
|-------|-------|----|
| `pin.label_token` | PIN do token | Token PIN |
| `pin.label_card` | PIN do cartão | Card PIN |
| `pin.length_hint` | {min} a {max} caracteres | {min} to {max} characters |
| `pin.show` / `pin.hide` | Mostrar PIN / Ocultar PIN | Show PIN / Hide PIN |
| `pin.privacy` | O PIN fica neste computador e não passa pelo navegador. | Your PIN stays on this computer and never goes through the browser. |
| `pin.os_prompt` | O {os} vai pedir o PIN numa janela própria. | {os} will ask for your PIN in its own window. |
| `pin.os_prompt_now` | Digite o PIN na janela do {os}. | Enter your PIN in the {os} window. |
| `pin.pinpad_before` | Depois de clicar em Assinar, digite o PIN no teclado do leitor. | After you click Sign, enter your PIN on the reader's keypad. |
| `pin.pinpad_now` | Digite o PIN no teclado do leitor. | Enter your PIN on the reader's keypad. |
| `pin.unlocked_session` | Token desbloqueado nesta sessão | Token unlocked for this session |
| `pin.incorrect` | PIN incorreto. | Incorrect PIN. |
| `pin.incorrect_low` | PIN incorreto. Restam poucas tentativas antes de o token bloquear. | Incorrect PIN. Only a few attempts left before the token locks. |
| `pin.incorrect_final` | PIN incorreto. Última tentativa: se errar de novo, o token bloqueia. | Incorrect PIN. Last attempt: one more mistake locks the token. |
| `pin.locked_title` | PIN bloqueado | PIN locked |
| `pin.locked_body` | O token bloqueou depois de muitas tentativas erradas. Desbloqueie com o PUK no {tool} ou procure a {issuer}. | The token locked after too many wrong attempts. Unlock it with the PUK in {tool} or contact {issuer}. |
| `pin.locked_body_generic` | O token bloqueou depois de muitas tentativas erradas. Desbloqueie com o PUK no programa do fabricante ou procure a autoridade certificadora. | The token locked after too many wrong attempts. Unlock it with the PUK in the vendor's software or contact your certificate authority. |
| `consent.remember` | Lembrar este site neste computador | Remember this site on this computer |
| `consent.remember_help` | Ele poderá saber qual certificado você usa sem perguntar. Cada assinatura continua pedindo sua confirmação. | It will be able to see which certificate you use without asking. Every signature still asks for your confirmation. |
| `consent.remember_disabled` | Endereços numéricos ou com caracteres especiais não podem ser lembrados. | Numeric or special-character addresses can't be remembered. |
| `consent.select_shares` | O site vai receber nome, tipo, emissor e validade do certificado escolhido. Nada é assinado agora. | The site will receive the name, type, issuer and validity of the chosen certificate. Nothing is signed now. |
| `action.sign` | Assinar | Sign |
| `action.signing` | Assinando… | Signing… |
| `action.use_cert` | Usar este certificado | Use this certificate |
| `footer.expires_in` | Este pedido expira em {seconds} s | This request expires in {seconds} s |
| `state.success_title` | Assinado | Signed |
| `state.success_body` | A assinatura foi enviada para {site}. | The signature was sent to {site}. |
| `state.select_success` | Certificado enviado para {site} | Certificate sent to {site} |
| `state.site_cancelled` | {site} cancelou o pedido. | {site} cancelled the request. |

#### Possíveis certificados e Complemento

| Chave | pt-BR | en |
|-------|-------|----|
| `possible.title` | {device} conectado | {device} connected |
| `possible.body_driver_token` | Nenhum certificado apareceu neste token. Para usá-lo, instale o {driver}. | No certificate showed up on this token. To use it, install {driver}. |
| `possible.body_driver_card` | Nenhum certificado apareceu neste cartão. Para usá-lo, instale o {driver}. | No certificate showed up on this card. To use it, install {driver}. |
| `possible.body_unknown_card` | Há um cartão no leitor {reader}, mas não o reconhecemos. Se ele tem certificado, instale o programa do fabricante do cartão. | There's a card in {reader}, but we don't recognize it. If it holds a certificate, install the card vendor's software. |
| `possible.no_link` | Procure o programa no site da autoridade certificadora que emitiu o seu certificado. | Look for the software on the website of the authority that issued your certificate. |
| `possible.rescan` | Já instalei, procurar de novo | I installed it, scan again |
| `possible.inline` | {device} conectado sem certificados | {device} connected with no certificates |
| `possible.inline_action` | Como resolver | How to fix |
| `possible.step` | {n}. {text} | {n}. {text} |
| `possible.step_install_driver` | Instale o {driver} | Install {driver} |
| `possible.step_install_complement` | Instale o Complemento WebeSign | Install the WebeSign Add-on |
| `complement.name` | Complemento WebeSign | WebeSign Add-on |
| `complement.title_needed` | Este token precisa do Complemento para Mac | This token needs the Mac Add-on |
| `complement.body_needed` | Apps da App Store não podem usar alguns drivers de token. O Complemento WebeSign, gratuito e do mesmo projeto, faz essa ponte. Ele só funciona quando o WebeSign pede. | App Store apps can't use some token drivers. The free WebeSign Add-on, from the same project, bridges that gap. It only runs when WebeSign asks for it. |
| `complement.download` | Baixar Complemento | Download Add-on |
| `complement.ok` | Complemento ativo · versão {version} | Add-on active · version {version} |
| `complement.outdated` | Atualize o Complemento ({installed} instalado, precisa da {required}) | Update the Add-on ({installed} installed, {required} required) |
| `complement.update` | Atualizar Complemento | Update Add-on |
| `complement.not_responding` | O Complemento está instalado, mas não respondeu. | The Add-on is installed but didn't respond. |
| `complement.open` | Abrir Complemento | Open Add-on |

#### Diagnóstico

| Chave | pt-BR | en |
|-------|-------|----|
| `diag.window_title` | Diagnóstico — WebeSign | Diagnostics — WebeSign |
| `diag.status.ok` | Pronto para assinar | Ready to sign |
| `diag.status.attention` | Precisa de atenção | Needs attention |
| `diag.status.blocked` | Ainda não dá para assinar | Can't sign yet |
| `diag.status.na` | Não se aplica | Not applicable |
| `diag.tab.browsers` | Navegadores | Browsers |
| `diag.tab.devices` | Dispositivos | Devices |
| `diag.tab.certs` | Certificados | Certificates |
| `diag.tab.help` | Ajuda | Help |
| `diag.copy` | Copiar diagnóstico | Copy diagnostics |
| `diag.copied` | Diagnóstico copiado. Não inclui nomes, CPF nem sites. | Diagnostics copied. No names, ID numbers or sites included. |
| `onboarding.title` | Primeiros passos | Getting started |
| `onboarding.step_app` | App instalado | App installed |
| `onboarding.step_extension` | Extensão no navegador | Browser extension |
| `onboarding.step_cert` | Certificado encontrado | Certificate found |
| `onboarding.step_test` | Teste de assinatura | Test signature |
| `onboarding.test_button` | Testar assinatura | Test a signature |
| `onboarding.dismiss` | Ocultar | Hide |
| `browsers.subtitle` | Onde a extensão está instalada e conectada ao app. | Where the extension is installed and connected to the app. |
| `browsers.section` | Navegadores neste computador | Browsers on this computer |
| `browsers.connected` | Extensão {version} conectada · {when} | Extension {version} connected · {when} |
| `browsers.not_detected` | Extensão não detectada | Extension not detected |
| `browsers.install` | Instalar extensão | Install extension |
| `browsers.external_prompt_hint` | Se o {browser} mostrar “Nova extensão adicionada”, clique em Ativar. | If {browser} shows “New extension added”, click Enable. |
| `browsers.ext_outdated` | Extensão {version} · precisa da {required} ou mais nova | Extension {version} · needs {required} or newer |
| `browsers.ext_outdated_hint` | Reinicie o {browser} para atualizar. | Restart {browser} to update. |
| `browsers.host_missing` | O {browser} não encontra o app WebeSign | {browser} can't find the WebeSign app |
| `browsers.repair` | Reparar | Repair |
| `browsers.repaired` | Reparado. Reinicie o {browser}. | Repaired. Restart {browser}. |
| `browsers.safari_disabled` | Extensão desativada no Safari | Extension turned off in Safari |
| `browsers.safari_open_settings` | Abrir ajustes do Safari | Open Safari settings |
| `browsers.snap_hint` | Firefox instalado como Snap: na primeira assinatura, permita o acesso quando o sistema perguntar. | Firefox is installed as a Snap: on your first signature, allow access when the system asks. |
| `browsers.none` | Nenhum navegador compatível encontrado. O WebeSign funciona com Chrome, Edge, Firefox, Brave e Safari. | No supported browser found. WebeSign works with Chrome, Edge, Firefox, Brave and Safari. |
| `sites.section` | Sites com permissão | Allowed sites |
| `sites.row` | Lembrado em {date} · último uso {when} | Remembered on {date} · last used {when} |
| `sites.revoke` | Revogar | Revoke |
| `sites.revoke_confirm` | Confirmar revogação | Confirm revoke |
| `sites.revoked` | Permissão de {site} revogada. | {site} is no longer allowed. |
| `sites.empty` | Nenhum site lembrado. Quando você marcar “Lembrar este site” numa assinatura, ele aparece aqui. | No remembered sites. When you tick “Remember this site” while signing, it shows up here. |
| `devices.subtitle` | Tokens, cartões e drivers que o WebeSign enxerga agora. | Tokens, cards and drivers WebeSign can see right now. |
| `devices.section_tokens` | Tokens e cartões | Tokens and cards |
| `devices.section_readers` | Leitores de cartão | Card readers |
| `devices.section_drivers` | Drivers de token | Token drivers |
| `devices.certs_found` (one / other) | {count} certificado / {count} certificados | {count} certificate / {count} certificates |
| `devices.no_certs` | Sem certificados · instale o {driver} | No certificates · install {driver} |
| `devices.unknown_certs` | Não sabemos se tem certificados | We can't tell if it has certificates |
| `devices.generic_ccid` | Dispositivo de cartão inteligente | Smart card device |
| `devices.reader_empty` | Sem cartão | No card |
| `devices.reader_card` | Cartão: {card} | Card: {card} |
| `devices.reader_card_unknown` | Cartão não reconhecido | Unrecognized card |
| `devices.driver_loaded` (one / other) | Carregado · {count} token / Carregado · {count} tokens | Loaded · {count} token / Loaded · {count} tokens |
| `devices.driver_no_token` | Carregado · nenhum token conectado | Loaded · no token connected |
| `devices.driver_failed` | Não carregou: {reason} | Failed to load: {reason} |
| `devices.driver_auto` | Encontrado automaticamente | Found automatically |
| `devices.driver_user` | Adicionado por você | Added by you |
| `devices.driver_add` | Adicionar driver… | Add driver… |
| `devices.driver_add_help` | Use só se o fabricante do token pedir. | Only use this if your token vendor tells you to. |
| `devices.driver_remove` | Remover | Remove |
| `devices.pcscd` | Serviço de cartões (pcscd) | Card service (pcscd) |
| `devices.pcscd_ok` | Ativo | Running |
| `devices.pcscd_stopped` | Parado. Rode: {command} | Stopped. Run: {command} |
| `devices.empty` | Nenhum token ou leitor conectado. Conecte o token na porta USB; esta lista atualiza sozinha. | No token or reader connected. Plug your token into a USB port; this list updates by itself. |
| `certs_tab.subtitle` | Todos os certificados que este computador oferece, inclusive os que não servem para assinar. | Every certificate this computer offers, including the ones that can't sign. |
| `certs_tab.group_windows` | No Windows | In Windows |
| `certs_tab.group_macos` | No Keychain do Mac | In the Mac Keychain |
| `certs_tab.group_driver` | Pelo driver do token | Through the token driver |
| `certs_tab.hidden_group` | Não servem para assinar ({count}) | Can't sign ({count}) |
| `certs_tab.import` | Importar arquivo .pfx… | Import .pfx file… |
| `certs_tab.import_help_windows` | O Windows importa o arquivo e guarda o certificado com segurança. Você vai precisar da senha do arquivo. | Windows imports the file and stores the certificate securely. You'll need the file's password. |
| `certs_tab.import_help_macos` | O Acesso às Chaves importa o arquivo e guarda o certificado com segurança. Você vai precisar da senha do arquivo. | Keychain Access imports the file and stores the certificate securely. You'll need the file's password. |
| `certs_tab.import_linux` | No Linux, use o certificado em token ou cartão. | On Linux, use a certificate on a token or card. |
| `certs_tab.empty` | Nenhum certificado neste computador. Conecte o token, insira o cartão ou importe um arquivo .pfx. | No certificates on this computer. Plug in your token, insert your card or import a .pfx file. |
| `help.faq_title` | Perguntas comuns | Common questions |
| `help.q_missing` | Meu certificado não aparece | My certificate doesn't show up |
| `help.a_missing` | Confira: o token está conectado? O driver do fabricante está instalado (veja Dispositivos)? O certificado venceu (veja Certificados)? Se é um arquivo .pfx, ele foi importado? | Check: is the token plugged in? Is the vendor's driver installed (see Devices)? Has the certificate expired (see Certificates)? If it's a .pfx file, was it imported? |
| `help.q_pin` | Errei o PIN / o token bloqueou | I got the PIN wrong / the token locked |
| `help.a_pin` | Depois de algumas tentativas erradas, o token bloqueia para proteger você. Para desbloquear, use o PUK (código de desbloqueio que veio com o token) no programa do fabricante. Sem o PUK, procure a autoridade certificadora. | After a few wrong attempts the token locks to protect you. To unlock it, use the PUK (the unlock code that came with the token) in the vendor's software. Without the PUK, contact your certificate authority. |
| `help.q_privacy` | O que o WebeSign envia para os sites? | What does WebeSign send to sites? |
| `help.a_privacy` | Só o certificado que você escolheu e a assinatura do código de conferência. Nunca o PIN, nunca o documento. | Only the certificate you chose and the signature of the verification code. Never your PIN, never the document. |
| `help.q_shortcuts` | Atalhos de teclado | Keyboard shortcuts |
| `help.report_title` | Relatar um problema | Report a problem |
| `help.report_preview` | Isto é exatamente o que será copiado. Não inclui nomes, CPF, sites nem números de série. | This is exactly what will be copied. It includes no names, ID numbers, sites or serial numbers. |
| `help.open_support` | Abrir página de suporte | Open support page |
| `help.about_title` | Sobre | About |
| `help.about_line` | WebeSign {version} · protocolo {protocol} · GPL-3.0-or-later | WebeSign {version} · protocol {protocol} · GPL-3.0-or-later |
| `help.source` | Código-fonte | Source code |
| `help.privacy` | Privacidade | Privacy |

#### Popup da extensão

| Chave | pt-BR | en |
|-------|-------|----|
| `popup.checking` | Verificando… | Checking… |
| `popup.ready_title` | Tudo pronto para assinar | Ready to sign |
| `popup.ready_body` | O app WebeSign {version} está conectado a este navegador. | The WebeSign app {version} is connected to this browser. |
| `popup.open_diagnostics` | Abrir diagnóstico | Open diagnostics |
| `popup.missing_title` | Falta instalar o app WebeSign | The WebeSign app isn't installed |
| `popup.missing_body` | A extensão precisa do app no computador para assinar. | The extension needs the app on your computer to sign. |
| `popup.download` | Baixar para {os} | Download for {os} |
| `popup.activate` | Já instalei? Ativar o app | Already installed? Activate the app |
| `popup.outdated_title` | Atualize o app WebeSign | Update the WebeSign app |
| `popup.outdated_body` | Você tem a versão {installed}. Este navegador precisa da {required} ou mais nova. | You have version {installed}. This browser needs {required} or newer. |
| `popup.update_in` | Atualizar na {store} | Update in {store} |
| `popup.error_title` | O app não respondeu | The app didn't respond |
| `popup.error_body` | Tente de novo. Se continuar, reinicie o computador. | Try again. If it keeps happening, restart your computer. |
| `popup.retry` | Tentar de novo | Try again |
| `popup.redownload` | Baixar de novo | Download again |
| `popup.unsupported_title` | O WebeSign ainda não funciona neste sistema | WebeSign doesn't work on this system yet |
| `popup.unsupported_body` | Use um computador com Windows, macOS ou Linux. | Use a computer running Windows, macOS or Linux. |
| `popup.footer_versions` | Extensão {ext} · App {app} | Extension {ext} · App {app} |
| `popup.privacy` | Privacidade | Privacy |
| `store.microsoft` | Microsoft Store | Microsoft Store |
| `store.apple` | Mac App Store | Mac App Store |
| `store.linux` | página de download | download page |

Os textos de erro estão na [§15](#15-erros).

---

## 14. Acessibilidade

| Área | Regra |
|------|-------|
| AccessKit | Ligado sempre (feature `accesskit` do eframe). Todo widget tem papel e nome; botões só com ícone têm nome (`common.*`) e dica. Idioma do nó raiz = idioma da interface, para a voz certa. |
| Lista | Papel `RadioGroup` com nome "Assinar com"; cada linha `RadioButton` com nome completo: "Ana Beatriz Souza, ICP-Brasil A3, CPF parcialmente oculto 456 789, AC SOLUTI Multipla v5, Cartão no leitor, vence em 23 dias". Linha desabilitada continua focável, com `disabled` e o motivo no nome. |
| Regiões vivas | Educada: certificado encontrado/removido, código pronto, "Assinado". Assertiva: PIN incorreto, PIN bloqueado, erros. |
| Origem | A origem e seus alertas formam um único nó de texto lido ao abrir ("Pedido de assinatura de app.diagnos.health, pelo Google Chrome, site com permissão"; aqui com o nome completo do navegador). O chip "Site novo" é lido pelo nome acessível completo (`confirm.site_new_a11y`). |
| Contraste | AA em ambos os temas (tabela na §11.1); componentes e foco ≥ 3:1. |
| Foco | Anel `focus` de 2 px com 2 px de afastamento, em todo elemento focável, sempre visível ao navegar por teclado; nunca removido. |
| Teclado | Tudo operável sem mouse (§4.9, §8.8); sem armadilha de foco; Esc sempre sai. |
| Alvos | Mínimo 32 × 32 px (acima dos 24 × 24 do WCAG 2.2 AA); linhas de 72 px; botões de ícone com 20 px visuais têm área de 32. |
| Cor | Nenhum estado só por cor: semáforo tem forma de ícone e texto; validade tem texto; erro de campo tem mensagem. |
| Texto maior | Windows "Tornar o texto maior" multiplica os tamanhos de fonte; o corpo das janelas rola, então nada é cortado. |
| Movimento | §11.4. |
| Popup | HTML semântico: `<main>`, cartão de estado com `role="status"` e `aria-live="polite"`, `<button>` de verdade, foco inicial no botão primário, `:focus-visible` com o token `focus`. |
| Verificação | NVDA + Windows, VoiceOver + macOS, Orca + GNOME. `TODO(gustavo)`: confirmar que o AccessKit do egui expõe AT-SPI corretamente ao Orca antes de prometer Linux acessível. |

---

## 15. Erros

Espelham os erros tipados do SDK (`error.code`). Se a trilha do SDK usar outro nome, vale o do SDK e esta
tabela é ajustada. Regra de cancelamento: ao fechar a janela, o SDK recebe o código do **último bloqueio
visível** (`NoCertificates`, `PinLocked`, `CertificateUnavailable`); sem bloqueio, `UserCancelled`.

Onde aparece: **J** = Janela de Confirmação · **P** = popup · **S** = site (texto sugerido em
`site.errors.*`, exportado pelo SDK) · **D** = só para quem desenvolve (inglês, no `error.message`).

| Código | Quando | Onde | pt-BR (título — texto) | en (título — texto) | Ação |
|--------|--------|------|------------------------|---------------------|------|
| `ExtensionMissing` | A página não recebeu o anúncio da extensão | S | Instale a extensão WebeSign — Para assinar neste navegador, instale a extensão gratuita. | Install the WebeSign extension — To sign in this browser, install the free extension. | Botão com `installUrl()` |
| `AppMissing` | `connectNative` falhou | S, P | Instale o app WebeSign — A extensão precisa do app no computador para assinar. | Install the WebeSign app — The extension needs the app on your computer to sign. | Baixar · "Já instalei? Ativar o app" |
| `AppOutdated` | App < `MIN_APP_VERSION` ou protocolo antigo | S, P | Atualize o app WebeSign — Você tem a versão {installed}; é preciso a {required} ou mais nova. | Update the WebeSign app — You have version {installed}; {required} or newer is required. | Atualizar na loja |
| `ExtensionOutdated` | App exige protocolo mais novo que o da extensão | S | Atualize a extensão WebeSign — Reinicie o navegador para ela se atualizar. | Update the WebeSign extension — Restart your browser so it updates. | — |
| `InsecureOrigin` | Origem `http` não local, `file:`, etc. | S, D (J só por bug) | Site sem conexão segura — Este site não usa https. Por segurança, o WebeSign não assina para ele. | Insecure site — This site doesn't use https. For your safety, WebeSign won't sign for it. | — (D: "Call the SDK from an https origin or localhost.") |
| `UserCancelled` | Cancelar, Esc ou X sem bloqueio visível | S | Assinatura cancelada. | Signature cancelled. | O site decide |
| `Timeout` | 5 min sem decisão | J, S | O pedido expirou — Ninguém respondeu em 5 minutos. Volte ao site e tente de novo. | The request expired — Nobody answered for 5 minutes. Go back to the site and try again. | — |
| `NoCertificates` | Fechou a janela com a lista vazia | J (estado vazio), S | Nenhum certificado encontrado — Conecte o token ou insira o cartão e tente de novo. | No certificates found — Plug in your token or insert your card and try again. | Abrir diagnóstico |
| `CertificateUnavailable` | O certificado pedido/escolhido sumiu (token removido, chave apagada) | J, S | Certificado indisponível — O certificado escolhido não está mais acessível. Conecte o token de novo ou escolha outro. | Certificate unavailable — The chosen certificate is no longer available. Plug the token back in or choose another one. | Foco na lista |
| `CertificateNotValid` | O site pediu um certificado vencido ou ainda não válido | J (linha desabilitada), S | Certificado fora da validade — Este certificado venceu ou ainda não começou a valer. Escolha outro ou renove com a sua autoridade certificadora. | Certificate out of validity — This certificate has expired or isn't valid yet. Choose another one or renew it with your certificate authority. | Abrir diagnóstico |
| `InvalidRequest` | Digest com tamanho diferente do hash declarado, hash desconhecido, parâmetros inválidos | D | — | "Digest is 20 bytes; SHA-256 requires 32." | A janela não abre |
| `UnsupportedAlgorithm` | O algoritmo pedido não existe para a chave/driver (ex.: RSASSA-PSS num CSP antigo) | J, S | Este certificado não assina desse jeito — O site pediu {algorithm}, que este certificado ou driver não oferece. Escolha outro certificado. | This certificate can't sign that way — The site asked for {algorithm}, which this certificate or driver doesn't support. Choose another certificate. | Foco na lista; linha fica "Não compatível com este pedido" |
| `PinIncorrect` | PIN errado | J (não chega ao site) | ver `pin.incorrect*` (§13.2) | — | Campo limpo e focado |
| `PinLocked` | PIN bloqueado | J, S | PIN bloqueado — ver `pin.locked_body`. Site: "O PIN do seu token está bloqueado. Desbloqueie com o PUK e tente de novo." | PIN locked — see `pin.locked_body`. Site: "Your token PIN is locked. Unlock it with the PUK and try again." | Escolher outro certificado |
| `TokenRemoved` | Token saiu durante a assinatura | J, S | O token foi removido — Conecte o token de novo e clique em Tentar de novo. | The token was removed — Plug the token back in and click Try again. | Tentar de novo (habilita quando o token volta) |
| `DriverFailure` | Erro do CNG/CAPI/PKCS#11/Keychain que não é PIN nem cancelamento | J, S | O driver do token falhou — O {driver} não respondeu como esperado. Tente de novo; se continuar, reconecte o token. | The token driver failed — {driver} didn't respond as expected. Try again; if it keeps happening, reconnect the token. | Tentar de novo · "Tentar pelo driver do token" (se houver caminho alternativo, §5.11) · Abrir diagnóstico · Detalhes técnicos: `CKR_DEVICE_ERROR (0x00000030)`, `NTE_BAD_KEYSET (0x80090016)` |
| `Busy` | Fila com 10 pedidos | S | Há pedidos esperando confirmação — Conclua os pedidos abertos na janela do WebeSign. | Requests are waiting for confirmation — Finish the open requests in the WebeSign window. | — |
| `Internal` | Qualquer falha inesperada do app | J, S | Algo deu errado no WebeSign — Copie os detalhes e abra o diagnóstico para relatar. | Something went wrong in WebeSign — Copy the details and open diagnostics to report it. | Copiar detalhes · Abrir diagnóstico |

Chaves: `errors.<snake_code>.title` e `errors.<snake_code>.body` para a janela; `site.errors.<snake_code>.title`
e `.body` para o site; ações usam as chaves comuns.

Visual do erro na janela: aviso `danger-soft` com borda `danger-border`, `radius-lg`, padding 12/16, ícone
`x-circle` (fill) 20 px, título `text-body-strong`, texto `text-small`, "Detalhes técnicos" recolhido com o
código em `text-mono` e botão `copy`. Fica acima da lista, para o usuário poder escolher outro certificado.

---

## 16. Casos de teste de referência

Funções puras que os testes devem cobrir antes do código (TDD). "Hoje" nos casos de validade = 29/09/2026,
fuso local.

### 16.1 Código de conferência (`fingerprint`)

| Digest (primeiros bytes, hex) | `text` | `colorIndex` | Linhas do identicon (1 = acesa) |
|-------------------------------|--------|--------------|----------------------------------|
| `7F3A9C21E0B455D8…` | `7F3A 9C21 E0B4 55D8` | 3 | `00100 11011 01010 10101 11011` |
| SHA-256("") = `E3B0C44298FC1C14…` | `E3B0 C442 98FC 1C14` | 7 | `00100 00000 11011 00000 11011` |
| SHA-256("abc") = `BA7816BF8F01CFEA…` | `BA78 16BF 8F01 CFEA` | 5 | `01110 01010 00000 00100 11111` |
| 32 bytes zero | `0000 0000 0000 0000` | 0 | todas apagadas |

### 16.2 Origem (`format_origin`)

| Entrada | Destaque (registrável) | Restante apagado | Alerta | Pode lembrar |
|---------|------------------------|------------------|--------|--------------|
| `https://app.diagnos.health` | `diagnos.health` | `https://app.` | — | sim |
| `https://app.diagnos.health:443` | `diagnos.health` | `https://app.` (porta padrão oculta) | — | sim |
| `https://laudos.clinicasaolucas.med.br` | `clinicasaolucas.med.br` | `https://laudos.` | — | sim |
| `https://diagnos.health.cadastro-medico.com` | `cadastro-medico.com` | `https://diagnos.health.` | — | sim |
| `https://xn--dignos-4nf.health` | `xn--dignos-4nf.health` | `https://` | `idn` + "Aparece como diаgnos.health" | não |
| `https://192.168.0.20:8443` | `192.168.0.20` | `https://`, `:8443` | `ip_local` | não |
| `https://203.0.113.7` | `203.0.113.7` | `https://` | `ip` | não |
| `http://localhost:5173` | `localhost` | `http://`, `:5173` | `localhost` | sim |
| `http://laudos.exemplo.com` | — | — | bloqueado (`InsecureOrigin`) | — |

### 16.3 Nome do titular (`display_name`)

| CN | Resultado |
|----|-----------|
| `ANA BEATRIZ SOUZA:12345678909` | `Ana Beatriz Souza` |
| `JOAO DA SILVA DOS SANTOS:98765432100` | `Joao da Silva dos Santos` |
| `CLINICA SOUZA IMAGEM LTDA:12345678000190` | `Clinica Souza Imagem Ltda` |
| `E SOUZA COMERCIO ME` | `E Souza Comercio ME` (partícula no início fica maiúscula) |
| `Marta Sofia Carvalho` | `Marta Sofia Carvalho` (não está em caixa alta: mantém) |

### 16.4 Documento (`display_document`)

| Entrada | Resultado |
|---------|-----------|
| otherName 2.16.76.1.3.1 = `01021985` + `12345678909` + … | `CPF •••.456.789-••` |
| otherName 2.16.76.1.3.3 = `12345678000190` | `CNPJ 12.345.678/0001-90` |
| serialNumber `IDCPT-12345123` | `Documento •••••123` |
| nada | (sem documento) |

### 16.5 Validade (`validity_label`)

| notBefore → notAfter | Texto | Tom |
|----------------------|-------|-----|
| … → 30/10/2026 (31 dias) | Válido até 30/10/2026 | muted |
| … → 29/10/2026 (30 dias) | Vence em 30 dias | warning |
| … → 22/10/2026 | Vence em 23 dias | warning |
| … → 06/10/2026 (7 dias) | Vence em 7 dias | danger |
| … → 30/09/2026 | Vence amanhã | danger |
| … → 29/09/2026 23:59 | Vence hoje | danger |
| … → 10/05/2026 | Venceu em 10/05/2026 (desabilitado) | danger |
| 01/10/2026 → … | Válido a partir de 01/10/2026 (desabilitado) | warning |

Dias = diferença entre **datas do calendário local**, não períodos de 24 h.

### 16.6 Lista (`build_cert_list`)

Entrada: A3 da Ana pelo Windows **e** pelo driver (mesmo DER); A1 da Ana; A1 da clínica; A3 antigo vencido;
certificado de login do Cartão de Cidadão com irmão de assinatura; certificado sem chave privada.
Saída esperada: 3 utilizáveis (A3 pelo Windows com `also_via_driver`, A1 da Ana, A1 da clínica), 1 desabilitado
(A3 vencido), ocultos: login do Cartão de Cidadão e o sem chave. Com "último usado neste site" = A1 da Ana,
ele vem primeiro e selecionado; inserir um token novo durante a janela acrescenta a linha no fim sem mover a
seleção.

---

## 17. Pendências

- `TODO(gustavo)`: aprovar a API de `sign({ prepare })` e a semântica de `certificates()` (R1, R2) com a trilha do SDK.
- `TODO(gustavo)`: o Diagnos mostra o código de conferência e o identicon ao lado de "Aguardando confirmação" (R3).
- `TODO(gustavo)`: foco da janela no Windows quando o host já está rodando (§4.1) — incluir na prova 1.
- `TODO(gustavo)`: assinatura em lote — entra ou não, e com qual limite (§4.11).
- `TODO(gustavo)`: filtro por política pedido pelo site (ex.: só ICP-Brasil) (R6).
- `TODO(gustavo)`: conferir OIDs de política ICP-Brasil contra o DOC-ICP-04 vigente (§5.3).
- `TODO(gustavo)`: manter o token autenticado durante a sessão (sem `CKA_ALWAYS_AUTHENTICATE`) ou pedir PIN a cada assinatura (§4.6).
- `TODO(gustavo)`: caminho para A1 (.pfx) no Linux (§8.5).
- `TODO(gustavo)`: logo definitivo (hoje `seal-check` num quadrado `accent`).
- `TODO(gustavo)`: acessibilidade do egui no Linux com Orca (§14).
- `TODO(gustavo)`: textos pt-PT, es e fr (formato pronto, §13.1).
