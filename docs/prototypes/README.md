# Provas de risco

Antes de construir o produto, cada risco técnico que pode derrubar uma decisão
de arquitetura é provado (ou refutado) com código real. Cada prova tem um
documento com **sim/não, evidência e decisão**.

| # | Prova | Documento | Status |
|---|---|---|---|
| 1 | Windows: CNG/CAPI com tokens reais, PIN em primeiro plano, MSIX grava HKCU e o navegador inicia o host pelo alias | [1-windows.md](1-windows.md) | CI ✅ CNG/CAPI (A1 legado; PSS pela ponte CNG) e ✅ MSIX (HKCU real + Chromium inicia o host pelo alias); falta tokens, Windows cliente e aceite da loja |
| 2 | Mac: app na sandbox grava manifestos, Chrome inicia o host e assina via CryptoTokenKit, ponte do Safari | [2-mac.md](2-mac.md) | CI ✅ (sandbox, manifestos, host, Keychain); falta Mac real e Safari |
| 3 | Tokens no Mac: quais middlewares expõem o token ao CryptoTokenKit; PKCS#11 dentro da sandbox | [3-tokens-mac.md](3-tokens-mac.md) | CI: PKCS#11 ❌ na sandbox → complemento para tokens só-PKCS#11; falta matriz com tokens |
| 4 | Linux: assinatura via p11-kit, native messaging (inclusive Firefox Snap) | [4-linux.md](4-linux.md) | ✅ SoftHSM2/p11-kit + Chromium ponta a ponta (local e CI); falta token real e Firefox Snap |

## O kit

[`kit/`](kit/) é um workspace Cargo separado do produto:

- [`kit/probe-core/`](kit/probe-core/) — lógica pura e testada (algoritmos, codificação de
  assinaturas, resumo de certificados com ICP-Brasil e eIDAS, verificação, deduplicação).
  Especificação em [`SPEC.md`](kit/probe-core/SPEC.md). Será promovida para o app.
- [`kit/probe/`](kit/probe/) — o binário `websign-probe`: lista e assina por **todos** os
  caminhos que o app vai usar (CNG/CAPI, Keychain/CryptoTokenKit, PKCS#11), confere cada
  assinatura e também funciona como host de native messaging.
- `kit/extension/`, `kit/nm-e2e/` — extensão mínima e teste ponta a ponta de native messaging.
- `kit/windows/`, `kit/msix/`, `kit/macos/`, `kit/linux/` — scripts de prova por sistema.

O CI ([`.github/workflows/prototypes.yml`](../../.github/workflows/prototypes.yml)) roda as
provas com chaves de software nos três sistemas. Tokens reais: roteiro em cada documento.

### Uso rápido

```sh
cd docs/prototypes/kit
cargo run -p websign-probe -- list                 # todos os certificados, por origem
cargo run -p websign-probe -- sign --all --hash all --pss
cargo run -p websign-probe -- devices              # USB e leitores com ATR
cargo run -p websign-probe -- report --run-signatures --all --out relatorio.md
```

O relatório não contém nomes, CPF/CNPJ nem números de série — pode ser colado aqui.
