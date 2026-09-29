# Matriz de compatibilidade

Resultado de testes **manuais** com dispositivos reais. O CI cobre só chaves de software
(SoftHSM2, provedores de software do Windows, keychain de teste); tudo que depende de hardware
entra aqui.

Como preencher: rode `websign-probe report --run-signatures --all --hash all --pss --out r.md`
(binários nos artefatos do CI, `websign-probe-*`). Para cada dispositivo, preencha uma linha por
caminho e anexe o relatório ao PR. O relatório não tem nome, CPF/CNPJ nem número de série.

Legenda: ✅ funciona · ❌ não funciona · ⚠️ funciona com ressalva (explique) · — não se aplica · vazio = não testado.

## Tokens e cartões

| Dispositivo | Middleware (versão) | SO | Caminho | Lista sem PIN | RSA v1.5 | RSA-PSS | ECDSA | PIN em primeiro plano | PIN errado / bloqueio | Data · quem |
|---|---|---|---|---|---|---|---|---|---|---|
| SafeNet eToken 5110 | SafeNet Authentication Client | Windows 11 | CNG (`allow`) | | | | | | | |
| SafeNet eToken 5110 | SafeNet Authentication Client | Windows 11 | PKCS#11 (`eTPKCS11.dll`) | | | | | — | | |
| SafeSign (G&D StarSign) | SafeSign Identity Client | Windows 11 | CNG / CAPI | | | | | | | |
| Feitian ePass2003 | driver do fabricante / OpenSC | Windows 11 | CNG (minidriver) | | | | | | | |
| Watchdata ProxKey | Watchdata | Windows 11 | CAPI (CSP) | | | | | | | |
| A1 instalado (.pfx) | — | Windows 11 | CAPI → reabertura AES | | | | — | — | — | |
| SafeNet eToken 5110 | SafeNet Authentication Client | macOS 15 | CryptoTokenKit | | | | | — | | |
| SafeSign | SafeSign 4.x | macOS 15 | CryptoTokenKit | | | | | — | | |
| ePass2003 | OpenSC (OpenSCToken) | macOS 15 | CryptoTokenKit | | | | | — | | |
| Cartão de Cidadão (PT) | Autenticação.gov 3.11+ | macOS 15 | CryptoTokenKit | | | | | — | | |
| DNIe (ES) | libpkcs11-dnie | macOS 15 | PKCS#11 (sandbox?) | | | | | — | | |
| SafeNet eToken 5110 | SafeNet Authentication Client | Ubuntu 24.04 | PKCS#11 via p11-kit | | | | | — | | |
| ePass2003 | OpenSC | Ubuntu 24.04 | PKCS#11 via p11-kit | | | | | — | | |

## Navegadores (native messaging)

| SO | Navegador (versão) | Instalação do app | Host iniciado | Assina | Observações | Data · quem |
|---|---|---|---|---|---|---|
| Windows 11 | Chrome | MSIX (alias) | | | | |
| Windows 11 | Edge | MSIX (alias) | | | | |
| Windows 11 | Firefox | MSIX (alias) | | | | |
| Windows 10 22H2 | Chrome | MSIX (alias) | | | | |
| macOS 15 | Chrome | app na sandbox | | | | |
| macOS 15 | Safari | app da loja + appex | | | | |
| Ubuntu 24.04 | Chrome (deb) | .deb | | | | |
| Ubuntu 24.04 | Firefox (Snap, portal) | .deb | | | | |
| Fedora | Firefox (rpm) | .rpm | | | | |

## CI (chaves de software)

O resultado mais recente de cada sistema fica nos artefatos `report-linux`, `report-windows` e
`report-macos` do workflow `prototypes`, e está resumido em cada documento de
[`docs/prototypes/`](prototypes/).
