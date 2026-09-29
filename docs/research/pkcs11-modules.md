# PKCS#11: onde estão os módulos e como o `websign-probe` os encontra

Pesquisa para a prova 4 (Linux) e para a lista de caminhos conhecidos do app. Cobre o registro no p11-kit
(formato, diretórios, regras de exclusão), os caminhos de instalação dos módulos por fabricante × sistema
operacional e o que ficou sem confirmação. A lista que o código usa está em
[`kit/probe/src/keystores/pkcs11/known_paths/`](../prototypes/kit/probe/src/keystores/pkcs11/known_paths/);
**toda linha daquela lista tem a fonte aqui.**

**Como ler a coluna "Base":**

| Sigla | Significa |
|---|---|
| **D** | documentação do fabricante ou do órgão emissor (abri e li o texto) |
| **L** | verificado nesta máquina (Ubuntu 24.04, pacotes `opensc`, `softhsm2`, `p11-kit` instalados) |
| **R** | repositório ou lista de terceiros que li (demoiselle/signer, NCryptoki, ArchWiki, OpenSC wiki) |
| **B** | guia de blog, fórum ou fabricante conhecido só pelo resumo de uma busca (não abri a página inteira) |
| **?** | sem fonte: é convenção do sistema, tratar como hipótese até a prova |

Nada aqui foi copiado. As listas de terceiros foram lidas só para conferir nomes e caminhos.

---

## 1. Resumo do que decide o projeto

1. **Só OpenSC e SoftHSM2 se registram no p11-kit** (e o `p11-kit-trust`, que só guarda CAs). Nenhum dos
   middlewares usados na ICP-Brasil (SafeSign, SafeNet, Watchdata, ePass2003) instala um arquivo `.module`.
   Sem a lista de caminhos conhecidos, o app não veria esses tokens por PKCS#11 (L, R).
2. **O caminho varia com a versão, a distribuição e a arquitetura** (`/usr/lib`, `/usr/lib64`, `/opt/…-<data>/`).
   Por isso cada produto tem **vários caminhos alternativos e só o primeiro que existe é carregado**, e o
   diagnóstico continua precisando do botão "adicionar módulo" (`--module`).
3. **O mesmo token chega por 2 ou 3 caminhos** (p11-kit registrado, caminho conhecido, `--module`, ou o
   `p11-kit-proxy.so`, que reexporta todos os módulos registrados). O app junta por **arquivo** (o mesmo
   inode, mesmo por link simbólico) antes de carregar e por **impressão digital do certificado** depois de
   listar. A prova 4 exercita as duas camadas (§5).
4. **Módulos de certificado em nuvem (BirdID, NeoID, DesktopID) ficam fora da lista.** São PKCS#11 locais que
   falam com um serviço remoto e podem abrir janelas ou rede só por serem carregados. Quem os usa adiciona o
   arquivo por `--module` (§4).
5. **Um processo de 64 bits só carrega módulo de 64 bits.** Middleware de 32 bits no Windows fica em
   `SysWOW64` e não está na lista de propósito; o erro de carga menciona a arquitetura (§6).
6. **O `libpcsclite.so.1` é dependência dinâmica do binário** (`ldd`), por causa do comando `devices`: em uma
   máquina sem pcsc-lite o host nem inicia, mesmo que só use PKCS#11. O pacote `.deb`/`.rpm` precisa declarar
   `libpcsclite1`/`pcsc-lite-libs` (§6).

---

## 2. Como o probe descobre os módulos

Ordem (código: `discovery.rs`), sempre com **uma carga por arquivo**:

1. `--module <caminho>` (repetível): sempre entra; arquivo ausente ou que não carrega **é avisado**.
2. Registrados no p11-kit (desligável com `--no-p11-kit`): arquivos `*.module` dos diretórios abaixo; arquivo
   ausente ou que não carrega **é avisado**, porque alguém (o pacote ou o usuário) o registrou.
3. Caminhos conhecidos por SO (desligável com `--no-known-modules`): só o **primeiro caminho que existe** de cada
   produto; ausente **não é avisado** (é só um palpite), mas presente e sem carregar **é avisado**, com o nome
   do produto na mensagem.

### 2.1 Formato e regras do p11-kit (D: [`pkcs11.conf(5)`](https://p11-glue.github.io/p11-glue/p11-kit/manual/pkcs11-conf.html))

