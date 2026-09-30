# Spec questions from the blind test writer

Each item is an ambiguity in `SPEC.md` / `docs/ux.md` where the tests picked a reading. If the implementer or
reviewer disagrees, fix the spec first, then the test named in brackets.

## Certificate list

1. SPEC §1.5 vector lists the input as "Cartão de Cidadão login certificate with a signing sibling" but the expected
   usable set has only 3 entries. A signing sibling must itself be usable, so the vector cannot be literal. The test
   builds the vector with the sibling and expects 4 usable rows `[A3, CC signing, A1 Ana, A1 clinic]`.
   [`cert_list_vector.rs`]
2. "Same device" for `LoginSibling` when both devices are `None`: not tested (only `Some(token)` devices).
3. `HiddenReason` order of the `hidden` vector is unspecified; tests look entries up by fingerprint.
4. Ties in the never-used order (equal hardware, name and `not_after`) are unspecified; tests avoid them or only
   assert group membership.
5. `CertList::append` into a list with `selected == None`: SPEC says "the selection never moves by itself"; the test
   expects the first usable row to become selected (a fresh build would). [`cert_list_append.rs`, last test]
6. `append` with a candidate that vanished from the listing: unspecified, untested.
7. Validity labels use local dates from `not_before`/`not_after` Unix seconds; the tests use noon UTC so any offset
   from -11 to +11 h gives the same date. Which time zone `build_cert_list` uses is not in `ListContext`.
8. `matches_filter`: a digits-only query is also allowed to match name/issuer text (SPEC says "or"); tested with a
   name containing digits.

## Confirmation window

9. "`Open` -> LoadingCerts; re-arm on `Focus(true)`": tests always send `Focus(true)` after `Open`. Only
   `a_new_open_re_arms_without_another_focus_event` assumes `Open` itself re-arms (SPEC: "a new `Open` does").
10. Which inputs are "ignored while unarmed"? SPEC says everything but Escape. Tests assume this covers
    primary press/release, Enter, `Select` and `Rescan`. `CancelButton`/`CloseButton` are only tested when armed
    (ux says Cancel is "enabled at all times", which contradicts "clicks are discarded").
11. Escape while `Signing` through the OS or the PIN pad (Cancel disabled): unspecified, untested.
12. `Certificates` and `DigestReady` re-arming: SPEC lists only `Focus`, `Open` and digest in `Choosing`. ux §4.7/§4.8
    also re-arm on a new digest in `Ready` and on a token inserted; tests assume both.
13. PIN validity when the token states no limits (`length: None`): tests assume any length >= 1 is valid and 0 is not.
14. `Failure::PinIncorrect { count_low: true, final_try: true }` maps to `PinError::IncorrectFinal`.
15. After `PinIncorrect` the field is empty: tests assume the model resets its typed length to 0 (the app zeroizes
    the real field) and needs a new `PinLength` before signing.
16. `UseAlternatePath` produces `Intent::Sign { via: 1, .. }` and moves to `Signing`; SPEC does not list it.
17. After a recoverable `Error` (driver failure) the primary button is `Retry` and a click signs again with `via: 0`.
    Which errors count as recoverable is not stated; only `DriverFailure` is asserted.
18. Empty state with only disabled rows: assumed `view.list` is `Some` and carries the disabled group.
19. `FooterHint::OsPinPrompt` is asserted in `Ready` for a `System` PIN; `FooterHint::OpenDiagnostics` in `Empty`.
20. Countdown: `ExpiresIn { seconds }` is asserted only at whole-second offsets from `Open` (remaining 30 -> 30,
    remaining 15 -> 15, remaining 1 -> 1); rounding of fractions is untested.
21. `next_deadline` is asserted only in states with one timer (arming pending, success hold, site-cancelled hold,
    last-30-s countdown) and as "not before 269 s" in a quiet armed state; the 150 ms skeleton deadline is not pinned.
22. The skeleton (`Preparing { skeleton: true }`) is measured from the `Select` that started preparing, and only
    tested for remembered callers (for a new caller the reference instant after `Continue` is ambiguous).
23. `Finish::Timeout` -> `Timeout` state; whether it auto-closes to `Idle` is unspecified, untested.
    `Finish::Aborted` and `Hide` are unspecified, untested.
24. A `Queue` update carrying another request's key is ignored.

## Diagnostics report

25. `DeviceLine::Reader.atr` is assumed to be already colon-separated (`3B:D5:...`) and printed as is; if `render`
    is meant to format raw hex, `the_ux_example_is_reproduced_byte_for_byte` needs raw input. [`diagnostics_report.rs`]
26. Empty `kinds` / `keys`: the text is unspecified (the example always has both); tests never render them empty.
27. More than 20 `recent_errors`: unspecified (doc says "at most 20" for the input); untested.
28. `render` keeps `kinds`/`keys` in input order (the ux example is not alphabetical: `a3` before `a1`).
