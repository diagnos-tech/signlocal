# Spec questions from the test writer

Each item says what the SPEC leaves open and what the tests assume (marked
`SPEC:` in code where it matters). Resolve by amending `SPEC.md`; the tests
then either stay or change.

## §4.1 Brainpool

1. Fixture keys for the three `r1` curves are new; the old `brainpoolP256r1.der` was replaced (its key was never committed, so no signature could be made for it). Assumption: nothing depends on its bytes.
2. The `BrainpoolP512r1` order of errors is assumed to follow §7 literally: digest length, then `KeyMismatch`, then `UnsupportedKey` (even for junk signatures). Tests: `verify_brainpool.rs`.
3. Off-curve EC points for Brainpool are asserted as `UnsupportedKey` by corrupting the last byte of the point; a single-bit flip could in theory land on the curve (probability about 2^-256), which the tests ignore.

## §9 Origin

4. Empty port (`https://a.example:`) and a trailing dot in the host (`https://example.com.`): not specified; only "no panic" is tested.
5. Leading/trailing whitespace or control characters: tests only require an `Err` (either variant).
6. A port written with a plus sign (`:+443`) is asserted as `Malformed` (not a port in the URL grammar).
7. `xn--dignos-4nf.health` is assumed to decode to `di` + Cyrillic `а` (U+0430) + `gnos.health`, as in the §9 table.
8. An IDN host under `*.localhost` (`http://bücher.localhost`) is accepted as secure, gets `Idn` (first warning that applies) and `can_remember = false`. The spec says both rules apply; the interplay is inferred.
9. Private-section suffixes: `app.tenant.github.io` is asserted to have `tenant.github.io` as registrable domain, and `github.io` alone is shown whole. This depends on the `psl` crate's list snapshot.
10. `unicode` for an IDN host with ASCII registrable domain (`bücher.example.com`): tests do not pin the exact string, only that the warning is `Idn`.

## §10 Holder

11. Company suffixes "kept as written in the input": tests only feed them in upper case (`ME`, `EPP`, `EIRELI`, `S.A.`, `S/A`). Lowercase input (`me`) is not tested; it is unclear whether the match is case-insensitive.
12. "Words" are non-space runs, so `R2:D2` is one word and becomes `R2:d2`; `A.B.` becomes `A.b.`. Asserted as the literal reading of the rule.
13. A particle inside a hyphenated word (`DA-SILVA`) and a leading particle after leading spaces are not tested.
14. The fingerprint fallback is lowercase hex and therefore never title-cased (its letters are not all uppercase). Asserted.
15. When CN and O are blank and only one of given name / surname exists, the fingerprint prefix is used. Asserted.

## §11 Document

16. A malformed CPF (not exactly 11 ASCII digits) or CNPJ (not 14) in `IcpBrasil` (the fields are public and can be set by hand): asserted to be skipped, falling through to the next rule, never shown and never a panic. §11 says `cpf = 12345678909` only.
17. An ETSI serial number is matched on the whole string: prefix exactly three plus two uppercase ASCII letters, a dash, at least three characters after it. The mask uses the last three characters (Unicode scalar values), so `IDCPT-ABCDEFGH-1234` gives `•••••234`.
18. `icp-pj-a1` has both the responsible person's CPF and a CNPJ: §11 says CPF first, so the label is the CPF of the responsible person. Asserted as written, though `docs/ux.md` may prefer the company number for a company certificate.

## §12 Caller

19. "At most 64 characters with `…`": whether the ellipsis counts in the 64 is unspecified; tests accept 64 or 65 characters in total and require 63 kept characters.
20. An executable path with no file name (`/`, empty) has no specified label name; only "no panic" is tested.
21. Test paths use `/` only. Windows-style `C:\...` paths are not tested because `Path::file_name` on Linux does not split on `\`.

## §13 Wire

22. `Qualified.types` repeats (`[Web, ESign, ESeal, Web]`) are mapped one to one, keeping order and duplicates, as §6.4 says QcType statements are not de-duplicated.

## docs/ux.md §16.1 verification code

23. The vectors of §16.1 test `websign_protocol::code::verification_code` (Apache-2.0 crate, re-exported by this crate), not new `websign-core` code. No `websign-core` test covers them; they belong to the protocol crate's tests. If the core is expected to own them, say so in SPEC §9–§13.

## Fixtures

24. `ADDITIVE=1 generate.sh` produced the new fixtures without regenerating the promoted ones, to keep the promoted certificates byte-identical. The new certificates are signed by a throwaway root (same subject as `ca.der`, different key). Nothing verifies certificate signatures.
