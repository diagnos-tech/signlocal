# Como contribuir

> **English summary.** Every commit must be signed off (`git commit -s`, DCO 1.1).
> By signing off you also accept that your contribution is distributed under the
> license of the part you change — including the GPL section 7 additional
> permission for app stores in [`LICENSE`](LICENSE). Commit messages follow
> `<type>(<scope>): description`. New behavior starts with a spec and tests.

## Licença e DCO (obrigatório)

Cada commit precisa da linha `Signed-off-by: Nome <email>` (use `git commit -s`).
Com ela você declara que concorda com o [Developer Certificate of Origin 1.1](https://developercertificate.org/)
e que sua contribuição é licenciada sob a licença da parte que você alterou:

| Parte | Licença |
|---|---|
| `app/`, `extension/`, `safari/`, `packaging/`, `site/`, `docs/` | GPL-3.0-or-later **com a permissão adicional de lojas de apps** (seção 7, em [`LICENSE`](LICENSE)) |
| `sdk/` | Apache-2.0 |
| `devices.json` | CC0-1.0 |

A permissão adicional existe para que o app possa ser distribuído pela Mac App Store
e pela Microsoft Store. Contribuição que não aceite essa permissão não pode ser integrada.

Dependências novas precisam ser compatíveis com GPL-3.0 (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0).
No `sdk/` a regra é mais dura: **zero dependências de runtime**.

## Commits

```
<tipo>(<escopo>): descrição no imperativo, em pt-BR
```

- Cabeçalho com no máximo 100 caracteres.
- Tipos: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- Escopos: `app`, `extension`, `sdk`, `safari`, `devices`, `packaging`, `site`, `docs`, `kit`.
- Um commit, um assunto. `git add` só dos arquivos do seu trabalho.

Exemplo: `feat(app): listar certificados do repositório do Windows via CNG`

## Fluxo de desenvolvimento (TDD em três papéis)

Todo comportamento novo passa por três papéis, que podem ser pessoas ou agentes:

1. **Especificação** — a interface pública (tipos e assinaturas) e o comportamento
   esperado, incluindo erros, num `SPEC.md` ao lado do código ou na descrição do PR.
2. **Testes e implementação às cegas, em paralelo** — quem escreve os testes não vê a
   implementação e vice-versa; os dois partem só da especificação. Divergências
   revelam ambiguidades da especificação, não só bugs.
3. **Revisão crítica** — alguém experiente roda tudo, decide quem está certo em cada
   divergência (à luz da especificação e das normas), corrige e documenta a decisão.

## Estilo

- Arquivos pequenos (idealmente < 200 linhas), um conceito por arquivo, subpastas por assunto.
- Nomes que dispensam comentário; comentários explicam o **porquê**, nunca o óbvio.
- Código e comentários em inglês; documentação em `docs/` em pt-BR.
- Pendências como `TODO(nome)`; nada de referências a planos ou fases no código.
- Rust: `cargo fmt` e `cargo clippy -- -D warnings` limpos.

## Segurança

Nunca registre PIN, certificado completo, nome, CPF/CNPJ ou digest em logs.
Vulnerabilidades: não abra issue pública — escreva para o mantenedor.
<!-- TODO(gustavo): criar SECURITY.md com o e-mail de contato para vulnerabilidades. -->