```text
module: opensc-pkcs11.so        # relativo: vem do diretório de módulos do p11-kit
disable-in: p11-kit-proxy       # nomes de programa (nome-base do executável)
enable-in: seahorse, ssh        # lista de permissão; não usar junto com disable-in
trust-policy: yes               # fonte de política de confiança (só CAs)
remote: |ssh host p11-kit remote /caminho/módulo.so
```

| Regra | O que o probe faz |
|---|---|
| Diretórios lidos, do menor para o maior precedência | `/usr/share/p11-kit/modules`, `/etc/pkcs11/modules`, `$XDG_CONFIG_HOME/pkcs11/modules` (ou `~/.config/pkcs11/modules`). No macOS, também os prefixos do Homebrew (`/opt/homebrew`, `/usr/local`). Windows: nenhum. (L, D) |
| Mesmo nome de arquivo em mais de um diretório | vale o do diretório de maior precedência. Um `module:` **em branco** no diretório do usuário desliga o módulo do sistema (D) |
| `module:` relativo | procurado em `/usr/lib/<multiarch>/pkcs11`, `/usr/lib64/pkcs11`, `/usr/lib/pkcs11`, `/usr/local/lib/pkcs11` (e `/opt/homebrew/lib/pkcs11` no macOS). O p11-kit usa o diretório compilado nele (`pkg-config`); o probe não chama `pkg-config` (L) |
| `disable-in` / `enable-in` | comparados com `websign` e com o nome-base do executável atual (`websign-probe`, ou o nome do app depois). Cobre o caso do `p11-kit-proxy`: `disable-in: p11-kit-proxy` **não** exclui o probe (D) |
| `trust-policy: yes` ou `remote:` | ignorados: o módulo de confiança nunca tem chave privada, e `remote:` não tem arquivo para carregar |
| `pkcs11.conf` global (`user-config`, `managed`) | **não é lido.** Um módulo que o p11-kit marca como `critical` ou `managed: no` é tratado como os outros |

Verificado aqui (L): `/usr/share/p11-kit/modules/` tem `opensc-pkcs11.module`, `softhsm2.module` e
`p11-kit-trust.module` (este com `disable-in: p11-kit-proxy` e `trust-policy: yes`).

### 2.2 Padrões nos caminhos conhecidos

- `%VAR%`: variável de ambiente (Windows: `%SystemRoot%`, `%ProgramFiles%`, `%ProgramFiles(x86)%`). Variável
  ausente = o caminho não existe nesta máquina.
- Um `*` dentro de **um** nome de diretório, para instaladores que gravam a data no nome:
  `/opt/ePass2003-Castle-*/x64/redist/libcastle.so.1.0.0` (o mais novo, em ordem alfabética, vence).

---

## 3. Caminhos por produto

Cada célula lista os caminhos que o probe tenta, na ordem. **Confiança por linha** na última coluna.

