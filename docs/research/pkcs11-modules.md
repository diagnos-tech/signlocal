# PKCS#11: where the modules are and how `websign-probe` finds them

Research for proof 4 (Linux) and for the app's list of known paths. It covers p11-kit registration
(format, directories, exclusion rules), the modules' installation paths by vendor × operating
system, and what remains unconfirmed. The list the code uses is in
[`kit/probe/src/keystores/pkcs11/known_paths/`](../prototypes/kit/probe/src/keystores/pkcs11/known_paths/);
**every line of that list has its source here.**

**How to read the "Basis" column:**

| Code | Meaning |
|---|---|
| **D** | documentation from the vendor or issuing body (opened and read the text) |
| **L** | verified on this machine (Ubuntu 24.04, `opensc`, `softhsm2`, and `p11-kit` packages installed) |
| **R** | third-party repository or list that I read (demoiselle/signer, NCryptoki, ArchWiki, OpenSC wiki) |
| **B** | blog guide, forum, or vendor known only from a search summary (did not open the full page) |
| **?** | no source: it is a system convention, treat it as a hypothesis until proven |

Nothing here was copied. Third-party lists were read only to check names and paths.

---

## 1. Summary of what decides the project

1. **Only OpenSC and SoftHSM2 register with p11-kit** (and `p11-kit-trust`, which only stores CAs). None of the
   middlewares used in ICP-Brasil (SafeSign, SafeNet, Watchdata, ePass2003) installs a `.module` file.
   Without the list of known paths, the app would not see these tokens through PKCS#11 (L, R).
2. **The path varies with version, distribution, and architecture** (`/usr/lib`, `/usr/lib64`, `/opt/…-<date>/`).
   That is why each product has **several alternative paths and only the first one that exists is loaded**, and
   diagnostics still need the "add module" button (`--module`).
3. **The same token arrives through 2 or 3 paths** (registered p11-kit, known path, `--module`, or
   `p11-kit-proxy.so`, which re-exports every registered module). The app merges by **file** (the same
   inode, even through a symlink) before loading and by **certificate thumbprint** after
   listing. Proof 4 exercises both layers (§5).
4. **Cloud certificate modules (BirdID, NeoID, DesktopID) are left off the list.** They are local PKCS#11 modules that
   talk to a remote service and may open windows or the network just by being loaded. Whoever uses them adds the
   file through `--module` (§4).
5. **A 64-bit process only loads 64-bit modules.** 32-bit middleware on Windows lives in
   `SysWOW64` and is deliberately not on the list; the load error mentions the architecture (§6).
6. **`libpcsclite.so.1` is a dynamic dependency of the binary** (`ldd`), because of the `devices` command: on a
   machine without pcsc-lite the host does not even start, even if it only uses PKCS#11. The `.deb`/`.rpm` package must declare
   `libpcsclite1`/`pcsc-lite-libs` (§6).

---

## 2. How the probe discovers modules

Order (code: `discovery.rs`), always with **one load per file**:

1. `--module <path>` (repeatable): always included; a missing file, or one that does not load, **is reported**.
2. Registered in p11-kit (can be turned off with `--no-p11-kit`): `*.module` files from the directories below; a missing
   file, or one that does not load, **is reported**, because someone (the package or the user) registered it.
3. Known paths per OS (can be turned off with `--no-known-modules`): only the **first path that exists** of each
   product; a missing one **is not reported** (it is just a guess), but a present one that fails to load **is reported**, with the product
   name in the message.

### 2.1 p11-kit format and rules (D: [`pkcs11.conf(5)`](https://p11-glue.github.io/p11-glue/p11-kit/manual/pkcs11-conf.html))

```text
module: opensc-pkcs11.so        # relative: comes from p11-kit's module directory
disable-in: p11-kit-proxy       # program names (executable base name)
enable-in: seahorse, ssh        # allow list; do not use together with disable-in
trust-policy: yes               # trust policy source (CAs only)
remote: |ssh host p11-kit remote /path/module.so
```

