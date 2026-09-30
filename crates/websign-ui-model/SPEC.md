# websign-ui-model — specification

Pure view logic of both windows. Behavior comes from
[`docs/ux.md`](../../docs/ux.md) (pt-BR; section numbers below refer to it);
this file fixes what the UX leaves open and lists the test vectors. Time is
always passed in (`Instant`, `jiff::civil::Date`, and the time zone in
`ListContext::time_zone`), so no rule depends on the machine running it.

## 1. Certificate list (`certs`)

### 1.1 Candidate → row

`name` = `websign_core::present::holder::display_name`; `document` =
`present::document::display_document`; `issuer` = issuer CN, else O, else
`""`; `badge` §1.2; `location` §1.4; `validity` §1.3; `status` §1.5.
`validity` converts `not_before`/`not_after` (Unix seconds) to calendar dates
in `ListContext::time_zone` (the host passes `TimeZone::system()` and
`today` in the same zone); instants outside jiff's range clamp to the
nearest representable date.

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
→ `UnsupportedKey`; login sibling (same subject DN **and** equal `device`
as another candidate that has nonRepudiation, while this one has
digitalSignature without nonRepudiation) → `LoginSibling`. `device`
equality includes `None == None` (both in a store with no device mapping):
a login-only certificate is never the right choice next to a signing one of
the same holder, and diagnostics still counts it as login-only. `Some(_)`
and `None` are different places. `hidden` keeps the candidates' input
order.

Disabled (listed under "Can't sign"), first reason: `removed` → `Removed`;
`now > not_after` → `Expired`; `now < not_before` → `NotYetValid`; `pin`
is `App { locked: true, .. }` → `PinLocked`; `accepted` non-empty and no
algorithm of `accepted` in `candidate.algorithms` → `Incompatible`.

Usable order: (1) `last_used_here` if usable; (2) by position in
`recent_anywhere`; (3) never used: hardware before software (`Some(true)`
before `None` before `Some(false)`), then name A→Z (case- and
accent-insensitive), then later `not_after` first. Disabled rows: same order. The sort is
stable: rows equal on every key keep the candidates' input order.

Selection: `requested` if usable, else `last_used_here` if usable, else the
first usable, else `None`.

Vector §16.6: A3 of Ana via Windows **and** via driver (same fingerprint: one
candidate with an alternate), A1 of Ana, A1 of the clinic, expired old A3,
Cartão de Cidadão login certificate with its signing sibling, a certificate
without private key → usable [A3 (alternate kept), CC signing, A1 Ana,
A1 clinic] (hardware before software, then name); disabled [old A3
Expired]; hidden [CC login LoginSibling, no-key NoPrivateKey]. With
`last_used_here` = A1 Ana: it comes first and is selected. `docs/ux.md`
§16.6 says "3 usable" because it counts only the physician's and the
clinic's certificates; the signing sibling has nonRepudiation, passes every
rule and is listed, so the vector has **4** usable rows.

### 1.6 `CertList::append`

Existing rows keep position and selection; new usable candidates are
appended to `usable` (ordered among themselves by §1.5); a row whose
candidate is now `removed` becomes `Disabled(Removed)` **in place** (stays
where it is, selection kept); when it returns, it is usable again. Only
`Removed` clears by itself: a locked PIN or an expiry does not while the
window is open. A disabled row that becomes usable moves to the end of
`usable`. New disabled candidates go to the end of `disabled`; new hidden
ones are recorded in `hidden`. A row missing from the new listing is kept
unchanged (a skipped certificate is a key-store hiccup; a token that left is
reported with `removed`). A list with no selection selects its first usable
row, as a fresh build would: there is no choice of the person to keep.

### 1.7 `matches_filter`