| Produto (tokens) | Linux | macOS | Windows | p11-kit? | Base |
|---|---|---|---|---|---|
| **SafeNet Authentication Client** (Thales): eToken 5100/5110/5300 | `/usr/lib/libeToken.so`, `/usr/lib64/libeToken.so`, `/usr/lib/libeTPkcs11.so`, `/usr/lib64/libeTPkcs11.so`, `/usr/local/lib/libeTPkcs11.so` | `/usr/local/lib/libeTPkcs11.dylib`, `/Library/Frameworks/eToken.framework/Versions/A/libeToken.dylib` | `%SystemRoot%\System32\eTPKCS11.dll` | não | Linux: R [demoiselle], B [Synehan], B [Fedora], B [guillecro]. macOS: R [demoiselle], B [T1C]. Windows: R [demoiselle], R [NCryptoki], D [Nexus] |
| **IDPrime PKCS#11** (IDGo 800 / SAC 10.8R2+): eToken 5110 CC, IDPrime MD 840/940 | `/usr/lib/libIDPrimePKCS11.so`, `/usr/lib64/libIDPrimePKCS11.so`, `/usr/lib/pkcs11/libIDPrimePKCS11.so` | (sem caminho confirmado) | `%ProgramFiles%\SafeNet\Authentication\SAC\x64\IDPrimePKCS1164.dll`, `%ProgramFiles%\Gemalto\IDGo 800 PKCS#11\IDPrimePKCS1164.dll`, `%ProgramFiles(x86)%\Gemalto\IDGo 800 PKCS#11\IDPrimePKCS1164.dll` | não | Windows: D [Nexus]. Linux: B [python-pkcs11 #24], B [ubuntu-fr] |
| **SafeSign Identity Client** (A.E.T. Europe / G+D): StarSign Crypto USB Token, cartões YpsID/Cosmo da Certisign e Serasa | `/usr/lib/libaetpkss.so`, `/usr/lib/libaetpkss.so.3`, `/usr/lib64/libaetpkss.so` | `/usr/local/lib/libaetpkss.dylib`, `/Applications/tokenadmin.app/Contents/Frameworks/libaetpkss.dylib` | `%SystemRoot%\System32\aetpkss1.dll` | não. Guias sugerem criar `~/.config/pkcs11/modules/safesign.module` à mão | Linux: D [KPN SafeSign Linux], R [ArchWiki], B [gist SafeSign]; `lib64` é convenção (?). macOS: R [demoiselle], [3-tokens-mac.md](../prototypes/3-tokens-mac.md). Windows: D [KPN], R [demoiselle], R [NCryptoki] |
| **Watchdata ProxKey / WatchKey** (ICP-Brasil, SERPRO "token branco") | `/usr/lib/watchdata/ICP/lib/libwdpkcs_icp.so`, `/usr/lib/watchdata/lib/libwdpkcs.so`, `/opt/watchdata/lib64/libwdpkcs.so`, `/usr/local/lib64/libwdpkcs.so`, `/usr/local/lib/libwdpkcs.so`, `/usr/lib/libwdpkcs.so`, `/usr/lib/WatchData/ProxKey/lib/libwdpkcs_SignatureP11.so` | `/usr/local/lib/libwdpkcs.dylib`, `/usr/lib/libwdpkcs.dylib`, `/Applications/WatchKey USB Token Admin Tool.app/Contents/MacOS/lib/libWDP11_BR_GOV.dylib` | `%SystemRoot%\System32\Watchdata\Watchdata ICP CSP v1.0\WDPKCS.dll`, `…\Watchdata Brazil CSP v1.0\WDPKCS.dll`, `…\System32\WDPKCS.dll`, `…\WDICP_P11_CCID_v34.dll`, `…\SignatureP11.dll` (ProxKey) | não | Brasil: R [demoiselle] (commit e issue #24: o caminho "ICP CSP v1.0" substituiu o "Brazil CSP v1.0"). ProxKey: B [Vivaldi], B [Chilkat] |
| **Feitian ePass2003** ("Castle") | `/opt/ePass2003-Castle-*/x64/redist/libcastle.so.1.0.0`, `/usr/lib/libcastle.so.1.0.0`, `/usr/lib/libcastle_v2.so.1.0.0` | `/usr/local/lib/libcastle.1.0.0.dylib` | `%SystemRoot%\System32\eps2003csp11.dll` | não | Linux: R [demoiselle] (`/opt/ePass2003-Castle-20141128/i386/redist/…`), D [OpenSC #2424] (nomes `libcastle*.so.1.0.0`); o diretório `/usr/lib` é convenção (?). macOS: B [OpenSC wiki]. Windows: B [eps2003csp11]. **Alternativa:** o driver `epass2003` do OpenSC (D [OpenSC wiki ePass2003]) |
| **Feitian ePassNG** (ePass2000/3000) e **ePass3003** | `/usr/lib/libepsng_p11.so`, `/usr/local/ngsrv/libepsng_p11.so.1` | (sem caminho) | `%SystemRoot%\System32\ngp11v211.dll`, `…\ShuttleCsp11_3003.dll` | não | R [demoiselle], R [NCryptoki] |
| **OpenSC** (ePass2003, DNIe, PIV, cartões europeus) | `/usr/lib/x86_64-linux-gnu/pkcs11/opensc-pkcs11.so`, `/usr/lib/aarch64-linux-gnu/pkcs11/opensc-pkcs11.so`, `/usr/lib64/pkcs11/opensc-pkcs11.so`, `/usr/lib/pkcs11/opensc-pkcs11.so`, `/usr/lib64/opensc-pkcs11.so`, `/usr/lib/opensc-pkcs11.so`, `/usr/local/lib/opensc-pkcs11.so` | `/Library/OpenSC/lib/opensc-pkcs11.so`, `/usr/local/lib/opensc-pkcs11.so`, `/opt/homebrew/lib/opensc-pkcs11.so`, `/opt/homebrew/lib/pkcs11/opensc-pkcs11.so` | `%ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll` | **sim** (`opensc-pkcs11.module`) | Debian/Ubuntu: L. Arch: R [ArchWiki]. Fedora `lib64`: B [OpenSC #2424]. macOS: D [OpenSC macOS]; Homebrew é convenção (?). Windows: D [OpenSC Firefox] |
| **Cartão de Cidadão** (AMA, Portugal) | `/usr/local/lib/libpteidpkcs11.so`, `/usr/lib/libpteidpkcs11.so` | `/usr/local/lib/libpteidpkcs11.dylib` | `%SystemRoot%\System32\pteidpkcs11.dll` (64 bits; `SysWOW64` é o de 32) | não | D [Autenticação.Gov] (`/usr/lib` é convenção (?)) |
| **DNIe** (Policía Nacional, Espanha) | `/usr/lib/libpkcs11-dnie.so`, `/usr/lib64/libpkcs11-dnie.so`, `/opt/FNMTpkcs11dnie/lib/libpkcs11-dnie.so` | `/Library/Libpkcs11-dnie/lib/libpkcs11-dnie.so` | `%SystemRoot%\System32\DNIe_P11_x64.dll`, `…\DNIe_P11.dll`, `…\DNIe_P11_priv.dll` | não | Linux, macOS, FNMT: D [DNIe manual]; macOS também [3-tokens-mac.md](../prototypes/3-tokens-mac.md). Windows: B [FNMT Firefox Windows], B [Mozilla forum] |
| **Gemalto Classic Client** (GemSafe, Gemplus) | `/usr/lib/pkcs11/libgclib.so`, `/usr/lib/ClassicClient/libgclib.so` | (sem caminho) | `%SystemRoot%\System32\gclib.dll`, `%ProgramFiles%\Gemplus\GemSafe Libraries\BIN\gclib.dll` | não | Windows: R [demoiselle], R [NCryptoki]. Linux: R [libclassicclient], B [ubuntu-fr] |
| **Athena IDProtect** | `/usr/lib/x64-athena/libASEP11.so`, `/usr/lib/libASEP11.so`, `/lib64/libASEP11.so` | `/Library/Application Support/Athena/libASEP11.dylib`, `/usr/local/lib/libASEP11.dylib` | `%SystemRoot%\System32\asepkcs.dll` | não | Windows: R [demoiselle], R [NCryptoki]. Linux e macOS: B [webcrypto-local #211] |
| **Bit4id** (tokenME, Digital DNA, cartões CNS) | `/usr/lib/libbit4ipki.so`, `/usr/lib/libbit4xpki.so` | `/Library/bit4id/pkcs11/libbit4ipki.dylib`, `/Library/bit4id/pkcs11/libbit4xpki.dylib` | `%SystemRoot%\System32\bit4ipki.dll`, `…\bit4xpki.dll`, `…\bit4opki.dll` | não | Windows: R [NCryptoki], B [Mozilla Itália] (`bit4xpki` para cartões posteriores a fev/2016). Linux: B [Mozilla Itália]. macOS: B [AOC manual Bit4id] |
| **Yubico ykcs11** (YubiKey PIV) | `/usr/local/lib/libykcs11.so` | `/usr/local/lib/libykcs11.dylib`, `/opt/homebrew/lib/libykcs11.dylib` | `%ProgramFiles%\Yubico\Yubico PIV Tool\bin\libykcs11.dll` | não | D [YKCS11]. Homebrew é convenção (?). O macOS já expõe o PIV pelo CryptoTokenKit |
| **Certisign Cosmo / Oberthur AWP** | `/usr/local/AWP/lib/libOcsCryptoki.so` | (sem caminho) | `%SystemRoot%\System32\OcsCryptoki.dll` | não | R [demoiselle] |
| **SafeWeb / CIS (Charismathics)** | `/usr/lib/libcmP11.so` | (sem caminho) | `%SystemRoot%\System32\cmP11.dll` | não | R [demoiselle] |
| **SoftHSM2** (chaves de software, provas) | `/usr/lib/softhsm/libsofthsm2.so`, `/usr/lib/x86_64-linux-gnu/softhsm/libsofthsm2.so`, `/usr/lib/aarch64-linux-gnu/softhsm/libsofthsm2.so`, `/usr/lib64/softhsm/libsofthsm2.so`, `/usr/local/lib/softhsm/libsofthsm2.so` | `/opt/homebrew/lib/softhsm/libsofthsm2.so`, `/usr/local/lib/softhsm/libsofthsm2.so` | `C:\SoftHSM2\lib\softhsm2-x64.dll` | **sim** (`softhsm2.module`) | Linux: L. macOS: D [cryptoki] (`/usr/local/lib/softhsm/`); Homebrew é convenção (?). Windows: instalador do SoftHSM2 (?) |

### 3.1 Módulos especiais do p11-kit (L)

| Arquivo | Para que serve | O probe |
|---|---|---|
| `p11-kit-proxy.so` (link para `libp11-kit.so.0`) | reexporta **todos** os módulos registrados como um só módulo | não é carregado sozinho; funciona com `--module` e lista e assina normalmente (§5) |
| `p11-kit-trust.so` | CAs do sistema (centenas de certificados, nenhuma chave privada) | nunca carregado (`trust-policy: yes`) |
| `p11-kit-client.so` | cliente do módulo remoto (`p11-kit remote`) | não é carregado; módulos `remote:` são ignorados |
| `onepin-opensc-pkcs11.so`, `pkcs11-spy.so` | OpenSC com um só PIN (cartão de identidade da Estônia); registro de chamadas | fora da lista; usar `--module` |

---

## 4. Fora da lista, de propósito

| Módulo | Por que não |
|---|---|
| BirdID (`vault-pkcs11`), SERPRO NeoID (`SerproPkcs11.dll`, `libneoidp11`), Certisign DesktopID (`desktopID_Provider.dll`) | PKCS#11 de **certificado em nuvem**: falam com um serviço remoto e podem abrir janela ou rede ao carregar. Fora do escopo do produto ([R] demoiselle). `--module` os aceita |
| Tokens brasileiros sem caminho público | **Dexon DXToken** (`0483:a389`): o middleware DXSafe traz "biblioteca PKCS#11" mas nenhum guia público cita o nome do arquivo. **Longmai mToken CryptoID**: idem. Não achei; ficam para `--module` até um token real |
| `libsoftokn3` (NSS) e módulos do Firefox | são repositórios de software do navegador, não tokens |

---

## 5. O que a prova 4 mostrou

Detalhes e saída em [4-linux.md](../prototypes/4-linux.md). Resumo das conclusões que dependem deste documento:

- **O p11-kit registrado é descoberto** (um `.module` no diretório do usuário apontando para outra cópia do
  SoftHSM2) e **um registro que aponta para lugar nenhum é só um aviso**, não derruba os outros módulos.
- **O mesmo certificado por dois módulos aparece uma vez**, com `+1 other path(s)`: SoftHSM2 direto mais
  `p11-kit-proxy.so`, nas duas ordens de carga. O proxy assina sozinho (4 certificados, 15 assinaturas verificadas).
- **Carregar o SoftHSM2 direto e pelo proxy no mesmo processo funciona:** o segundo `C_Initialize` responde
  `CKR_CRYPTOKI_ALREADY_INITIALIZED`, que o probe trata como sucesso.

---

## 6. Riscos e pontos de atenção

| Risco | Efeito | O que fazer |
|---|---|---|
| `cryptoki` 0.12 exige dados bem formados: um token com `CKF_CLOCK_ON_TOKEN` cuja hora não seja dígitos faz `C_GetTokenInfo` falhar na conversão | o slot inteiro some da lista (a mensagem diz "the module returned malformed data") | medir com os tokens reais; se ocorrer, ler `CK_TOKEN_INFO` cru com `cryptoki-sys` |
| `cryptoki` 0.12 não tem `C_Sign` sem `C_SignInit` | chaves com `CKA_ALWAYS_AUTHENTICATE` (assinatura qualificada do Cartão de Cidadão, DNIe e do cartão da Estônia) precisam do login de contexto **entre** os dois. Com o `cryptoki` só sobra o multi-part, que o OpenSC aceita e o SoftHSM2 recusa | ver [4-linux.md §5](../prototypes/4-linux.md#5-o-que-ficou-sem-prova). Com `cryptoki-sys = "0.5"` a chamada crua assinou as 6 combinações de uma chave RSA `--always-auth` no SoftHSM2 |
| `libpcsclite.so.1` é dependência dinâmica | sem pcsc-lite o binário não inicia | `Depends: libpcsclite1` no `.deb`; no `.rpm`, `pcsc-lite-libs`. Alternativa: carregar a biblioteca em tempo de execução |
| Dois módulos do mesmo fornecedor (ex.: `libeToken.so` e `libeTPkcs11.so`) iniciados no mesmo processo | podem disputar o mesmo serviço; a impressão digital junta a lista, mas o `C_Initialize` duplicado do fornecedor não é garantido | só o primeiro caminho existente de cada produto entra; se um registro do p11-kit apontar para outro, o probe carrega os dois |
| p11-kit "gerenciado" | o p11-kit avisa que módulos usados por mais de uma biblioteca do mesmo processo pedem coordenação (`C_Initialize` uma vez) | o app não carrega outra biblioteca que use p11-kit; nunca chamamos `C_Finalize` |
| PIN esperando o usuário dentro do módulo | módulos com diálogo próprio (Cartão de Cidadão, DNIe) bloqueiam `C_Sign` até a pessoa responder; cancelar devolve `CKR_FUNCTION_REJECTED` | mapeado para `Cancelled`; a confirmação do app precisa avisar "digite o PIN na janela do driver" |
| 32 × 64 bits | `wrong ELF class` (Linux), erro 193 (Windows), `incompatible architecture` (macOS) | a mensagem de erro cita a arquitetura |

---

## 7. Como acrescentar um módulo

1. Achar o caminho **em uma fonte** (manual do fabricante, guia da autoridade certificadora, repositório) e
   anotar a base (D/R/B/?) na tabela acima.
2. Uma linha em `known_paths/<sistema>.rs` (um produto por linha; caminhos alternativos na ordem em que devem
   ser tentados). Os testes do módulo recusam caminho relativo, duplicado ou com mais de um `*`.
3. Se o token é novo, o `devices.json` ganha a entrada com `match.usb`/`match.atr` e o mesmo caminho em
   `driver.pkcs11` ([tokens.md](tokens.md)).

---

## Fontes

Todas acessadas em 29/09/2026.

**Documentação e arquivos que li por inteiro (D, L):**

- p11-kit, [`pkcs11.conf(5)`](https://p11-glue.github.io/p11-glue/p11-kit/manual/pkcs11-conf.html): campos `module`, `enable-in`, `disable-in`, `trust-policy`, `remote`; nome-base do executável; `module:` em branco.
- Esta máquina: `/usr/share/p11-kit/modules/*.module`, `/usr/lib/x86_64-linux-gnu/pkcs11/`, `/usr/lib/softhsm/`.
- [OpenSC wiki: macOS Quick Start](https://github.com/OpenSC/OpenSC/wiki/macOS-Quick-Start) (`/Library/OpenSC/lib/opensc-pkcs11.so`, cópias em `/usr/local/lib`); [OpenSC wiki: Feitian ePass2003](https://github.com/OpenSC/OpenSC/wiki/Feitian-ePass2003).
- [Manual de Utilização da Autenticação.Gov](https://amagovpt.github.io/docs.autenticacao.gov/user_manual.html) (Cartão de Cidadão: `pteidpkcs11.dll`, `libpteidpkcs11.so`, `libpteidpkcs11.dylib`).
- [Manual de instalação do Multicard PKCS11 DNIe](https://www.dnielectronico.es/PDFs/manuales_instalacion_unix/Manual_de_Instalacion_de_MulticardPKCS11_DNIE.pdf) (Linux, macOS e `/opt/FNMTpkcs11dnie`); [página de instaladores do DNIe](https://www.dnielectronico.es/PortalDNIe/PRF1_Cons02.action?pag=REF_1112).
- [Nexus: middleware Gemalto/SafeNet/Thales](https://doc.nexusgroup.com/pub/encoding-using-gemalto-safenet-thales-middleware-i) (caminhos do IDPrimePKCS11 e `eTPKCS11.dll`; observação de que `eTPKCS11.dll` não atende o slot de assinatura do IDPrime 940).
- [YKCS11 (Yubico)](https://developers.yubico.com/yubico-piv-tool/YKCS11/).
- [SafeSign IC Standard 4.1 para Linux (KPN)](http://certificaat.kpn.com/files/drivers/SafeSign/SafeSign%20IC%20Standard%20Version%204.1%20for%20Linux%20Release%20Document.pdf) ("libaetpkss.so … /usr/lib/").

**Listas e repositórios de terceiros (R):**

- [demoiselle/signer, commit `d6728df`](https://github.com/demoiselle/signer/commit/d6728dfa6a2b5b8ea45dc65a1a1c1931dd94d4a6): mapa de drivers de tokens brasileiros (Windows, Linux, macOS); [issue #24](https://github.com/demoiselle/signer/issues/24) (novo caminho da Watchdata); [issue #413](https://github.com/demoiselle/signer/issues/413).
- [NCryptoki: Known PKCS#11 modules](http://wiki.ncryptoki.com/Known-PKCS-11-modules.ashx): nomes de DLL por fabricante.
- [ArchWiki: Electronic identification](https://wiki.archlinux.org/title/Electronic_identification): SafeSign para ICP-Brasil, `/usr/lib/opensc-pkcs11.so`, `p11-kit-proxy.so` no Firefox.
- [suscerte/libclassicclient](https://github.com/suscerte/libclassicclient/blob/master/usr/lib/pkcs11/libgclib.so).

**Guias e resumos de busca (B):**

- SafeNet Linux: [Synehan/safenet-linux](https://github.com/Synehan/safenet-linux), [eToken 5110 no Fedora](https://sztsian.github.io/2022/02/21/Using-Safenet-eToken-5110-With-Fedora.html), [gist do Ubuntu 24.04](https://gist.github.com/guillecro/595dcc62893bf4a7fc4a0f3c405a50e2), [T1C-Java Guide: SafeNet PKCS #11](https://t1t.gitbooks.io/t1c-java-guide/containers/safenet.html) (macOS e Linux).
- SafeSign no Firefox/Linux para ICP-Brasil: [gist](https://gist.github.com/jonasmalacofilho/a5fcd493b779d6d435c0); [safesign-distrobox](https://github.com/HenriKCorrea/safesign-distrobox) (`safesign.module`).
- IDPrime no Linux: [python-pkcs11 #24](https://github.com/danni/python-pkcs11/issues/24), [Ubuntu-fr](https://forum.ubuntu-fr.org/viewtopic.php?id=2034468).
- Watchdata ProxKey: [Vivaldi Forum](https://forum.vivaldi.net/topic/84092/security-device-token-support), [Chilkat: PROXKey Token](https://cknotes.com/proxkey-token/).
- Feitian: [OpenSC #2424](https://github.com/OpenSC/OpenSC/issues/2424) (`libcastle.so.1.0.0`, `libcastle_v2.so.1.0.0`), [eps2003csp11-interface](https://github.com/melgmry0101b/eps2003csp11-interface) (Windows).
- [OpenSC wiki: OpenSC no Firefox](https://github.com/OpenSC/OpenSC/wiki/Installing-OpenSC-PKCS11-Module-in-Firefox,-Step-by-Step) (Windows).
- Athena: [webcrypto-local #211](https://github.com/PeculiarVentures/webcrypto-local/issues/211). Bit4id: [Mozilla Itália](https://forum.mozillaitalia.org/index.php?topic=49911.0), [manual Bit4id para macOS (AOC)](https://suport.aoc.cat/en-US/article/?servei=tcat&id=KA-06988_manual-de-bit4id-per-a-mac-os).
- DNIe no Windows: [FNMT: módulo no Firefox](https://www.cert.fnmt.es/documents/10445900/10528353/instalacion_modulos_criptograficos_firefox_windows.pdf/9fd42709-db1f-4281-98b4-1458405ce14d?version=1.2), [fórum do Mozilla](https://support.mozilla.org/es/questions/1227358).