| Rule | What the probe does |
|---|---|
| Directories read, from lowest to highest precedence | `/usr/share/p11-kit/modules`, `/etc/pkcs11/modules`, `$XDG_CONFIG_HOME/pkcs11/modules` (or `~/.config/pkcs11/modules`). On macOS, also the Homebrew prefixes (`/opt/homebrew`, `/usr/local`). Windows: none. (L, D) |
| Same file name in more than one directory | the one from the highest-precedence directory wins. A **blank** `module:` in the user directory turns off the system module (D) |
| Relative `module:` | looked up in `/usr/lib/<multiarch>/pkcs11`, `/usr/lib64/pkcs11`, `/usr/lib/pkcs11`, `/usr/local/lib/pkcs11` (and `/opt/homebrew/lib/pkcs11` on macOS). p11-kit uses the directory compiled into it (`pkg-config`); the probe does not call `pkg-config` (L) |
| `disable-in` / `enable-in` | compared with `websign` and with the current executable's base name (`websign-probe`, or the app's name later). Covers the `p11-kit-proxy` case: `disable-in: p11-kit-proxy` does **not** exclude the probe (D) |
| `trust-policy: yes` or `remote:` | ignored: the trust module never has a private key, and `remote:` has no file to load |
| Global `pkcs11.conf` (`user-config`, `managed`) | **not read.** A module that p11-kit marks as `critical` or `managed: no` is treated like the others |

Verified here (L): `/usr/share/p11-kit/modules/` has `opensc-pkcs11.module`, `softhsm2.module`, and
`p11-kit-trust.module` (the latter with `disable-in: p11-kit-proxy` and `trust-policy: yes`).

### 2.2 Patterns in the known paths

- `%VAR%`: environment variable (Windows: `%SystemRoot%`, `%ProgramFiles%`, `%ProgramFiles(x86)%`). A missing
  variable = the path does not exist on this machine.
- A `*` inside **one** directory name, for installers that write the date into the name:
  `/opt/ePass2003-Castle-*/x64/redist/libcastle.so.1.0.0` (the newest, in alphabetical order, wins).

---

## 3. Paths per product

Each cell lists the paths the probe tries, in order. **Confidence per row** in the last column.

