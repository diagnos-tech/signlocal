# Tokens e leitores: Brasil, Portugal, Espanha e América Latina

Semente do `devices.json` (dados: USB `VID:PID` e ATR → modelo → driver, com links por sistema). Cada linha
tem a fonte. Os campos seguem o que a interface consome ([ux.md R7](../ux.md#r7-campos-de-devicesjson-que-a-interface-consome)):
`match.usb`, `match.atr`, `driver.name`, `driver.download`, `driver.pkcs11`. Os caminhos do módulo PKCS#11
de cada middleware, com a fonte deles, estão em [pkcs11-modules.md](pkcs11-modules.md).

**O `devices.json` só ajuda, nunca bloqueia:** "detectamos um SafeNet eToken 5110 sem certificado → instale o
SafeNet Authentication Client". Um dispositivo que não está aqui continua funcionando pelo caminho normal.

**Como ler a coluna "Base":**

| Sigla | Significa |
|---|---|
| **C** | lista de leitores e tokens CCID do `libccid` 1.5.5 (`/usr/lib/pcsc/drivers/ifd-ccid.bundle/Contents/Info.plist`, instalado nesta máquina; projeto: [ccid.apdu.fr](https://ccid.apdu.fr/)) |
| **U** | banco de IDs USB [`usb.ids`](https://raw.githubusercontent.com/vcrhonek/hwdata/master/usb.ids) (hwdata) |
| **A** | lista pública de ATRs do [pcsc-tools](https://raw.githubusercontent.com/LudovicRousseau/pcsc-tools/master/smartcard_list.txt) (`smartcard_list.txt`, 2002–2023) |
| **D** | página oficial do fabricante, da autoridade certificadora ou do governo (abri e li) |
| **B** | resumo de busca ou guia de terceiros (não abri a página inteira) |
| **?** | não achei em fonte pública: precisa de um dispositivo real |

**ATR:** cada padrão vem do pcsc-tools; `..` casa um byte qualquer. Em `[12]0` o primeiro dígito é 1 ou 2. Um
mesmo ATR pode pertencer a mais de um produto (o chip é o mesmo, a personalização muda): o ATR identifica a
**família do cartão**, não o certificado. O ATR de alguns cartões pode conter bytes do chip; nunca vai para um
relatório de usuário, só para a lista.

---

## 1. Brasil (ICP-Brasil A3: token USB e cartão)

| Dispositivo | USB `VID:PID` | ATR (exemplos) | Middleware | Download oficial | Base |
|---|---|---|---|---|---|
| **SafeNet eToken 5110 / 5100** (Thales) | `0529:0620` (o `usb.ids` chama de "Token JC"; o `libccid`, de "eToken 5100"; relatos de Linux, de "eToken 5110 SC") · 5110+ FIPS `08e6:34cf` · 5300 `08e6:34cc` | 5110 SC: `3B FF 96 00 00 81 31 FE 43 80 31 80 65 B0 85 59 56 FB 12 0F FE 82 90 00 00` · 5110+ FIPS: `3B FF 96 00 00 81 31 FE 43 80 31 80 65 B0 84 65 66 FB 12 01 78 82 90 00 85` · 5110 CC (IDPrime 940C): `3B 7F 96 00 00 80 31 80 65 B0 85 05 00 39 12 0F FE 82 90 00` · eToken PRO Java: `3B D5 18 00 81 31 3A 7D 80 73 C8 21 10 30` | **SafeNet Authentication Client** (SAC). O 5110 CC tem um segundo slot, de assinatura, só visível pelo `IDPrimePKCS11` | [SAC (Thales)](https://cpl.thalesgroup.com/access-management/security-applications/authentication-client-token-management), [eToken 5110](https://cpl.thalesgroup.com/access-management/authenticators/pki-usb-authentication/etoken-5110-usb-token). O instalador fica no portal de suporte da Thales; as ACs distribuem cópias, e é a versão da AC que o médico usa | USB: C, U, B ([linux-hardware](https://linux-hardware.org/index.php?id=usb:0529-0620), [relato Gentoo](https://www.bananas-playground.net/2020/06/aladdin-etoken-safenet-gentoo-and-cisco/)). ATR: A. Página: D |
| **G+D StarSign Crypto USB Token** / cartões **SafeSign** | `1059:0017` · StarSign CUT S `1059:0019` | StarSign USB Token: `3B FD 18 00 00 81 31 FE 45 53 43 45 36 30 2D 43 43 30 38 31 2D 46 C2` · StarSign Token: `3B F8 18 00 00 80 31 FE 45 00 73 C8 40 13 00 90 00 92` · SafeSign: `3B 74 18 00 00 73 66 74 65` | **SafeSign Identity Client** (A.E.T. Europe / G+D) | [SafeSign (G+D Brasil): Windows, Mac e Linux](https://safesign.gdamericadosul.com.br/download) | USB: C. ATR: A. Página: D |
| **Feitian ePass2003** | `096e:0807` · ePass2003Auto `096e:080a` | `3B 9F 95 81 31 FE 9F 00 66 46 53 05 01 00 11 71 DF 00 00 .. .. .. ..` | driver **ePass2003 "Castle"** da Feitian, minidriver do Windows (Windows Update) ou o driver `epass2003` do **OpenSC** | [Feitian PKI Standard](https://www.ftsafe.com/Products/PKI/Standard), [Resources](https://www.ftsafe.com/Support/Resources) | USB: C, U. ATR: A. Links: D/B |
| **Watchdata ProxKey / WatchKey** (SERPRO "token branco") | `163c:0407`, `163c:0417`, `163c:0418` · `163c:0406` ("WatchCNPC USB CCID Key") · `163c:0a03` (W5181) | `3B 6E 00 00 57 44 36 19 69 86 93 02 ..` (várias séries: `…02 33 11 75 42 33 1E` é o token branco do SERPRO) · `3B 6D 00 00 57 44 36 41 01 86 93 ..` (ProxKey) | **Watchdata ICP** / WatchKey Admin Tool | não achei link estável do fabricante (`watchdata.com/brazil/…` responde 404); usar o link da AC ou do SERPRO. `TODO(gustavo)` | USB: C. ATR: A. Link: ? |
| **Dexon DXToken** (fabricação nacional) | `0483:a389` · eSmartDX `0483:a40b` | `3B 6B 00 00 80 5A 44 58 54 4F 4B 45 4E 30 31` · `3B 6E 00 00 80 31 08 72 14 22 57 44 58 54 4B 4E 30 31` | **DXSafe** (Windows 7+, macOS 10.10+, Linux) | [downloads da Dexon](https://www.dexon.ind.br/downloads) (aba "Drivers do DX-Safe") | USB: C. ATR: A. Link: B |
| **Athena IDProtect Key v2** | `0dc3:0900` | `3B D5 18 FF 81 91 FE 1F C3 80 73 C8 21 13 09` | **IDProtect Client** | [página do produto (Athena)](http://www.athena-scs.com/product.asp?pid=33), citada pelo pcsc-tools; pode estar desatualizada | USB: C. ATR: A. Link: ? |
| **Cartões A3** das ACs | (leitor CCID, ver §5) | e-CPF Gemalto TOP DL v2 (Imprensa Oficial, Caixa): `3B 7D 96 00 00 80 31 80 65 B0 83 11 11 E5 83 00 90 00` · e-CNPJ Certisign (Sagem YpsID): `3B 7D 18 00 02 80 57 59 50 53 49 44 30 33 83 7F 90 00` · e-CPF Safeweb (Morpho YpsID): `3B 7D 18 00 02 80 57 59 50 53 49 44 30 34 83 7F 90 00` · e-CNPJ Certisign (Oberthur Cosmo v7): `3B DD 18 00 81 31 FE 45 80 F9 A0 00 00 00 77 01 08 00 07 90 00 FE` · Serasa: `3B 3B F7 18 00 00 80 31 FE 45 73 66 74 65 2D` · e-CPF: `3B 68 00 00 00 73 C8 40 12 00 90 00` | SafeSign (YpsID e cartões G+D), Gemalto Classic Client / IDPrime (TOP DL), AWP da Oberthur | ver as linhas acima | ATR: A |

---

## 2. Portugal

| Dispositivo | USB `VID:PID` | ATR (exemplos) | Middleware | Download oficial | Base |
|---|---|---|---|---|---|
| **Cartão de Cidadão** (AMA) | (leitor CCID, ver §5) | 1ª geração: `3B 7D 95 00 00 80 31 80 65 B0 83 11 .. .. 83 00 90 00` · anterior: `3B 6B 00 00 00 31 C0 64 08 04 61 12 0F 90 00` · 2ª geração: `3B FF 96 00 00 81 31 FE 43 80 31 80 65 B0 85 04 01 20 12 0F FF 82 90 00 D0` | **Autenticação.Gov** (módulo `pteidpkcs11`; no macOS também `PteidToken` para o CryptoTokenKit). O CC emitido desde jun/2024 usa **ECDSA** | [Aplicação Autenticação.Gov (Windows, macOS, Linux)](https://www.autenticacao.gov.pt/web/guest/cc-aplicacao) · [manual](https://amagovpt.github.io/docs.autenticacao.gov/user_manual.html) | ATR: A. Links e módulo: D |

O CC tem chave com `CKA_ALWAYS_AUTHENTICATE` (PIN de assinatura a cada uso) e o módulo pede o PIN em janela
própria: ver [pkcs11-modules.md §6](pkcs11-modules.md#6-riscos-e-pontos-de-atenção).

---

## 3. Espanha

| Dispositivo | USB `VID:PID` | ATR (exemplos) | Middleware | Download oficial | Base |
|---|---|---|---|---|---|
| **DNIe** (Policía Nacional) | (leitor CCID, ver §5) | DNIe 1.0/2.0: `3B 7F 38 00 00 00 6A 44 4E 49 65 [12]0 02 4C 34 01 13 03 90 00` · DNI 3.0: `3B 7F 96 00 00 00 6A 44 4E 49 65 20 01 01 55 04 10 03 90 00` · DNIe 4.0: `3B 88 80 01 E1 F3 5E 11 77 81 00 00 A2` | **Módulo PKCS#11 do DNIe** (`libpkcs11-dnie`) ou **OpenSC** | [DNIe: instaladores](https://www.dnielectronico.es/PortalDNIe/PRF1_Cons02.action?pag=REF_1112) (na data: `.deb` para Debian 13/Ubuntu 25.04 e Debian 10 32 bits, `.rpm` para Fedora 38/openSUSE 15.6; Windows e macOS na mesma área) · [manual do módulo](https://www.dnielectronico.es/PDFs/manuales_instalacion_unix/Manual_de_Instalacion_de_MulticardPKCS11_DNIE.pdf) | ATR: A. Links: D |
| **FNMT-RCM** (cartão criptográfico CERES) | (leitor CCID) | `3B 7F 96 00 00 00 6A 46 4E 4D 54 03 04 11 43 04 30 03 90 00` · série CERES: `3B EF .. 00 40 14 80 25 43 45 52 45 53 57 .. .. 01 01 03 90 00` | módulo PKCS#11 FNMT-DNIe (`/opt/FNMTpkcs11dnie/lib/libpkcs11-dnie.so`) | [manual FNMT do módulo](https://www.sede.fnmt.gob.es/documents/10445900/10528353/Manual_de_Instalacion_de_MulticardPKCS11_FNMT_DNIE.pdf) | ATR: A. Manual: B |

---

## 4. América Latina

Nos cinco países que pesquisei o dispositivo dominante é o **SafeNet eToken 5110** (mesma linha do Brasil, mesmo
SAC). O que muda é quem o vende e qual é a lista oficial.

| País | Dispositivos citados | Middleware e download | Base |
|---|---|---|---|
| **Argentina** (firma digital, AFIP/ONTI) | SafeNet eToken 5110+ (FIPS 140-2 nível 2/3) e mToken CryptoID (FIPS 140-3 nível 3) | SafeNet Authentication Client; instalar antes de usar ([guia de compra e uso](https://mendohard.com.ar/token-firma-digital-cual-comprar-argentina/), [lojas](https://sitepro.com.ar/web/productos/firma-digital/token-onti-safenet-5110/)). O eToken 5110+ FIPS tem `08e6:34cf` | B |
| **Chile** (firma electrónica avanzada) | SafeNet eToken 5110 e 5110+ (Certinet, E-Sign, Acepta, E-CertChile) | SAC 10.8 (`SAC_10_8.exe`); versões antigas não atendem o 5110+ ([Acepta](https://asistencia.acepta.com/firma-avanzada.html), [Certinet](https://www.certinet.cl/firma-avanzada)). A cédula com chip (RUT) tem ATR `3B 88 80 01 31 CC CC 01 77 83 A1 00 6C` | B; ATR: A |
| **Colômbia** (Certicámara) | "E-Token SafeNet" e "E-Token Feitian" (Windows 10/11, 64 bits) | [Centro de descargas da Certicámara](https://web.certicamara.com/soporte/centro-de-descargas) | D |
| **Equador** (BCE, Security Data, Uanataca, Judicatura, Registro Civil) | tokens de cada entidade; a Uanataca usa o middleware da Bit4id | [drivers para tokens (firmadigital.gob.ec)](https://www.firmadigital.gob.ec/drivers-para-tokens/): links de cada entidade, inclusive o [middleware Bit4id](https://cdn.bit4id.com/es/middleware.htm). No Linux instalar antes `pcsc-lite`/`pcscd` | D |
| **Peru** (RENIEC, DNIe 2.0/3.0 e tokens) | homologados: Bit4id (IAm, CryptoKey, tokenME v1–v3, Digital DNA, J-Sign), SafeNet (iKey 4000, eToken 5100, eToken PRO Java, eToken 5110), Feitian (ePass 2003, AudioPass, BePass 2003), Athena IDProtect Key LASER, Sagem ypsID, Longmai mToken CryptoID, ACS (ACR101I, ACOS5-64, CryptoMate Nano) | [dispositivos homologados](https://pki.reniec.gob.pe/dcdelivery3/dispositivos-homologados.html) · [guia dos controladores do DNIe](https://identidad.reniec.gob.pe/documents/d/guest/guia_dnie_version3). Leitor com PKCS#11 multiplataforma: ACS ACR39U (`072f:b100`) | D |

Bit4id (`25dd:…`): IAm `2221`, CryptoKey `1201`, CKey4 `2321`, tokenME FIPS v3 `2341`, tokenME EVO v2 `2371`,
Digital DNA `2351`/`2354`/`2362` (C). ATR do tokenME: `3B 9F 11 81 31 FE 9F 00 6B 42 49 54 34 49 44 20 33 2E 30 00 90 00 CF` (A).
Longmai mToken CryptoID: ATR `3B 9F 11 81 31 FE 9F 00 6A 6D 54 6F 6B 65 6E 2D 46 00 00 81 90 00 79` (A); **`VID:PID` não achei** (?).
México (e.firma do SAT) usa arquivos `.cer`/`.key`, não token: fora do escopo.

---

## 5. Leitores comuns (CCID)

Qualquer leitor CCID serve (o `pcscd` usa o `libccid`); estes são os que aparecem nos guias de autoridades
certificadoras e em lojas dos países acima. IDs da lista CCID (C).

| Leitor | `VID:PID` |
|---|---|
| Gemalto PC Twin Reader | `08e6:3437` |
| Gemalto USB GemPCPinpad (com teclado de PIN) | `08e6:3478` |
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

Outros dispositivos frequentes: YubiKey 5 (PIV) OTP+U2F+CCID `1050:0407` e CCID `1050:0404` (U); ATR
do YubiKey 4: `3B F8 13 00 00 81 31 FE 15 59 75 62 69 6B 65 79 34 D4` (A).

---

## 6. Como isto vira `devices.json`

Exemplo com os campos que a interface consome, preenchido só com o que tem fonte acima:

```jsonc
{
  "id": "safenet-etoken-5110",
  "name": "SafeNet eToken 5110",
  "kind": "token",
  "match": {
    "usb": ["0529:0620"],
    // ATR em hex maiúsculo sem separadores; "..", do pcsc-tools, vira "??" (um byte qualquer)
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

Regras para a conversão:

1. **Licença antes de importar em massa.** O `devices.json` é CC0. A lista de ATRs do pcsc-tools é GPL-2.0+, o
   `Info.plist` do libccid é LGPL-2.1+ e o `usb.ids` é BSD/GPL. Números (`VID:PID`, bytes de ATR) e o nome
   comercial são fatos; **as descrições e os agrupamentos deles não são**. Importar só os fatos e escrever as
   descrições, ou pedir permissão a Ludovic Rousseau. `TODO(gustavo)`.
2. Um `match.usb` por dispositivo (lista, porque o mesmo produto tem várias revisões); o ATR só identifica
   cartões dentro de leitores genéricos, então é `match.atr`.
3. O ATR `..` do pcsc-tools vira curinga de um byte no `match`; o app compara em hex maiúsculo sem separadores
   (formato do `websign-probe devices`).
4. `driver.pkcs11` usa o mesmo primeiro caminho de [known_paths](../prototypes/kit/probe/src/keystores/pkcs11/known_paths/).
   Onde o fabricante não documenta caminho (Dexon, Longmai), o campo fica vazio: o diagnóstico continua
   oferecendo "adicionar módulo".
5. Sem certificado detectado e dispositivo conhecido → sugestão do driver; **nunca** bloquear a lista.

## 7. Lacunas (precisam de dispositivo real)

- Link estável de download da **Watchdata** (WatchKey/ProxKey) e da **Athena**; `TODO(gustavo)`: pedir à AC/SERPRO.
- Nome do arquivo PKCS#11 do **DXToken** e do **mToken CryptoID**, e o `VID:PID` do mToken.
- Se o 5110 **sem** "CC" (JavaCard, `0529:0620`) e o **5110 CC** (IDPrime 940) usam o mesmo `libeToken.so` no Linux
  ou se o CC exige `libIDPrimePKCS11.so`. A observação da Nexus (`eTPKCS11.dll` não expõe o slot de assinatura)
  sugere que o CC precisa do segundo.
- ATRs dos tokens ICP-Brasil **reais** de cada AC (as ACs mudam o fornecedor do chip por lote): a lista do
  pcsc-tools é um começo, o `websign-probe devices` em cada token confirma.

## Como atualizar

```sh
curl -O https://raw.githubusercontent.com/LudovicRousseau/pcsc-tools/master/smartcard_list.txt
curl -O https://raw.githubusercontent.com/vcrhonek/hwdata/master/usb.ids
# libccid: /usr/lib/pcsc/drivers/ifd-ccid.bundle/Contents/Info.plist (ifdVendorID, ifdProductID, ifdFriendlyName)
websign-probe devices --all-usb          # o que a máquina vê
```

Todas as páginas foram acessadas em 29/09/2026. O `github.com` é bloqueado no ambiente de pesquisa; os arquivos
brutos vieram de `raw.githubusercontent.com`.