Case- and accent-insensitive (`NFD` strip of combining marks) substring of
`name` or `issuer`; or `query` made of digits (ignoring `.`, `-`, `/`, space)
contained in the digits of the document's visible part (`visible` for CPF,
the formatted CNPJ, the last 3 of a national ID). Empty query matches all.
The two tests are alternatives: a digits-only query also matches digits in
the name or issuer text.

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
| Idle | `Open` | LoadingCerts; timeout starts; arming per §2.2.1 |
| LoadingCerts / Empty | `Certificates` with a usable row | Choosing (selection from the list) |
| LoadingCerts | `Certificates` without usable rows | Empty (`list` kept, with its disabled rows) |
| Choosing | new caller, primary click (armed) | intent `Continue(selected)`; code card `Preparing` |
| Choosing | remembered caller | (host already sent the digest request) code card `Preparing` |
| Choosing | `DigestReady` for the selected certificate, after Continue or for a remembered caller | Ready; re-arm |
| Ready | `DigestReady` again for the selection | code replaced; re-arm |
| Choosing / Ready / PinError / PinLocked / Error | `Select(other usable)` (armed) | Choosing; intent `Selected(other)`; re-arm; PIN length, PIN error, banner and path reset; (remembered: `Preparing`, new: `ContinueHint`) |
| Ready / PinError | primary click or `Enter` (armed; PIN length valid when the field is shown) | Signing; intent `Sign { via, … }` |
| Error (retryable) | primary click or `Enter` (armed, "Try again") | Signing; intent `Sign` on the last path |
| Error `DriverFailure { alternate: true }` | `UseAlternatePath` (armed) | Signing; intent `Sign { via: 1 }`; later retries keep `via: 1` |
| Signing | `Failed(PinIncorrect)` | PinError; typed length reset to 0 (the app zeroizes the field), focus PIN |
| Signing | `Failed(PinLocked)` | PinLocked; selected row `Disabled(PinLocked)` in place |
| Signing | `Failed(UnsupportedAlgorithm)` | Error; selected row `Disabled(Incompatible)` in place |
| Signing | `Failed(other)` | Error { code } |
| Signing | `Finished(Signed)` | Success; `tick` after 900 ms → Idle (or next `Open`) |
| request on screen | Escape / Cancel / Close | intent `Cancel(cancel_code(state))`, every press |
| Success / SiteCancelled / Timeout | Escape / Cancel / Close | Idle, no intent (the request is already answered) |
| any | `Finished(SiteCancelled)` | SiteCancelled; 1500 ms → Idle |
| any | `Finished(Timeout)` | Timeout ("The request expired"); 1500 ms → Idle |
| any | `Finished(Aborted)` or `Hide` | Idle at once, no notice |
| Choose mode, Choosing (armed) | primary click | intent `Choose { … }` once; `Finished(Chosen)` → Success |

Commands whose `key` is not the request on screen are dropped (a `Queue`
for another key included). `DigestReady` is ignored for a certificate that
is not selected and, for a new caller, before Continue: the window never
asked for it (D11).

#### 2.2.1 Arming

- The 600 ms count from the window being visible **and** focused (ux §4.7).
  The model tracks focus: `Focus(true)` re-arms, `Focus(false)` disarms.
  `Open` re-arms only a window that already has focus (the next request of a
  queue); a window that is just being shown stays unarmed until
  `Focus(true)`, so a click that only brings it forward never approves.
- Re-arm events (each restarts the count, or leaves the window unarmed while
  unfocused): `Focus(true)`, `Open`, `Select`, `DigestReady`, and a
  `Certificates` update that increases the number of rows able to sign (a
  token inserted, or plugged back in). A `Queue` update never re-arms.