| Product (tokens) | Linux | macOS | Windows | p11-kit? | Basis |
|---|---|---|---|---|---|
| **SafeNet Authentication Client** (Thales): eToken 5100/5110/5300 | `/usr/lib/libeToken.so`, `/usr/lib64/libeToken.so`, `/usr/lib/libeTPkcs11.so`, `/usr/lib64/libeTPkcs11.so`, `/usr/local/lib/libeTPkcs11.so` | `/usr/local/lib/libeTPkcs11.dylib`, `/Library/Frameworks/eToken.framework/Versions/A/libeToken.dylib` | `%SystemRoot%\System32\eTPKCS11.dll` | no | Linux: R [demoiselle], B [Synehan], B [Fedora], B [guillecro]. macOS: R [demoiselle], B [T1C]. Windows: R [demoiselle], R [NCryptoki], D [Nexus] |
| **IDPrime PKCS#11** (IDGo 800 / SAC 10.8R2+): eToken 5110 CC, IDPrime MD 840/940 | `/usr/lib/libIDPrimePKCS11.so`, `/usr/lib64/libIDPrimePKCS11.so`, `/usr/lib/pkcs11/libIDPrimePKCS11.so` | (no confirmed path) | `%ProgramFiles%\SafeNet\Authentication\SAC\x64\IDPrimePKCS1164.dll`, `%ProgramFiles%\Gemalto\IDGo 800 PKCS#11\IDPrimePKCS1164.dll`, `%ProgramFiles(x86)%\Gemalto\IDGo 800 PKCS#11\IDPrimePKCS1164.dll` | no | Windows: D [Nexus]. Linux: B [python-pkcs11 #24], B [ubuntu-fr] |
| **SafeSign Identity Client** (A.E.T. Europe / G+D): StarSign Crypto USB Token, Certisign and Serasa YpsID/Cosmo cards | `/usr/lib/libaetpkss.so`, `/usr/lib/libaetpkss.so.3`, `/usr/lib64/libaetpkss.so` | `/usr/local/lib/libaetpkss.dylib`, `/Applications/tokenadmin.app/Contents/Frameworks/libaetpkss.dylib` | `%SystemRoot%\System32\aetpkss1.dll` | no. Guides suggest creating `~/.config/pkcs11/modules/safesign.module` by hand | Linux: D [KPN SafeSign Linux], R [ArchWiki], B [gist SafeSign]; `lib64` is convention (?). macOS: R [demoiselle], [3-tokens-mac.md](../prototypes/3-tokens-mac.md). Windows: D [KPN], R [demoiselle], R [NCryptoki] |
| **Watchdata ProxKey / WatchKey** (ICP-Brasil, SERPRO "white token") | `/usr/lib/watchdata/ICP/lib/libwdpkcs_icp.so`, `/usr/lib/watchdata/lib/libwdpkcs.so`, `/opt/watchdata/lib64/libwdpkcs.so`, `/usr/local/lib64/libwdpkcs.so`, `/usr/local/lib/libwdpkcs.so`, `/usr/lib/libwdpkcs.so`, `/usr/lib/WatchData/ProxKey/lib/libwdpkcs_SignatureP11.so` | `/usr/local/lib/libwdpkcs.dylib`, `/usr/lib/libwdpkcs.dylib`, `/Applications/WatchKey USB Token Admin Tool.app/Contents/MacOS/lib/libWDP11_BR_GOV.dylib` | `%SystemRoot%\System32\Watchdata\Watchdata ICP CSP v1.0\WDPKCS.dll`, `…\Watchdata Brazil CSP v1.0\WDPKCS.dll`, `…\System32\WDPKCS.dll`, `…\WDICP_P11_CCID_v34.dll`, `…\SignatureP11.dll` (ProxKey) | no | Brazil: R [demoiselle] (commit and issue #24: the "ICP CSP v1.0" path replaced "Brazil CSP v1.0"). ProxKey: B [Vivaldi], B [Chilkat] |
| **Feitian ePass2003** ("Castle") | `/opt/ePass2003-Castle-*/x64/redist/libcastle.so.1.0.0`, `/usr/lib/libcastle.so.1.0.0`, `/usr/lib/libcastle_v2.so.1.0.0` | `/usr/local/lib/libcastle.1.0.0.dylib` | `%SystemRoot%\System32\eps2003csp11.dll` | no | Linux: R [demoiselle] (`/opt/ePass2003-Castle-20141128/i386/redist/…`), D [OpenSC #2424] (`libcastle*.so.1.0.0` names); the `/usr/lib` directory is convention (?). macOS: B [OpenSC wiki]. Windows: B [eps2003csp11]. **Alternative:** OpenSC's `epass2003` driver (D [OpenSC wiki ePass2003]) |
| **Feitian ePassNG** (ePass2000/3000) and **ePass3003** | `/usr/lib/libepsng_p11.so`, `/usr/local/ngsrv/libepsng_p11.so.1` | (no path) | `%SystemRoot%\System32\ngp11v211.dll`, `…\ShuttleCsp11_3003.dll` | no | R [demoiselle], R [NCryptoki] |
| **OpenSC** (ePass2003, DNIe, PIV, European cards) | `/usr/lib/x86_64-linux-gnu/pkcs11/opensc-pkcs11.so`, `/usr/lib/aarch64-linux-gnu/pkcs11/opensc-pkcs11.so`, `/usr/lib64/pkcs11/opensc-pkcs11.so`, `/usr/lib/pkcs11/opensc-pkcs11.so`, `/usr/lib64/opensc-pkcs11.so`, `/usr/lib/opensc-pkcs11.so`, `/usr/local/lib/opensc-pkcs11.so` | `/Library/OpenSC/lib/opensc-pkcs11.so`, `/usr/local/lib/opensc-pkcs11.so`, `/opt/homebrew/lib/opensc-pkcs11.so`, `/opt/homebrew/lib/pkcs11/opensc-pkcs11.so` | `%ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll` | **yes** (`opensc-pkcs11.module`) | Debian/Ubuntu: L. Arch: R [ArchWiki]. Fedora `lib64`: B [OpenSC #2424]. macOS: D [OpenSC macOS]; Homebrew is convention (?). Windows: D [OpenSC Firefox] |
| **Cartão de Cidadão** (AMA, Portugal) | `/usr/local/lib/libpteidpkcs11.so`, `/usr/lib/libpteidpkcs11.so` | `/usr/local/lib/libpteidpkcs11.dylib` | `%SystemRoot%\System32\pteidpkcs11.dll` (64-bit; `SysWOW64` is the 32-bit one) | no | D [Autenticação.Gov] (`/usr/lib` is convention (?)) |
| **DNIe** (Policía Nacional, Spain) | `/usr/lib/libpkcs11-dnie.so`, `/usr/lib64/libpkcs11-dnie.so`, `/opt/FNMTpkcs11dnie/lib/libpkcs11-dnie.so` | `/Library/Libpkcs11-dnie/lib/libpkcs11-dnie.so` | `%SystemRoot%\System32\DNIe_P11_x64.dll`, `…\DNIe_P11.dll`, `…\DNIe_P11_priv.dll` | no | Linux, macOS, FNMT: D [DNIe manual]; macOS also [3-tokens-mac.md](../prototypes/3-tokens-mac.md). Windows: B [FNMT Firefox Windows], B [Mozilla forum] |
| **Gemalto Classic Client** (GemSafe, Gemplus) | `/usr/lib/pkcs11/libgclib.so`, `/usr/lib/ClassicClient/libgclib.so` | (no path) | `%SystemRoot%\System32\gclib.dll`, `%ProgramFiles%\Gemplus\GemSafe Libraries\BIN\gclib.dll` | no | Windows: R [demoiselle], R [NCryptoki]. Linux: R [libclassicclient], B [ubuntu-fr] |
| **Athena IDProtect** | `/usr/lib/x64-athena/libASEP11.so`, `/usr/lib/libASEP11.so`, `/lib64/libASEP11.so` | `/Library/Application Support/Athena/libASEP11.dylib`, `/usr/local/lib/libASEP11.dylib` | `%SystemRoot%\System32\asepkcs.dll` | no | Windows: R [demoiselle], R [NCryptoki]. Linux and macOS: B [webcrypto-local #211] |
| **Bit4id** (tokenME, Digital DNA, CNS cards) | `/usr/lib/libbit4ipki.so`, `/usr/lib/libbit4xpki.so` | `/Library/bit4id/pkcs11/libbit4ipki.dylib`, `/Library/bit4id/pkcs11/libbit4xpki.dylib` | `%SystemRoot%\System32\bit4ipki.dll`, `…\bit4xpki.dll`, `…\bit4opki.dll` | no | Windows: R [NCryptoki], B [Mozilla Italia] (`bit4xpki` for cards after Feb 2016). Linux: B [Mozilla Italia]. macOS: B [AOC manual Bit4id] |
| **Yubico ykcs11** (YubiKey PIV) | `/usr/local/lib/libykcs11.so` | `/usr/local/lib/libykcs11.dylib`, `/opt/homebrew/lib/libykcs11.dylib` | `%ProgramFiles%\Yubico\Yubico PIV Tool\bin\libykcs11.dll` | no | D [YKCS11]. Homebrew is convention (?). macOS already exposes PIV through CryptoTokenKit |
| **Certisign Cosmo / Oberthur AWP** | `/usr/local/AWP/lib/libOcsCryptoki.so` | (no path) | `%SystemRoot%\System32\OcsCryptoki.dll` | no | R [demoiselle] |
| **SafeWeb / CIS (Charismathics)** | `/usr/lib/libcmP11.so` | (no path) | `%SystemRoot%\System32\cmP11.dll` | no | R [demoiselle] |
| **SoftHSM2** (software keys, proofs) | `/usr/lib/softhsm/libsofthsm2.so`, `/usr/lib/x86_64-linux-gnu/softhsm/libsofthsm2.so`, `/usr/lib/aarch64-linux-gnu/softhsm/libsofthsm2.so`, `/usr/lib64/softhsm/libsofthsm2.so`, `/usr/local/lib/softhsm/libsofthsm2.so` | `/opt/homebrew/lib/softhsm/libsofthsm2.so`, `/usr/local/lib/softhsm/libsofthsm2.so` | `C:\SoftHSM2\lib\softhsm2-x64.dll` | **yes** (`softhsm2.module`) | Linux: L. macOS: D [cryptoki] (`/usr/local/lib/softhsm/`); Homebrew is convention (?). Windows: SoftHSM2 installer (?) |

### 3.1 Special p11-kit modules (L)

| File | What it is for | The probe |
|---|---|---|
| `p11-kit-proxy.so` (link to `libp11-kit.so.0`) | re-exports **all** registered modules as a single module | not loaded on its own; works with `--module` and lists and signs normally (§5) |
| `p11-kit-trust.so` | system CAs (hundreds of certificates, no private key) | never loaded (`trust-policy: yes`) |
| `p11-kit-client.so` | client of the remote module (`p11-kit remote`) | not loaded; `remote:` modules are ignored |
| `onepin-opensc-pkcs11.so`, `pkcs11-spy.so` | OpenSC with a single PIN (Estonian ID card); call logging | off the list; use `--module` |

---

## 4. Off the list, on purpose

| Module | Why not |
|---|---|
| BirdID (`vault-pkcs11`), SERPRO NeoID (`SerproPkcs11.dll`, `libneoidp11`), Certisign DesktopID (`desktopID_Provider.dll`) | **cloud certificate** PKCS#11: they talk to a remote service and may open a window or the network on load. Outside the product's scope ([R] demoiselle). `--module` accepts them |
| Brazilian tokens with no public path | **Dexon DXToken** (`0483:a389`): the DXSafe middleware ships a "PKCS#11 library" but no public guide cites the file name. **Longmai mToken CryptoID**: same. Not found; they are left to `--module` until a real token |
| `libsoftokn3` (NSS) and Firefox modules | they are browser software stores, not tokens |

---

## 5. What proof 4 showed

Details and output in [4-linux.md](../prototypes/4-linux.md). Summary of the conclusions that depend on this document:

- **Registered p11-kit is discovered** (a `.module` in the user directory pointing to another copy of
  SoftHSM2) and **a registration that points nowhere is only a warning**; it does not take down the other modules.
- **The same certificate through two modules appears once**, with `+1 other path(s)`: SoftHSM2 directly plus
  `p11-kit-proxy.so`, in both load orders. The proxy signs on its own (4 certificates, 15 signatures verified).
- **Loading SoftHSM2 directly and through the proxy in the same process works:** the second `C_Initialize` answers
  `CKR_CRYPTOKI_ALREADY_INITIALIZED`, which the probe treats as success.

---

## 6. Risks and points of attention

| Risk | Effect | What to do |
|---|---|---|
| `cryptoki` 0.12 requires well-formed data: a token with `CKF_CLOCK_ON_TOKEN` whose time is not digits makes `C_GetTokenInfo` fail in conversion | the whole slot vanishes from the list (the message says "the module returned malformed data") | measure with real tokens; if it happens, read the raw `CK_TOKEN_INFO` with `cryptoki-sys` |
| `cryptoki` 0.12 has no `C_Sign` without `C_SignInit` | keys with `CKA_ALWAYS_AUTHENTICATE` (qualified signature of the Cartão de Cidadão, DNIe, and the Estonian card) need the context login **between** the two. With `cryptoki` only multi-part remains, which OpenSC accepts and SoftHSM2 rejects | see [4-linux.md §5](../prototypes/4-linux.md#5-what-remains-unproven). With `cryptoki-sys = "0.5"` the raw call signed all 6 combinations of an `--always-auth` RSA key on SoftHSM2 |
| `libpcsclite.so.1` is a dynamic dependency | without pcsc-lite the binary does not start | `Depends: libpcsclite1` in the `.deb`; in the `.rpm`, `pcsc-lite-libs`. Alternative: load the library at runtime |
| Two modules from the same vendor (e.g. `libeToken.so` and `libeTPkcs11.so`) started in the same process | they may fight over the same service; the thumbprint merges the list, but the vendor's duplicate `C_Initialize` is not guaranteed | only the first existing path of each product is included; if a p11-kit registration points to another, the probe loads both |
| "Managed" p11-kit | p11-kit warns that modules used by more than one library in the same process need coordination (`C_Initialize` once) | the app does not load another library that uses p11-kit; we never call `C_Finalize` |
| PIN waiting for the user inside the module | modules with their own dialog (Cartão de Cidadão, DNIe) block `C_Sign` until the person answers; cancelling returns `CKR_FUNCTION_REJECTED` | mapped to `Cancelled`; the app's confirmation must say "enter the PIN in the driver window" |
| 32 × 64 bits | `wrong ELF class` (Linux), error 193 (Windows), `incompatible architecture` (macOS) | the error message names the architecture |

---

## 7. How to add a module

1. Find the path **in a source** (vendor manual, certificate authority guide, repository) and
   note the basis (D/R/B/?) in the table above.
2. One line in `known_paths/<system>.rs` (one product per line; alternative paths in the order they should
   be tried). The module's tests reject relative paths, duplicates, and paths with more than one `*`.
3. If the token is new, `devices.json` gets the entry with `match.usb`/`match.atr` and the same path in
   `driver.pkcs11` ([tokens.md](tokens.md)).

---

## Sources

All accessed on 2026-09-29.

**Documentation and files I read in full (D, L):**

- p11-kit, [`pkcs11.conf(5)`](https://p11-glue.github.io/p11-glue/p11-kit/manual/pkcs11-conf.html): `module`, `enable-in`, `disable-in`, `trust-policy`, `remote` fields; executable base name; blank `module:`.
- This machine: `/usr/share/p11-kit/modules/*.module`, `/usr/lib/x86_64-linux-gnu/pkcs11/`, `/usr/lib/softhsm/`.
- [OpenSC wiki: macOS Quick Start](https://github.com/OpenSC/OpenSC/wiki/macOS-Quick-Start) (`/Library/OpenSC/lib/opensc-pkcs11.so`, copies in `/usr/local/lib`); [OpenSC wiki: Feitian ePass2003](https://github.com/OpenSC/OpenSC/wiki/Feitian-ePass2003).
- [Autenticação.Gov User Manual](https://amagovpt.github.io/docs.autenticacao.gov/user_manual.html) (Cartão de Cidadão: `pteidpkcs11.dll`, `libpteidpkcs11.so`, `libpteidpkcs11.dylib`).
- [Multicard PKCS11 DNIe installation manual](https://www.dnielectronico.es/PDFs/manuales_instalacion_unix/Manual_de_Instalacion_de_MulticardPKCS11_DNIE.pdf) (Linux, macOS, and `/opt/FNMTpkcs11dnie`); [DNIe installers page](https://www.dnielectronico.es/PortalDNIe/PRF1_Cons02.action?pag=REF_1112).
- [Nexus: Gemalto/SafeNet/Thales middleware](https://doc.nexusgroup.com/pub/encoding-using-gemalto-safenet-thales-middleware-i) (IDPrimePKCS11 and `eTPKCS11.dll` paths; note that `eTPKCS11.dll` does not serve the IDPrime 940 signing slot).
- [YKCS11 (Yubico)](https://developers.yubico.com/yubico-piv-tool/YKCS11/).
- [SafeSign IC Standard 4.1 for Linux (KPN)](http://certificaat.kpn.com/files/drivers/SafeSign/SafeSign%20IC%20Standard%20Version%204.1%20for%20Linux%20Release%20Document.pdf) ("libaetpkss.so … /usr/lib/").

**Third-party lists and repositories (R):**

- [demoiselle/signer, commit `d6728df`](https://github.com/demoiselle/signer/commit/d6728dfa6a2b5b8ea45dc65a1a1c1931dd94d4a6): driver map for Brazilian tokens (Windows, Linux, macOS); [issue #24](https://github.com/demoiselle/signer/issues/24) (Watchdata's new path); [issue #413](https://github.com/demoiselle/signer/issues/413).
- [NCryptoki: Known PKCS#11 modules](http://wiki.ncryptoki.com/Known-PKCS-11-modules.ashx): DLL names by vendor.
- [ArchWiki: Electronic identification](https://wiki.archlinux.org/title/Electronic_identification): SafeSign for ICP-Brasil, `/usr/lib/opensc-pkcs11.so`, `p11-kit-proxy.so` in Firefox.
- [suscerte/libclassicclient](https://github.com/suscerte/libclassicclient/blob/master/usr/lib/pkcs11/libgclib.so).

**Guides and search summaries (B):**

- SafeNet Linux: [Synehan/safenet-linux](https://github.com/Synehan/safenet-linux), [eToken 5110 on Fedora](https://sztsian.github.io/2022/02/21/Using-Safenet-eToken-5110-With-Fedora.html), [Ubuntu 24.04 gist](https://gist.github.com/guillecro/595dcc62893bf4a7fc4a0f3c405a50e2), [T1C-Java Guide: SafeNet PKCS #11](https://t1t.gitbooks.io/t1c-java-guide/containers/safenet.html) (macOS and Linux).
- SafeSign in Firefox/Linux for ICP-Brasil: [gist](https://gist.github.com/jonasmalacofilho/a5fcd493b779d6d435c0); [safesign-distrobox](https://github.com/HenriKCorrea/safesign-distrobox) (`safesign.module`).
- IDPrime on Linux: [python-pkcs11 #24](https://github.com/danni/python-pkcs11/issues/24), [Ubuntu-fr](https://forum.ubuntu-fr.org/viewtopic.php?id=2034468).
- Watchdata ProxKey: [Vivaldi Forum](https://forum.vivaldi.net/topic/84092/security-device-token-support), [Chilkat: PROXKey Token](https://cknotes.com/proxkey-token/).
- Feitian: [OpenSC #2424](https://github.com/OpenSC/OpenSC/issues/2424) (`libcastle.so.1.0.0`, `libcastle_v2.so.1.0.0`), [eps2003csp11-interface](https://github.com/melgmry0101b/eps2003csp11-interface) (Windows).
- [OpenSC wiki: OpenSC in Firefox](https://github.com/OpenSC/OpenSC/wiki/Installing-OpenSC-PKCS11-Module-in-Firefox,-Step-by-Step) (Windows).
- Athena: [webcrypto-local #211](https://github.com/PeculiarVentures/webcrypto-local/issues/211). Bit4id: [Mozilla Italia](https://forum.mozillaitalia.org/index.php?topic=49911.0), [Bit4id manual for macOS (AOC)](https://suport.aoc.cat/en-US/article/?servei=tcat&id=KA-06988_manual-de-bit4id-per-a-mac-os).
- DNIe on Windows: [FNMT: module in Firefox](https://www.cert.fnmt.es/documents/10445900/10528353/instalacion_modulos_criptograficos_firefox_windows.pdf/9fd42709-db1f-4281-98b4-1458405ce14d?version=1.2), [Mozilla forum](https://support.mozilla.org/es/questions/1227358).
