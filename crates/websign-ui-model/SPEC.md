# websign-ui-model — specification

Pure view logic of both windows. Behavior comes from
[`docs/ux.md`](../../docs/ux.md) (pt-BR; section numbers below refer to it);
this file fixes what the UX leaves open and lists the test vectors. Time is
always passed in (`Instant`, `jiff::civil::Date`).

## 1. Certificate list (`certs`)

### 1.1 Candidate → row

`name` = `websign_core::present::holder::display_name`; `document` =
`present::document::display_document`; `issuer` = issuer CN, else O, else
`""`; `badge` §1.2; `location` §1.4; `validity` §1.3; `status` §1.5.

### 1.2 `badge` (ux §5.3), first rule that matches

1. issuer CN contains `Cartão de Cidadão` (case-insensitive) → `PtCitizenCard`;
2. issuer O equals `DIRECCION GENERAL DE LA POLICIA` (case-insensitive,
   accents ignored) → `EsDnie`;
3. `icp_brasil.level` is `A1..A4`, `S1..S4`, `T3`, `T4` → `IcpBrasil { class }`;
4. `icp_brasil` is `Some` (other or no level) → `IcpBrasilPlain`;
5. `qualified` with `compliance && sscd` → `EidasQualified`;
6. `qualified` with `compliance` → `Eidas`;
7. → `Generic`.

### 1.3 `validity_label(not_before, not_after, today)` (ux §5.6, §16.5)

`days = not_after - today` in calendar days.

| Condition (checked in order) | Label | Tone |
|---|---|---|
| `today < not_before` | `ValidFrom(not_before)` | Warning |
| `today > not_after` | `ExpiredOn(not_after)` | Danger |
| `days == 0` | `ExpiresToday` | Danger |
| `days == 1` | `ExpiresTomorrow` | Danger |
| `2 ≤ days ≤ 7` | `ExpiresInDays(days)` | Danger |
| `8 ≤ days ≤ 30` | `ExpiresInDays(days)` | Warning |
| `days > 30` | `ValidUntil(not_after)` | Muted |

Vectors with today = 2026-09-29: not_after 2026-10-30 → ValidUntil, Muted;
2026-10-29 → ExpiresInDays(30), Warning; 2026-10-22 → ExpiresInDays(23),
Warning; 2026-10-06 → ExpiresInDays(7), Danger; 2026-09-30 → ExpiresTomorrow;
2026-09-29 → ExpiresToday; 2026-05-10 → ExpiredOn; not_before 2026-10-01 →
ValidFrom, Warning.

### 1.4 `location` (ux §5.7)

- `source` Windows/MacosKeychain and `hardware == Some(false)` → `Computer`.
- `device == Token { name }` → `Token { name }`.
- `device == CardInReader` → `CardInReader`.
- otherwise (hardware true/unknown, no safe device) → `UnknownHardware`,
  except software keys (`hardware == Some(false)`) of any source → `Computer`.
- `via_driver` = `source` is `Driver`.

### 1.5 `build_cert_list` (ux §5.8, §5.9, §16.6)

Hidden (never listed), first reason that applies: `info` is `Err` →
`Unparseable`; `!has_private_key` → `NoPrivateKey`; `is_ca` → `CertificateAuthority`;
key usage present without digitalSignature/nonRepudiation → `KeyUsage`; EKU
non-empty and every OID in {serverAuth `1.3.6.1.5.5.7.3.1`, codeSigning `.3`,
timeStamping `.8`, OCSPSigning `.9`} → `ExtendedKeyUsage`; key `Unsupported`
→ `UnsupportedKey`; login sibling (same subject DN **and** same `device`
as another candidate that has nonRepudiation, while this one has
digitalSignature without nonRepudiation) → `LoginSibling`.

Disabled (listed under "Can't sign"), first reason: `removed` → `Removed`;
`now > not_after` → `Expired`; `now < not_before` → `NotYetValid`; `pin`
is `App { locked: true, .. }` → `PinLocked`; `accepted` non-empty and no
algorithm of `accepted` in `candidate.algorithms` → `Incompatible`.

Usable order: (1) `last_used_here` if usable; (2) by position in
`recent_anywhere`; (3) never used: hardware before software (`Some(true)`
before `None` before `Some(false)`), then name A→Z (case- and
accent-insensitive), then later `not_after` first. Disabled rows: same order.

Selection: `requested` if usable, else `last_used_here` if usable, else the
first usable, else `None`.

Vector §16.6: A3 of Ana via Windows **and** via driver (same fingerprint: one
candidate with an alternate), A1 of Ana, A1 of the clinic, expired old A3,
Cartão de Cidadão login certificate with a signing sibling, a certificate
without private key → usable [A3 (alternate kept), A1 Ana, A1 clinic];
disabled [old A3 Expired]; hidden [CC login LoginSibling, no-key
NoPrivateKey]. With `last_used_here` = A1 Ana: it comes first and is selected.

### 1.6 `CertList::append`

Existing rows keep position and selection; new usable candidates are
appended to `usable`; a selected row whose candidate is now `removed`
becomes `Disabled(Removed)` **in place** (stays in `usable`'s position,
selection kept); when it returns, it is usable again. New disabled
candidates go to the end of `disabled`.

### 1.7 `matches_filter`

Case- and accent-insensitive (`NFD` strip of combining marks) substring of
`name` or `issuer`; or `query` made of digits (ignoring `.`, `-`, `/`, space)
contained in the digits of the document's visible part (`visible` for CPF,
the formatted CNPJ, the last 3 of a national ID). Empty query matches all.

## 2. Confirmation window (`confirm`)

### 2.1 `Arming` (ux §4.7)

