# WebeSign

Assine em qualquer site com o seu certificado digital — token USB, cartão inteligente
ou certificado instalado no sistema. Chrome, Edge, Firefox e Safari; Windows, macOS e Linux.

> *Use your smart card, USB token, or OS certificate to sign on any website.
> Works with Chrome, Edge, Firefox, and Safari on Windows, macOS, and Linux.*

> **Status:** em validação técnica (provas de risco). Ainda não há versão para uso.
> Veja [`docs/prototypes/`](docs/prototypes/).

## Como funciona

```
site ──hash──▶ SDK ──▶ extensão ──native messaging──▶ app ──▶ sistema operacional
                                                        │      (CNG/CAPI, Keychain)
                                                        └────▶ driver PKCS#11
site ◀──────────── assinatura crua (ECDSA em r‖s, RSA) ◀─┘
```

- **Só o hash** sai do site; o documento nunca. Quem monta PAdES/CAdES/XAdES é o site —
  por isso serve a qualquer padrão (ICP-Brasil, eIDAS…) e a qualquer país.
- **O app pergunta ao sistema, não ao chip.** Todo token, cartão ou certificado que o
  sistema operacional ou um driver PKCS#11 enxerga funciona, sem lista de modelos.
- **Nada roda em segundo plano** e nenhuma porta é aberta: o navegador inicia o app
  quando precisa. Cada assinatura é confirmada numa janela do próprio app.

## Peças

| Pasta | O que é | Licença |
|---|---|---|
| `sdk/` | Biblioteca TypeScript para sites, sem dependências | Apache-2.0 |
| `extension/` | Extensão (Chrome, Edge, Firefox, Safari) que liga a página ao app | GPL-3.0-or-later |
| `app/` | App desktop em Rust: host de native messaging + janelas de confirmação e diagnóstico | GPL-3.0-or-later |
| `safari/` | Ponte Swift exigida pela Apple para a extensão do Safari | GPL-3.0-or-later |
| `devices.json` | Dicas de driver por dispositivo (USB e ATR), mantidas pela comunidade | CC0-1.0 |
| `packaging/`, `site/` | Empacotamento por loja/SO e página de download | GPL-3.0-or-later |

O GPL do app e da extensão não se estende aos sites: eles conversam por mensagens,
não por ligação de código, e o SDK que o site empacota é Apache-2.0.
A licença inclui uma permissão adicional para distribuição por lojas de apps — veja [`LICENSE`](LICENSE).

## Documentação

- [`docs/prototypes/`](docs/prototypes/) — provas de risco por sistema operacional
- [`docs/plan.md`](docs/plan.md) — plano de implementação
- [`docs/ux.md`](docs/ux.md) — especificação de interface
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — como contribuir (DCO obrigatório)

Nomes e identificadores provisórios ficam todos em [`project.toml`](project.toml).
