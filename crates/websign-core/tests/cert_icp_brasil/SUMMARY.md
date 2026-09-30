# crates/websign-core/tests/cert_icp_brasil

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §6.3: ICP-Brasil detection, level, holder, CPF and CNPJ.
- `reference.rs` — the reference certificates
- `level.rs` — level
- `detection.rs` — what makes a certificate ICP-Brasil
- `holder_name.rs` — holder name
- `cpf.rs` — CPF
- `cnpj.rs` — CNPJ
- `formatting.rs` — masked_cpf and formatted_cnpj
- `types.rs` — IcpLevel and IcpBrasil