- `rearm(now)`: `since = now`, forget a pending press.
- `disarm()`: `since = None`.
- `is_armed(now)` ⇔ `since` is set and `now - since ≥ 600 ms`.
- `press(now)` records whether it happened armed. `release(now)` returns
  `true` only if the press was armed **and** `is_armed(now)`; it clears the
  press either way.
- `armed_at()` = `since + 600 ms`.

### 2.2 `ConfirmModel`

States and transitions follow ux §4.8 with D11:

| From | Input | To / effect |
|---|---|---|
| Idle | `Open` | LoadingCerts; re-arm on `Focus(true)` |
| LoadingCerts | `Certificates` with usable rows | Choosing (selection from the list) |
| LoadingCerts | `Certificates` without usable rows | Empty |
| Empty | `Certificates` with usable rows | Choosing |
| Choosing | new caller, primary click (armed) | intent `Continue(selected)`; code card `Preparing` |
| Choosing | remembered caller | (host already sent the digest request) code card `Preparing` |
| Choosing | `DigestReady` for the selected certificate | Ready; re-arm |
| Ready | `Select(other)` | Choosing; intent `Selected(other)`; re-arm; (remembered: `Preparing`, new: `ContinueHint`) |
| Ready | primary click or Enter (armed; PIN length valid when the field is shown) | Signing; intent `Sign { … }` |
| Signing | `Failed(PinIncorrect)` | PinError; field cleared (the app zeroizes), focus PIN |
| Signing | `Failed(PinLocked)` | PinLocked; row disabled |
| Signing | `Failed(other)` | Error { code } |
| Signing | `Finished(Signed)` | Success; `tick` after 900 ms → Idle (or next `Open`) |
| any on screen | Escape / Cancel / Close | intent `Cancel(cancel_code(state))` |
| any | `Finished(SiteCancelled)` | SiteCancelled; 1500 ms → Idle |
| any | `Finished(Timeout)` | Timeout; closes |
| Choose mode, Choosing (armed) | primary click | intent `Choose { … }`; `Finished(Chosen)` → Success |

Rules: inputs other than Escape are ignored while unarmed; Enter on a list
row never signs (moves focus); focus loss disarms, focus gain re-arms; a
`Queue` update does not re-arm, a new `Open` does; `DigestReady` for a
certificate that is no longer selected is ignored.

`next_deadline`: the earliest of arming completion, the success/cancelled
hold end, the next countdown second in the last 30 s, and the 150 ms
skeleton delay.

### 2.3 `cancel_code`

Empty → `NoCertificates`; PinLocked → `PinLocked`; Error with
`CertificateUnavailable` → `CertificateUnavailable`; otherwise
`UserCancelled`.

### 2.4 `view` and `banner_code`

`view` maps the state to `ConfirmView` exactly as ux §4.2–§4.11 describe:
button labels (`Continue` for a new caller before release, `Sign`,
`Signing`, `Retry` after a recoverable error, `UseCertificate` in choose
mode), `primary_first` on Windows, `cancel_enabled = false` only while
signing with an OS or PIN-pad prompt, `FooterHint::ExpiresIn` in the last
30 s, `RememberBox` hidden for remembered callers, disabled when
`can_remember` is false. `banner_code` maps each `Failure` to its protocol
code (`PinIncorrect`, `PinLocked`, `TokenRemoved`, `DriverFailure`,
`UnsupportedAlgorithm`, `CertificateUnavailable`, `Internal`).

## 3. Diagnostics (`diagnostics`)

### 3.1 Lights (ux §8.1)

- Browsers: Red if `connected == 0` or `registration_missing_everywhere`;
  Yellow if `with_problems > 0`; else Green.
- Devices: Red if `pcscd_stopped`; Yellow if devices without certificates,
  failed drivers or an outdated Add-on; else Green.
- Certificates: Red if `usable == 0`; Yellow if `expiring_within_30_days > 0`;
  else Green.
- `overall`: the maximum by `Red > Yellow > Green > Gray`; empty → Gray.

### 3.2 Onboarding (ux §8.2)

app always Done; extension Done if connected, Attention if a problem,
Pending otherwise; certificate Done if usable > 0 else Pending; test Done if
done else Pending; `visible` = not dismissed and not all Done.

### 3.3 Report (ux §8.7)

Exact text, lines joined with `\n`, trailing newline, English:

```
WebeSign diagnostics v1
app: {app_version} ({packaging}, {arch}) · protocol {protocol} · locale {locale} · scale {scale_percent}%
os: {os}
render: {render}
browsers:
  {name} {version|?} · extension {extension_version|not seen} · host {registered|not registered}[ · last ping {last_ping}]
devices:
  usb {vid_pid} {hint_id|unknown} · certs {n}
  reader "{name}" · atr {ATR with ':' every byte|none} · certs {n}
pkcs11:
  {path} · loaded · slots {s} · tokens {t}[ (user-added)]
  {path} · failed: {reason}[ (user-added)]
certificates:
  usable {os+pkcs11} (os {os}, pkcs11 {pkcs11}, deduplicated {d}) · hidden {sum} (expired {e}, login-only {l}[, other {o}])
  kinds: {k1} {n1}, … · keys: {k} {n}, …
  expiring<=30d {n}
complement: {complement}
recent errors (last 20):
  {at} {operation} {code} {source}[ {native}]
```

Empty sections print `  none`. The golden test is the example of ux §8.7.

## 4. `time::relative`

`then` in the future or < 60 s ago → `JustNow`; < 60 min → `MinutesAgo(n)`
(floor); same local date → `TodayAt`; previous date → `YesterdayAt`; else
`On(then)`.