- Leaving is never gated: Escape, `CancelButton` and `CloseButton` work armed
  or not (ux §4.7 "except Esc", §4.9 "Cancel, always", Cancel "enabled at all
  times"). The only exception is `Signing` through the OS or a PIN pad, where
  no API can abort: Cancel is disabled ("Please wait…") and Escape does
  nothing; the OS dialog or the keypad has its own cancel.
- Everything else is ignored while unarmed: primary press/release, `Enter`,
  `Select`, `Remember`, `Rescan`, `OpenDiagnostics`, `UseAlternatePath`.
  `Focus` and `PinLength` are always taken (they decide nothing).
- `Enter` means Enter in the PIN field or on the focused primary button; the
  app never sends it for Enter on a list row (ux §4.9: it moves focus). The
  model accepts `Enter` **only as Sign** (Ready, PinError, retryable Error):
  it never releases a certificate (Continue) and never chooses one (Choose
  mode). Before the code is on screen focus sits on the list, never on the
  primary button, so an Enter there is a list Enter or a stray key from the
  site. Keyboard activation of the focused Continue / "Use this certificate"
  button (Space) is reported by the app as a press and a release.

#### 2.2.2 PIN, retries and the code card

- PIN length is valid when inside the token's `(min, max)`; with no stated
  limits, any length ≥ 1. It only matters when our field is shown.
- `PinIncorrect { final_try }` → `IncorrectFinal`; else `count_low` →
  `IncorrectLow`; else `Incorrect`. The person must type again (a new
  `PinLength`) before signing.
- "Try again" (`PrimaryButton::Retry`) is offered for `TokenRemoved` (enabled
  once the row is usable again) and `DriverFailure`, per ux §15. `Internal`
  offers "Copy details" / "Open diagnostics" instead; `CertificateUnavailable`
  and `UnsupportedAlgorithm` ask for another certificate. For those the
  primary button reads `Sign`, disabled.
- `Preparing { skeleton }` turns true 150 ms after the card started
  preparing: the Continue click (new caller), entering Choosing (remembered
  caller) or a `DigestPending` for the selection, whichever is latest.
- The countdown shows whole seconds left, rounded up (14.5 s → 15).
- `next_deadline`: the earliest future instant of arming completion, the
  success / site-cancelled / timeout hold end, the countdown start or its
  next second, and the skeleton delay.

### 2.3 `cancel_code`

Empty → `NoCertificates`; PinLocked → `PinLocked`; Error with
`CertificateUnavailable` → `CertificateUnavailable`; otherwise
`UserCancelled`.

### 2.4 `view` and `banner_code`

`view` maps the state to `ConfirmView` exactly as ux §4.2–§4.11 describe:
button labels (`Continue` for a new caller before release, `Sign`,
`Signing`, `Retry` after a retryable error, `UseCertificate` in choose
mode), `primary_first` on Windows, `cancel_enabled = false` only while
signing with an OS or PIN-pad prompt, `FooterHint::ExpiresIn` in the last
30 s (it wins over any hint), `FooterHint::OsPinPrompt` for a system PIN,
`FooterHint::OpenDiagnostics` in Empty, `RememberBox` hidden for remembered
callers, disabled when `can_remember` is false. In Empty, `list` is `Some`
and carries the disabled rows. An idle view (never rendered: the window is
hidden) has a blank desktop caller. `banner_code` maps each `Failure` to its
protocol code (`PinIncorrect`, `PinLocked`, `TokenRemoved`, `DriverFailure`,
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
  reader "{name}" · card {hint_id} · certs {n}
  reader "{name}" · atr {masked ATR|N bytes|unreadable|none} · certs {n}
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

Empty sections print `  none`; empty `kinds` or `keys` print `none`.
`kinds` and `keys` keep the order they are given (the ux example lists A3
before A1). A reader whose card matched a device
(`hint_id`) prints `card {hint_id}` and never its ATR. Otherwise the ATR is
accepted in any hex spelling (`3BD518…`, `3b d5 …`, `3B:D5:…`) and printed by
`diagnostics::atr_mask::mask_atr` (§3.4). Only the newest 20 errors are printed,
oldest first. The golden test is the example of ux §8.7.

### 3.4 `atr_mask::mask_atr` (ux §8.7)

Historical bytes can hold a chip serial number, so they never leave the
model. The ATR is parsed per ISO/IEC 7816-3: TS (`3B`/`3F`), T0 (high nibble
Y1, low nibble K), then TA/TB/TC/TD groups while the TD's high nibble is
non-zero, then K historical bytes, then TCK unless every offered protocol is
T=0. Output, first rule that applies:

| Input | Output |
|---|---|
| Not whole hex bytes (`:` and spaces ignored) | `unreadable` (never echoed) |
| Exactly one well-formed ATR | upper-case bytes joined by `:` through the last interface byte, then `..` per historical byte and for TCK |
| Whole hex bytes that do not parse (bad TS, truncated, trailing bytes) | `{n} bytes` (`1 byte`) |

`3BD518FF8191FE1FC38073C821100A` → `3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:..`;
`3B02AABB` → `3B:02:..:..`; `3BD518FF8191FE1FC38073C8` → `12 bytes`. Vectors:
every ATR of `devices.json` (wildcards as `00`) keeps a prefix and masks the
rest; cut by one byte it prints its length; no input panics.

## 4. `time::relative`

`then` in the future or < 60 s ago → `JustNow`; < 60 min → `MinutesAgo(n)`
(floor); same local date → `TodayAt`; previous date → `YesterdayAt`; else
`On(then)`.
