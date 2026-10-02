# Tokens and readers: Brazil, Portugal, Spain, and Latin America

Seed for `devices.json` (data: USB `VID:PID` and ATR → model → driver, with per-OS links). Every row
has its source. The fields follow what the interface consumes ([ux.md R7](../ux.md#r7-devicesjson-fields-the-interface-consumes)):
`match.usb`, `match.atr`, `driver.name`, `driver.download`, `driver.pkcs11`. The PKCS#11 module paths
of each middleware, with their sources, are in [pkcs11-modules.md](pkcs11-modules.md).

**`devices.json` only helps, it never blocks:** "we detected a SafeNet eToken 5110 with no certificate → install
SafeNet Authentication Client". A device that is not listed here keeps working through the normal path.

**How to read the "Basis" column:**

| Code | Meaning |
|---|---|
| **C** | list of CCID readers and tokens in `libccid` 1.5.5 (`/usr/lib/pcsc/drivers/ifd-ccid.bundle/Contents/Info.plist`, installed on this machine; project: [ccid.apdu.fr](https://ccid.apdu.fr/)) |
| **U** | USB ID database [`usb.ids`](https://raw.githubusercontent.com/vcrhonek/hwdata/master/usb.ids) (hwdata) |
| **A** | public ATR list of [pcsc-tools](https://raw.githubusercontent.com/LudovicRousseau/pcsc-tools/master/smartcard_list.txt) (`smartcard_list.txt`, 2002–2023) |
| **D** | official page of the manufacturer, certificate authority, or government (opened and read) |
| **B** | search summary or third-party guide (did not open the full page) |
| **?** | not found in a public source: needs a real device |

**ATR:** each pattern comes from pcsc-tools; `..` matches any byte. In `[12]0` the first digit is 1 or 2. The same
ATR can belong to more than one product (the chip is the same, the personalization changes): the ATR identifies the
**card family**, not the certificate. Some cards' ATR may contain chip bytes; it never goes into a
user report, only into the list.

---

## 1. Brazil (ICP-Brasil A3: USB token and card)

| Device | USB `VID:PID` | ATR (examples) | Middleware | Official download | Basis |
|---|---|---|---|---|---|
| **SafeNet eToken 5110 / 5100** (Thales) | `0529:0620` (`usb.ids` calls it "Token JC"; `libccid`, "eToken 5100"; Linux reports, "eToken 5110 SC") · 5110+ FIPS `08e6:34cf` · 5300 `08e6:34cc` | 5110 SC: `3B FF 96 00 00 81 31 FE 43 80 31 80 65 B0 85 59 56 FB 12 0F FE 82 90 00 00` · 5110+ FIPS: `3B FF 96 00 00 81 31 FE 43 80 31 80 65 B0 84 65 66 FB 12 01 78 82 90 00 85` · 5110 CC (IDPrime 940C): `3B 7F 96 00 00 80 31 80 65 B0 85 05 00 39 12 0F FE 82 90 00` · eToken PRO Java: `3B D5 18 00 81 31 3A 7D 80 73 C8 21 10 30` | **SafeNet Authentication Client** (SAC). The 5110 CC has a second, signing slot, visible only through `IDPrimePKCS11` | [SAC (Thales)](https://cpl.thalesgroup.com/access-management/security-applications/authentication-client-token-management), [eToken 5110](https://cpl.thalesgroup.com/access-management/authenticators/pki-usb-authentication/etoken-5110-usb-token). The installer is on Thales' support portal; CAs distribute copies, and the CA's version is the one the physician uses | USB: C, U, B ([linux-hardware](https://linux-hardware.org/index.php?id=usb:0529-0620), [Gentoo report](https://www.bananas-playground.net/2020/06/aladdin-etoken-safenet-gentoo-and-cisco/)). ATR: A. Page: D |
| **G+D StarSign Crypto USB Token** / **SafeSign** cards | `1059:0017` · StarSign CUT S `1059:0019` | StarSign USB Token: `3B FD 18 00 00 81 31 FE 45 53 43 45 36 30 2D 43 43 30 38 31 2D 46 C2` · StarSign Token: `3B F8 18 00 00 80 31 FE 45 00 73 C8 40 13 00 90 00 92` · SafeSign: `3B 74 18 00 00 73 66 74 65` | **SafeSign Identity Client** (A.E.T. Europe / G+D) | [SafeSign (G+D Brasil): Windows, Mac, and Linux](https://safesign.gdamericadosul.com.br/download) | USB: C. ATR: A. Page: D |
| **Feitian ePass2003** | `096e:0807` · ePass2003Auto `096e:080a` | `3B 9F 95 81 31 FE 9F 00 66 46 53 05 01 00 11 71 DF 00 00 .. .. .. ..` | Feitian's **ePass2003 "Castle"** driver, the Windows minidriver (Windows Update), or **OpenSC**'s `epass2003` driver | [Feitian PKI Standard](https://www.ftsafe.com/Products/PKI/Standard), [Resources](https://www.ftsafe.com/Support/Resources) | USB: C, U. ATR: A. Links: D/B |
| **Watchdata ProxKey / WatchKey** (SERPRO "white token") | `163c:0407`, `163c:0417`, `163c:0418` · `163c:0406` ("WatchCNPC USB CCID Key") · `163c:0a03` (W5181) | `3B 6E 00 00 57 44 36 19 69 86 93 02 ..` (several series: `…02 33 11 75 42 33 1E` is SERPRO's white token) · `3B 6D 00 00 57 44 36 41 01 86 93 ..` (ProxKey) | **Watchdata ICP** / WatchKey Admin Tool | no stable manufacturer link found (`watchdata.com/brazil/…` returns 404); use the CA's or SERPRO's link. `TODO(gustavo)` | USB: C. ATR: A. Link: ? |
| **Dexon DXToken** (made in Brazil) | `0483:a389` · eSmartDX `0483:a40b` | `3B 6B 00 00 80 5A 44 58 54 4F 4B 45 4E 30 31` · `3B 6E 00 00 80 31 08 72 14 22 57 44 58 54 4B 4E 30 31` | **DXSafe** (Windows 7+, macOS 10.10+, Linux) | [Dexon downloads](https://www.dexon.ind.br/downloads) ("Drivers do DX-Safe" tab) | USB: C. ATR: A. Link: B |
| **Athena IDProtect Key v2** | `0dc3:0900` | `3B D5 18 FF 81 91 FE 1F C3 80 73 C8 21 13 09` | **IDProtect Client** | [product page (Athena)](http://www.athena-scs.com/product.asp?pid=33), cited by pcsc-tools; may be out of date | USB: C. ATR: A. Link: ? |
| CAs' **A3 cards** | (CCID reader, see §5) | e-CPF Gemalto TOP DL v2 (Imprensa Oficial, Caixa): `3B 7D 96 00 00 80 31 80 65 B0 83 11 11 E5 83 00 90 00` · e-CNPJ Certisign (Sagem YpsID): `3B 7D 18 00 02 80 57 59 50 53 49 44 30 33 83 7F 90 00` · e-CPF Safeweb (Morpho YpsID): `3B 7D 18 00 02 80 57 59 50 53 49 44 30 34 83 7F 90 00` · e-CNPJ Certisign (Oberthur Cosmo v7): `3B DD 18 00 81 31 FE 45 80 F9 A0 00 00 00 77 01 08 00 07 90 00 FE` · Serasa: `3B 3B F7 18 00 00 80 31 FE 45 73 66 74 65 2D` · e-CPF: `3B 68 00 00 00 73 C8 40 12 00 90 00` | SafeSign (YpsID and G+D cards), Gemalto Classic Client / IDPrime (TOP DL), Oberthur AWP | see the rows above | ATR: A |

---

## 2. Portugal

| Device | USB `VID:PID` | ATR (examples) | Middleware | Official download | Basis |
|---|---|---|---|---|---|
| **Cartão de Cidadão** (AMA) | (CCID reader, see §5) | 1st generation: `3B 7D 95 00 00 80 31 80 65 B0 83 11 .. .. 83 00 90 00` · earlier: `3B 6B 00 00 00 31 C0 64 08 04 61 12 0F 90 00` · 2nd generation: `3B FF 96 00 00 81 31 FE 43 80 31 80 65 B0 85 04 01 20 12 0F FF 82 90 00 D0` | **Autenticação.Gov** (`pteidpkcs11` module; on macOS also `PteidToken` for CryptoTokenKit). Cards issued since Jun 2024 use **ECDSA** | [Autenticação.Gov application (Windows, macOS, Linux)](https://www.autenticacao.gov.pt/web/guest/cc-aplicacao) · [manual](https://amagovpt.github.io/docs.autenticacao.gov/user_manual.html) | ATR: A. Links and module: D |

The CC has a key with `CKA_ALWAYS_AUTHENTICATE` (signing PIN on every use) and the module asks for the PIN in its
own window: see [pkcs11-modules.md §6](pkcs11-modules.md#6-risks-and-points-of-attention).

---

## 3. Spain

| Device | USB `VID:PID` | ATR (examples) | Middleware | Official download | Basis |
|---|---|---|---|---|---|
| **DNIe** (Policía Nacional) | (CCID reader, see §5) | DNIe 1.0/2.0: `3B 7F 38 00 00 00 6A 44 4E 49 65 [12]0 02 4C 34 01 13 03 90 00` · DNI 3.0: `3B 7F 96 00 00 00 6A 44 4E 49 65 20 01 01 55 04 10 03 90 00` · DNIe 4.0: `3B 88 80 01 E1 F3 5E 11 77 81 00 00 A2` | **DNIe PKCS#11 module** (`libpkcs11-dnie`) or **OpenSC** | [DNIe: installers](https://www.dnielectronico.es/PortalDNIe/PRF1_Cons02.action?pag=REF_1112) (at the time: `.deb` for Debian 13/Ubuntu 25.04 and 32-bit Debian 10, `.rpm` for Fedora 38/openSUSE 15.6; Windows and macOS in the same area) · [module manual](https://www.dnielectronico.es/PDFs/manuales_instalacion_unix/Manual_de_Instalacion_de_MulticardPKCS11_DNIE.pdf) | ATR: A. Links: D |
| **FNMT-RCM** (CERES cryptographic card) | (CCID reader) | `3B 7F 96 00 00 00 6A 46 4E 4D 54 03 04 11 43 04 30 03 90 00` · CERES series: `3B EF .. 00 40 14 80 25 43 45 52 45 53 57 .. .. 01 01 03 90 00` | FNMT-DNIe PKCS#11 module (`/opt/FNMTpkcs11dnie/lib/libpkcs11-dnie.so`) | [FNMT module manual](https://www.sede.fnmt.gob.es/documents/10445900/10528353/Manual_de_Instalacion_de_MulticardPKCS11_FNMT_DNIE.pdf) | ATR: A. Manual: B |

---

## 4. Latin America

In the five countries I researched, the dominant device is the **SafeNet eToken 5110** (same line as in Brazil, same
SAC). What changes is who sells it and what the official list is.

| Country | Devices cited | Middleware and download | Basis |
|---|---|---|---|
| **Argentina** (firma digital, AFIP/ONTI) | SafeNet eToken 5110+ (FIPS 140-2 level 2/3) and mToken CryptoID (FIPS 140-3 level 3) | SafeNet Authentication Client; install before use ([buying and usage guide](https://mendohard.com.ar/token-firma-digital-cual-comprar-argentina/), [stores](https://sitepro.com.ar/web/productos/firma-digital/token-onti-safenet-5110/)). The eToken 5110+ FIPS has `08e6:34cf` | B |
| **Chile** (firma electrónica avanzada) | SafeNet eToken 5110 and 5110+ (Certinet, E-Sign, Acepta, E-CertChile) | SAC 10.8 (`SAC_10_8.exe`); older versions do not support the 5110+ ([Acepta](https://asistencia.acepta.com/firma-avanzada.html), [Certinet](https://www.certinet.cl/firma-avanzada)). The chip ID card (RUT) has ATR `3B 88 80 01 31 CC CC 01 77 83 A1 00 6C` | B; ATR: A |
| **Colombia** (Certicámara) | "E-Token SafeNet" and "E-Token Feitian" (Windows 10/11, 64-bit) | [Certicámara download center](https://web.certicamara.com/soporte/centro-de-descargas) | D |
| **Ecuador** (BCE, Security Data, Uanataca, Judicatura, Registro Civil) | each entity's tokens; Uanataca uses Bit4id's middleware | [drivers for tokens (firmadigital.gob.ec)](https://www.firmadigital.gob.ec/drivers-para-tokens/): links from each entity, including the [Bit4id middleware](https://cdn.bit4id.com/es/middleware.htm). On Linux, install `pcsc-lite`/`pcscd` first | D |
| **Peru** (RENIEC, DNIe 2.0/3.0 and tokens) | approved: Bit4id (IAm, CryptoKey, tokenME v1–v3, Digital DNA, J-Sign), SafeNet (iKey 4000, eToken 5100, eToken PRO Java, eToken 5110), Feitian (ePass 2003, AudioPass, BePass 2003), Athena IDProtect Key LASER, Sagem ypsID, Longmai mToken CryptoID, ACS (ACR101I, ACOS5-64, CryptoMate Nano) | [approved devices](https://pki.reniec.gob.pe/dcdelivery3/dispositivos-homologados.html) · [DNIe driver guide](https://identidad.reniec.gob.pe/documents/d/guest/guia_dnie_version3). Reader with cross-platform PKCS#11: ACS ACR39U (`072f:b100`) | D |

Bit4id (`25dd:…`): IAm `2221`, CryptoKey `1201`, CKey4 `2321`, tokenME FIPS v3 `2341`, tokenME EVO v2 `2371`,
Digital DNA `2351`/`2354`/`2362` (C). tokenME ATR: `3B 9F 11 81 31 FE 9F 00 6B 42 49 54 34 49 44 20 33 2E 30 00 90 00 CF` (A).
Longmai mToken CryptoID: ATR `3B 9F 11 81 31 FE 9F 00 6A 6D 54 6F 6B 65 6E 2D 46 00 00 81 90 00 79` (A); **`VID:PID` not found** (?).
Mexico (SAT's e.firma) uses `.cer`/`.key` files, not a token: out of scope.

---

## 5. Common readers (CCID)

Any CCID reader works (`pcscd` uses `libccid`); these are the ones that appear in certificate authority
guides and in stores in the countries above. IDs from the CCID list (C).

| Reader | `VID:PID` |
|---|---|
| Gemalto PC Twin Reader | `08e6:3437` |
| Gemalto USB GemPCPinpad (with PIN keypad) | `08e6:3478` |
| Gemalto GemPC Express | `08e6:34ec` |
| OMNIKEY CardMan 3121 | `076b:3021` |
| OMNIKEY CardMan 5321 | `076b:5321` |
| ACS ACR38U-CCID | `072f:90cc` |
| ACS ACR39U | `072f:b100` |
| SCM SCR 3310 | `04e6:5116` |
| Identiv uTrust 2900 R / 2910 R | `04e6:5811` / `04e6:5812` |
| Cherry SmartTerminal ST-2xxx | `046a:003e` |
| Bit4id miniLector-s / miniLector EVO | `25dd:1101` / `25dd:3111` |
| Feitian R502 / iR301 | `096e:060d` / `096e:0619` |

Other frequent devices: YubiKey 5 (PIV) OTP+U2F+CCID `1050:0407` and CCID `1050:0404` (U); YubiKey 4
ATR: `3B F8 13 00 00 81 31 FE 15 59 75 62 69 6B 65 79 34 D4` (A).

---

## 6. How this becomes `devices.json`

Example with the fields the interface consumes, filled in only with what has a source above:

```jsonc
{
  "id": "safenet-etoken-5110",
  "name": "SafeNet eToken 5110",
  "kind": "token",
  "match": {
    "usb": ["0529:0620"],
    // ATR in uppercase hex without separators; pcsc-tools' "..", becomes "??" (any one byte)
    "atr": ["3BFF9600008131FE4380318065B0855956FB120FFE82900000"]
  },
  "driver": {
    "name": "SafeNet Authentication Client",
    "download": {
      "windows": "https://cpl.thalesgroup.com/access-management/security-applications/authentication-client-token-management",
      "macos": "https://cpl.thalesgroup.com/access-management/security-applications/authentication-client-token-management",
      "linux": "https://cpl.thalesgroup.com/access-management/security-applications/authentication-client-token-management"
    },
    "pkcs11": { "windows": "eTPKCS11.dll", "macos": "/usr/local/lib/libeTPkcs11.dylib", "linux": "/usr/lib/libeToken.so" }
  }
}
```

Rules for the conversion:

1. **License before bulk import.** `devices.json` is CC0. The pcsc-tools ATR list is GPL-2.0+, libccid's
   `Info.plist` is LGPL-2.1+, and `usb.ids` is BSD/GPL. Numbers (`VID:PID`, ATR bytes) and the
   commercial name are facts; **their descriptions and groupings are not**. Import only the facts and write the
   descriptions, or ask Ludovic Rousseau for permission. `TODO(gustavo)`.
2. One `match.usb` per device (a list, because the same product has several revisions); the ATR only identifies
   cards inside generic readers, so it is `match.atr`.
3. pcsc-tools' `..` ATR becomes a one-byte wildcard in `match`; the app compares in uppercase hex without separators
   (the `websign-probe devices` format).
4. `driver.pkcs11` uses the same first path as [known_paths](../prototypes/kit/probe/src/keystores/pkcs11/known_paths/).
   Where the manufacturer documents no path (Dexon, Longmai), the field stays empty: diagnostics keep
   offering "add module".
5. No certificate detected and a known device → driver suggestion; **never** block the list.

## 7. Gaps (need a real device)

- Stable download link for **Watchdata** (WatchKey/ProxKey) and **Athena**; `TODO(gustavo)`: ask the CA/SERPRO.
- PKCS#11 file name for the **DXToken** and the **mToken CryptoID**, and the mToken's `VID:PID`.
- Whether the 5110 **without** "CC" (JavaCard, `0529:0620`) and the **5110 CC** (IDPrime 940) use the same `libeToken.so` on Linux
  or whether the CC requires `libIDPrimePKCS11.so`. Nexus's observation (`eTPKCS11.dll` does not expose the signing slot)
  suggests the CC needs the second one.
- ATRs of the **real** ICP-Brasil tokens of each CA (CAs change chip supplier per batch): the
  pcsc-tools list is a start, and `websign-probe devices` on each token confirms.

## How to update

```sh
curl -O https://raw.githubusercontent.com/LudovicRousseau/pcsc-tools/master/smartcard_list.txt
curl -O https://raw.githubusercontent.com/vcrhonek/hwdata/master/usb.ids
# libccid: /usr/lib/pcsc/drivers/ifd-ccid.bundle/Contents/Info.plist (ifdVendorID, ifdProductID, ifdFriendlyName)
websign-probe devices --all-usb          # what the machine sees
```

All pages were accessed on 2026-09-29. `github.com` is blocked in the research environment; the raw files
came from `raw.githubusercontent.com`.
