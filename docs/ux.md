# WebeSign: UX/UI Specification

> Reference document for whoever implements `app/` (egui), `extension/` (popup), `sdk/`, and `site/`.
> Navigable mockups: [`docs/ux/mockups.html`](ux/mockups.html) (light and dark; the tokens there are
> exactly those of [§11](#11-design-tokens)).
>
> Conventions: every decision carries a one-line **Why:**. Maintainer open items appear as
> `TODO(gustavo)`. Dimensions are in **logical px** (egui points, before DPI scaling). Interface texts
> appear in quotes with the i18n key next to them when one exists (e.g. `confirm.eyebrow`).
>
> **Language note.** The product UI ships in seven languages (en, es, pt-PT, pt-BR, fr, it, de). This
> specification is written in English; the copy tables in [§13](#13-copy-and-i18n) keep the pt-BR and en
> strings as data, because they are product copy. Where an example shows a pt-BR string, it is the
> Brazilian UI text, not documentation prose.

## Contents

1. [Principles](#1-principles)
2. [Vocabulary: from technical term to on-screen word](#2-vocabulary)
3. [UX requirements for the other components](#3-ux-requirements-for-the-other-components)
4. [Confirmation window](#4-confirmation-window)
5. [Certificate list](#5-certificate-list)
6. [Possible certificates](#6-possible-certificates)
7. [Add-on (macOS)](#7-add-on-macos)
8. [Diagnostics window](#8-diagnostics-window)
9. [Extension popup](#9-extension-popup)
10. [First run per operating system](#10-first-run-per-operating-system)
11. [Design tokens](#11-design-tokens)
12. [Icons](#12-icons)
13. [Copy and i18n](#13-copy-and-i18n)
14. [Accessibility](#14-accessibility)
15. [Errors](#15-errors)
16. [Reference test cases](#16-reference-test-cases)
17. [Open items](#17-open-items)

---

## 1. Principles

| # | Principle | Rule that follows from it |
|---|-----------|--------------------|
| P1 | **Trust comes from what can be verified.** | The window only shows what the app knows on its own: the site comes from the browser, the certificate comes from the operating system, the code comes from the digest. No text sent by the site appears in the window. |
| P2 | **Clarity for people outside the field.** | No technical term in the first layer (see [§2](#2-vocabulary)). Technical terms only in "Details" and in Diagnostics, always next to the plain-language version. |
| P3 | **Fast on day 100, not just on day 1.** | A physician signs dozens of reports a day: preselected certificate, PIN with Enter, a window that closes by itself on success. Every extra second multiplies. |
| P4 | **Anti-phishing by default.** | Registrable domain highlighted, warning for IP, punycode, and local sites, `http` blocked, Sign button armed only after 600 ms with the window in focus. |
| P5 | **Accessible without a special mode.** | AA contrast in both themes, everything operable by keyboard, AccessKit on, state never communicated by color alone. |
| P6 | **Actionable help, never a dead end.** | Every empty state or error says what to do and offers a button for it (download driver, repair, open diagnostics). |
| P7 | **Visible privacy.** | The user sees what leaves the computer: "The PIN stays on this computer", "The site will receive name, type, issuer, and validity", an exact preview of "Copy diagnostics". |

---

## 2. Vocabulary

The first layer of the interface uses only the "On screen" columns. The technical term may appear in "Details" and in
Diagnostics. (The pt-BR column is the Brazilian UI copy and stays in Portuguese.)

| Technical term | On screen (pt-BR) | On screen (en) |
|---------------|-----------------|--------------|
| digest / hash | código de conferência | verification code |
| origin | site | site |
| per-origin consent | permissão do site / "Lembrar este site" | site permission / "Remember this site" |
| PKCS#11 / cryptoki / module | driver do token | token driver |
| CNG, CAPI, Keychain, CryptoTokenKit | Windows / Mac / sistema | Windows / Mac / system |
| slot, PKCS#11 token | token | token |
| smart card | cartão | card |
| PC/SC reader | leitor de cartão | card reader |
| incompatible keyUsage / EKU | "não serve para assinar" / "feito para login" | "can't sign" / "made for login" |
| A1 certificate | "Neste computador" | "On this computer" |
| A3 certificate in hardware | "Token …" / "Cartão no leitor …" | "Token …" / "Card in reader …" |
| native messaging host not registered | "O navegador não encontra o app" | "The browser can't find the app" |
| protected authentication path | teclado do leitor | reader keypad |
| add-on (helper outside the sandbox) | Complemento | Add-on |
| CKR_PIN_LOCKED | PIN bloqueado | PIN locked |
| PUK / SO PIN | PUK (com explicação) | PUK (with explanation) |

**Why:** the main audience is physicians, not IT; IT support finds the technical terms in Diagnostics.

---

## 3. UX requirements for the other components

These requirements originate in the UX and must be respected by the SDK, extension, app, and devices.json tracks.
If an API name diverges, the track that owns the code decides the name; the behavior is what matters.

### R1. One window per signature, with the certificate chosen in it

PAdES requires the certificate **before** the digest (the `signing-certificate-v2` attribute goes into the signed
attributes). That is why the SDK's `sign()` receives a function that prepares the digest for the chosen certificate:

```ts
const result = await websign.sign({
  hash: "SHA-256",
  prepare: async (certificate) => buildSignedAttrsDigest(certificate), // Uint8Array of 32/48/64 bytes
});
// result: { certificate, chain?, signatureAlgorithm, signature }
```

Flow: the window opens → the preselected certificate goes to the page → the page returns the digest → the window
shows the verification code. If the user switches certificates, the app asks for a new digest (`prepare` runs
again) and the code changes.
**Why:** a single window per signature; the "choose a certificate in one window and sign in another" pattern doubles
the clicks of someone who signs 30 reports a day.

### R2. `certificates()` returns only what the user chose

- **Unremembered** site: `certificates()` opens the Confirmation window in **Choose** mode ([§4.10](#410-choose-mode-and-site-permission))
  and returns only the chosen certificate.
- **Remembered** site: returns, without a window, the certificates already used on that site (normally one).
- It never returns the computer's full list.

**Why:** in a clinic the computer is shared; the full list would hand other physicians' names and CPFs
to any site.

### R3. `fingerprint(digest)` in the SDK

The SDK exports `fingerprint(digest): { text: string; colorIndex: number; cells: boolean[] }` with the same
algorithm as [§4.4](#44-verification-code), so the site can show the same code and the same drawing near
its "Sign" button. `TODO(gustavo)`: Diagnos shows the code next to "Waiting for confirmation in WebeSign".

### R4. The site does not put text in the window

The protocol has **no** "reason", "document name", or similar field displayed in the window.
**Why:** site text inside our window would borrow our credibility (P1); the document context
stays on the site's page, linked by the verification code.

### R5. The extension reports origin, frame, and browser

Every request reaches the app with: the origin of the frame that called the SDK (from the `MessageSender`, never from the payload), the origin
of the top-level tab, and the browser's name and version. Non-local `http:` origins, `file:`, `data:`, and extension
origins are rejected by the extension with `InsecureOrigin` before reaching the app.

### R6. Optional filter requested by the site

The request may carry `accept: { algorithms?: ("ECDSA"|"RSA-PKCS1"|"RSA-PSS")[] }`. Incompatible certificates
appear disabled with "Not compatible with this request" ([§5.8](#58-what-goes-into-the-list)).
`TODO(gustavo)`: is a policy filter (e.g. ICP-Brasil only) left for later?

### R7. devices.json fields the interface consumes

```jsonc
{
  "id": "safenet-etoken-5110",
  "name": "SafeNet eToken 5110",          // commercial name displayed
  "kind": "token",                         // "token" | "card" | "reader"
  "match": { "usb": ["0529:0620"], "atr": [] },
  "driver": {
    "name": "SafeNet Authentication Client",
    "download": { "windows": "https://…", "macos": "https://…", "linux": "https://…" },
    "pkcs11": { "windows": "eTPKCS11.dll", "macos": "/usr/local/lib/libeTPkcs11.dylib", "linux": "/usr/lib/libeTPkcs11.so" }
  },
  "macos": { "cryptotokenkit": false },    // false → needs the Add-on (see §7)
  "pinUnlock": { "tool": "SafeNet Authentication Client" }   // where to unlock with the PUK
}
```

A missing field never blocks anything; it only swaps the specific hint for the generic one.

### R8. The app knows how to map certificate → device

To say "SafeNet eToken 5110 token" or "Card in reader Identiv", the app links each certificate to its
device: on Windows through `NCRYPT_READER_PROPERTY`/the reader name, on Mac through `kSecAttrTokenID`, on
PKCS#11 through the slot description. Without a safe mapping, the text falls back to "Token or card".

---
## 4. Confirmation window

The UI strings quoted in this section are the English copy (`en` column of [§13](#13-copy-and-i18n)); the pt-BR
equivalents live in the same table.

### 4.1 When it opens, size, and position

| Item | Decision | Why |
|------|---------|---------|
| Size | **480 × 600** logical px, fixed, not resizable | Fits 1366×768 at 100% (usable area ~728 px) with the title bar; 480 gives enough width for name + badge on the same line. |
| Small screen | If the usable area is under 640 px tall, the window uses `usable area − 40` as its height; header and footer stay fixed and the body scrolls | Never hide the buttons. |
| Position | Centered on the monitor where the pointer is | That is where the user just clicked "Sign" on the site; centering (rather than opening under the pointer) keeps the button from appearing under the cursor. |
| Foreground | Always on top while it waits for a decision; opaque; no minimize or maximize; close (X) = Cancel | A signature request must not get lost behind the browser. |
| Window title (OS) | "Sign for {site} — WebeSign" (`confirm.window_title`) | A screen reader announces the site on focus; the taskbar shows who asked. |
| Browser name | Short name: Chrome, Edge, Firefox, Brave, Safari, Chromium ("via Chrome") | Fits in the eyebrow next to the chip; the full name goes in the accessible name. |
| Theme | Follows the system (light/dark) | Consistency with the OS. |
| Source of truth for the site | Origin sent by the extension ([R5](#r5-the-extension-reports-origin-frame-and-browser)) | P1. |

**Decision (position):** the window is centered on the monitor winit reports for it (`current_monitor`, else the
primary), once, when the process creates it; between queued requests it is hidden and shown again where it was.
winit 0.30 / eframe 0.36 expose no global pointer position, so "the monitor with the pointer" needs a platform call
(`GetCursorPos` + `MonitorFromPoint`, `NSEvent.mouseLocation`, `XQueryPointer`; Wayland gives none) in
`app/src/platform/` and a `ViewportCommand::OuterPosition` before showing. Until then the OS's own placement of a
new top-level window (usually the active monitor) applies.

**Decision (OS title):** a host longer than 40 characters is cut from the **left** with "…" in the OS title
(taskbars cut titles at the end, where the registrable domain is); the registrable domain and the port are never
cut, even if that exceeds 40.

`TODO(gustavo)`: prove in Phase 0 whether Windows hands focus to the window when the host is already running
(connection kept open). If it does not: window on top, `FlashWindowEx`, and the button's arming only starts
when the user clicks the window (the first click only focuses, it triggers nothing).

### 4.2 Anatomy (Sign mode)

```
 480 px ──────────────────────────────────────────────────────────────
┌────────────────────────────────────────────────────────────────────┐ OS bar: "Sign for app.diagnos.health — WebeSign"
│ HEADER · bg-surface · padding 20/24/16 · bottom border border        │
│  [signature 16] Signature request · via Chrome   [✓ Allowed site]    │ text-caption fg-muted · permission chip on the right (22 px)
│  https://app.diagnos.health                                         │ text-headline: scheme+subdomain fg-subtle 400, domain fg 600
│  wants you to sign a document.                                      │ text-body, fg-muted
│  (origin warning line, only when present — §4.3)                    │ compact warning-soft notice, full width
├────────────────────────────────────────────────────────────────────┤
│ BODY · bg-canvas · padding 16/24 · gap 16 · scrolls if space is short │
│ ┌──────────────────────────────────────────────────────────────┐   │ code card: bg-surface, border, radius-lg, padding 12/16
│ │ [identicon 40]  Verification code                    SHA-256 │   │
│ │                 7F3A 9C21 E0B4 55D8                          │   │ text-code
│ │                 Check that the site shows the same code.     │   │ text-small, fg-subtle
│ └──────────────────────────────────────────────────────────────┘   │
│  Sign with                                                          │ text-caption, fg-muted (accessible label of the list)
│ ┌──────────────────────────────────────────────────────────────┐   │ list: bg-surface, border, radius-lg
│ │ (•) Ana Beatriz Souza                       [ICP-Brasil A3]   │   │ 72 px row (see §5.1)
│ │     CPF •••.456.789-•• · AC SOLUTI Multipla v5                │   │
│ │     [identification-card] Card in reader · Expires in 23 days │   │
│ ├──────────────────────────────────────────────────────────────┤   │
│ │ ( ) Ana Beatriz Souza                       [ICP-Brasil A1]   │   │
│ │ …                                                             │   │
│ ├──────────────────────────────────────────────────────────────┤   │
│ │ [caret-right] Can't sign (1)                                  │   │ 40 px row
│ └──────────────────────────────────────────────────────────────┘   │
│  (PIN block here, only for a key behind a driver — §4.6)            │
├────────────────────────────────────────────────────────────────────┤
│ FOOTER · bg-surface · 64 px · padding 0/24 · top border             │
│  Windows will ask for your PIN in its own window.  [Cancel] [Sign]  │ control-lg buttons (36); Sign min 112 px
└────────────────────────────────────────────────────────────────────┘
```

Layout rules:

- Header and footer are fixed; only the body scrolls. The list has a maximum height of **3 rows** (216 px) and scrolls
  within itself; with the PIN block visible, the maximum drops to **2 rows** and the selected row is kept
  visible.
- **Button order follows the platform.** Windows: `[Sign] [Cancel]` right-aligned. macOS and Linux:
  `[Cancel] [Sign]`. **Why:** the user clicks by muscle memory of their own system; reversing causes
  wrong clicks. The mockups have a "Windows / macOS and Linux" selector that swaps the order.
- The permission chip ("Allowed site" / "New site") sits on the eyebrow line, on the right; origin warnings
  sit on their own line right below the sentence, at full width. **Why:** the warning is more
  important than the status and must not be cut off; the status is short and fits in the eyebrow.
- Queue: the eyebrow becomes "Signature request 1 of 3" (`confirm.eyebrow_queue`) and the window title
  gains "(1 of 3)".
- The text on the left of the footer is the **next-step hint** (e.g. "Windows will ask for your PIN in its own window.",
  `pin.os_prompt`) or, in the last 30 s before the timeout, "This request expires in 28 s"
  (`footer.expires_in`).
- Everything in the mockup is reproducible in egui: no gradient, no blur, one shadow per frame.

### 4.3 Site origin

The origin is the most important element of the window. It comes from the extension ([R5](#r5-the-extension-reports-origin-frame-and-browser))
and is formatted like this:

1. **Split** scheme, subdomains, registrable domain (eTLD+1 by the Public Suffix List, `psl` crate), and port.
2. **Paint**: `https://` and subdomains in `fg-subtle` weight 400; registrable domain in `fg` weight 600; port
   (only if not the default) in `fg-subtle`. All in `text-headline` (20/26).
3. **Never cut the registrable domain.** If the host does not fit in 2 lines, cut subdomains from the
   **left** with "…" (`…secure.login.app.diagnos.health`). Hosts longer than 32 characters use
   `text-title` (16/22). **Why:** the classic scam is `diagnos.health.cadastro-medico.com`; what
   matters (`cadastro-medico.com`) is at the end.
4. **IDN**: if any label is not ASCII, show the **punycode** form (`xn--…`) as the main one and the Unicode form
   below in `fg-subtle` ("Displays as diаgnos.health", `origin.shown_as`), with a warning.
   **Why:** domains with accents are rare in our audience; being strict costs little and kills homographs.
5. **Frame**: if the request comes from an iframe whose origin differs from the tab's, add a warning line
   "Inside a page from {top_site}" (`confirm.inside_frame`).

Variants and warnings (the status chip sits in the eyebrow; the warning goes on its own line below "wants you to sign…"):

| Case | Example | Treatment | Key |
|------|---------|------------|-------|
| Normal https, remembered site | `https://app.diagnos.health` | success chip `check-circle` "Allowed site" | `confirm.site_remembered` |
| Normal https, new site | `https://app.diagnos.health` | neutral chip `info` "New site" (accessible name: "First time this site asks for anything on this computer") + "Remember this site" checkbox | `confirm.site_new` |
| Deceptive subdomain | `https://diagnos.health.cadastro-medico.com` | no extra warning; the domain highlight does the work; if new, "First time" chip | n/a |
| IDN / punycode | `https://xn--dignos-4nf.health` | warning `warning` "Address with special characters. Check it letter by letter." | `origin.warn_idn` |
| Public IP | `https://203.0.113.7` | warning "Numeric address with no site name" | `origin.warn_ip` |
| Local network IP | `https://192.168.0.20:8443` | warning "Local network numeric address" | `origin.warn_ip_local` |
| Local site | `http://localhost:5173`, `127.0.0.1`, `[::1]` | warning `terminal-window` "Local development site" | `origin.warn_localhost` |
| Non-local http | `http://laudos.exemplo.com` | **blocked** by the extension (`InsecureOrigin`); if it reaches the app because of a bug, error state without a Sign button | `origin.blocked_http` |

**Why block http:** same rule as WebAuthn (secure contexts only); anyone on the network could inject
a request. Hospital systems on an intranet IP keep working if they use https.

Warning alerts do **not** block and do not change the button; they are read by the screen reader along with the origin.
With any warning, the "Remember this site" checkbox is **unchecked and disabled** for IP and punycode
(the user can still sign, but cannot remember). **Why:** addresses that are hard to check must not
get silent access.

### 4.3.1 Desktop program as the caller

A desktop program (`websign connect`, `@websign/desktop`, `websign-client`) replaces the site origin with the
**program name** in the same headline slot; everything else in the window is unchanged.

| Element | Treatment | Key |
|---------|-----------|-----|
| Eyebrow | "Signature request · from a program on this computer" instead of "via {browser}" | `confirm.via_app` |
| Headline | The program's display name (executable name when it has none), `text-headline`, never cut | n/a |
| Line under the name | "Signed by {signer}" when the executable's signature verifies; otherwise a `warning` line "Unverified program. Only continue if you started it yourself." | `caller.signed_by`, `caller.unverified` |
| Chip | success "Allowed program" or neutral "New program" (accessible name: "First time this program asks for anything on this computer") | `confirm.app_remembered`, `confirm.app_new`, `confirm.app_new_a11y` |
| Remember checkbox | "Remember this program on this computer"; same help text and default (unchecked) as for sites | `consent.remember_app`, `consent.remember_help` |
| Window title (OS) | "Sign for {site} — WebeSign" with the program name in `{site}` | `confirm.window_title` |
| Revoke | Diagnostics › Browsers › "Allowed programs"; empty: "No remembered programs." | `sites.apps_section`, `sites.apps_empty` |

**Why:** the program name is self-declared, so the signature check and the "Unverified program" warning are the only
evidence the user gets; a program that is new or unverified never gets a silent pass.

### 4.4 Verification code

**Decision:** show the **first 8 bytes of the digest** in uppercase hexadecimal, in 4 groups of 4
(`7F3A 9C21 E0B4 55D8`, mono font), accompanied by a **mirrored 5×5 identicon** derived from the same bytes.
No emoji and no word list.

| Option evaluated | Result | Reason |
|----------------|-----------|--------|
| Full hex (64 characters) | ✗ | Nobody checks 64 characters. |
| Emoji-hash | ✗ | egui's emoji (embedded font) and the browser's (OS font) have different drawings; the user would compare drawings that do not match. |
| Word list | ✗ | Depends on the language; the site and the app could be in different languages; a per-language list weighs on the SDK. |
| **Short hex in groups + identicon** | ✓ | The identicon allows checking "at a glance" (shape + color); the hex allows checking exactly; both are reproduced in the SDK in a few lines and with no special font. |

**New callers see the certificate only after Continue (D11).** For a caller that is not remembered (new site or
new program), the window does not tell it which certificate is highlighted until the user decides:

- In `choosing`, the code card is replaced by the neutral hint "Choose the certificate and click Continue to see the
  verification code." (`code.continue_hint`) and the primary button reads "Continue" (`action.continue`) instead of
  "Sign". Continue obeys the same arming as Sign ([§4.7](#47-buttons-arming-and-accidental-click-prevention)): it
  works only 600 ms after the window is visible **and** focused, so a click that merely brings the window forward
  never releases a certificate. Enter never triggers Continue; a click or Space on the focused button does.
- Pressing Continue releases the chosen certificate to the caller, which prepares the document and returns the
  digest; the code card shows "Preparing the document…" and, when the digest arrives, the window moves to `ready`
  with the code visible, re-arms, and the button becomes "Sign".
- A digest that arrives before Continue, or for a certificate that is not selected, is ignored: the window never
  asked for it. Choosing another certificate after Continue brings back the hint and "Continue".
- A remembered caller skips this step: its certificate is already known, so the code appears as soon as the digest
  arrives.

**Why:** the certificate holds the person's name and ID; a caller that was never allowed must not learn it merely
because the window opened. Continue is the moment the user consents to share it.

Algorithm (identical in Rust and in the SDK, `R3`):

```text
b          = digest[0..8]
text       = hex_upper(b) in groups of 4, separated by a space     → "7F3A 9C21 E0B4 55D8"
colorIndex = b[0] >> 5                                             → 0..7 (identicon palette, §11.1)
bits       = (b[1] << 8) | b[2]                                    → uses the low 15 bits
cell(row r ∈ 0..4, column c ∈ 0..2) lit ⇔ bit (r*3 + c) of bits = 1
column 3 = column 1; column 4 = column 0                           (mirror)
```

Drawing: 40×40 frame with `bg-sunken` background, 1 px `border` border, `radius-sm`; 6 px cells with 1 px of
spacing, in the identicon palette color. The 8 colors have contrast ≥ 3:1 against `bg-surface` in both themes, so the
same drawing works in light and dark (and on the site).

Supporting text: "Check that the site shows the same code." (`code.help`). With no certificate selected (empty list
or only disabled ones) there is no digest and the code card **does not appear**; it enters with
`motion-base` when a certificate is selected. **Why:** an empty card takes up the space the empty
state needs for its actionable hint. A `SHA-256`/`SHA-384`/`SHA-512` badge
in `text-caption` on the right of the label. While the site prepares the digest ([R1](#r1-one-window-per-signature-with-the-certificate-chosen-in-it)):
code skeleton + "Preparing the document…" (`code.preparing`), shown only if it takes longer than 150 ms.

Accessibility: the identicon is decorative (hidden from AccessKit); the code is selectable text and is read
character by character ("7 F 3 A, 9 C 2 1, …", `code.a11y`).

### 4.5 Certificate section

The full list specification is in [§5](#5-certificate-list). In the confirmation window:

- Label "Sign with" (`certs.label_sign`); in Choose mode, "Choose a certificate" (`certs.label_select`).
- **One certificate**: the row appears without the selection circle (the only one is already chosen); the rest is the same.
- **Several**: radio group; ↑/↓ changes the selection.
- **None**: empty state + [Possible certificates](#6-possible-certificates).
- Switching certificates **re-arms** the Sign button (600 ms) and asks the site for a new digest (R1).

### 4.6 PIN

| Key origin | Who asks for the PIN | What our window shows |
|-----------------|-----------------|-----------------------------|
| Windows (CNG/CAPI) or Mac (Keychain/CryptoTokenKit) | The system's or the middleware's window | Footer hint: "Windows will ask for your PIN in its own window." (`pin.os_prompt`). After the click: "Enter your PIN in the Windows window." (`pin.os_prompt_now`). Our window stops being "always on top" while the system asks for the PIN (the PIN window is its child through `NCRYPT_WINDOW_HANDLE_PROPERTY`). |
| Token driver (PKCS#11), regular keyboard | Our PIN field | PIN block (below). |
| Token driver with a keypad on the reader (`CKF_PROTECTED_AUTHENTICATION_PATH`) | The reader's keypad | No field. Before the click: "After you click Sign, enter your PIN on the reader's keypad." (`pin.pinpad_before`). After: card with `dots-nine` and "Enter your PIN on the reader's keypad." (`pin.pinpad_now`). |
| Token driver already authenticated in this session (and a key without `CKA_ALWAYS_AUTHENTICATE`) | Nobody | `lock-key-open` chip "Token unlocked for this session" (`pin.unlocked_session`). |

PIN block (token driver):

```
 Token PIN                                            4 to 16 characters     ← text-caption fg-muted / text-small fg-subtle
┌────────────────────────────────────────────────────────────────[eye]┐     ← control-md 32 field, bg-sunken, border-strong border
│ ••••••                                                              │
└─────────────────────────────────────────────────────────────────────┘
 [shield-check] Your PIN stays on this computer and never goes through the browser.     ← text-small fg-subtle (disappears when there is an error)
```

- Label: "Token PIN" or "Card PIN" (by the device's `kind`); limits from
  `CK_TOKEN_INFO.ulMinPinLen/ulMaxPinLen` ("4 to 16 characters", `pin.length_hint`). Sign only enables when the
  length is within the limit.
- `eye`/`eye-slash` button to show the PIN (`pin.show`/`pin.hide`), off by default.
  **Why:** a wrong PIN locks the token and costs a trip to the CA; seeing what was typed reduces lockouts.
- Enter in the field = Sign (if armed and of valid length).
- Security: buffer zeroed right after `C_Login` (`zeroize` crate); on Mac, `EnableSecureEventInput` while the
  field has focus; the value never goes to AccessKit (`PasswordInput` role, hidden value); no copy/paste
  out of the field.
- **Incorrect PIN** (`CKR_PIN_INCORRECT`): field emptied and focused, `danger` border, message below with
  `x-circle`. PKCS#11 does not report the exact number of attempts; we use the token's flags:

  | Flag after the error | Message | Key |
  |------------------|----------|-------|
  | none | "Incorrect PIN." | `pin.incorrect` |
  | `CKF_USER_PIN_COUNT_LOW` | "Incorrect PIN. Only a few attempts left before the token locks." | `pin.incorrect_low` |
  | `CKF_USER_PIN_FINAL_TRY` | "Incorrect PIN. Last attempt: one more mistake locks the token." (text in `danger`, weight 600) | `pin.incorrect_final` |

  **Why:** promising "3 attempts left" would be inventing a number that the standard does not provide.
- **Locked PIN** (`CKR_PIN_LOCKED` or `CKF_USER_PIN_LOCKED`): the field disappears; a `danger` notice appears with
  `lock-key` (fill): title "PIN locked" and text "The token locked after too many wrong attempts.
  Unlock it with the PUK in SafeNet Authentication Client or contact AC SOLUTI." (`pin.locked_body`, using
  `pinUnlock.tool` from devices.json and the issuer). The certificate row gains the reason "PIN locked" and is
  disabled; Sign is disabled. If the user cancels in this state, the SDK receives `PinLocked`.
- System key with a wrong PIN: the system shows its own error; if `SCARD_W_WRONG_CHV` comes back, we show "Incorrect PIN."
  in the footer and the user clicks Sign again; if `SCARD_W_CHV_BLOCKED` comes back, PIN locked state.
- There is no Caps Lock warning: egui/winit does not expose the key's state reliably.

**Decision:** the OS named in the footer hint is the **key's store**, not the running system: a key in the
Windows store says "Windows", a Keychain/CryptoTokenKit key says "macOS". A token driver (PKCS#11) has no system
dialog (`C_Login` needs the PIN from us), so a driver key the host marks "system PIN" gets our field with no stated
length limits; Linux therefore never shows the OS hint. The window drops "always on top" only while an OS dialog
is asking (`signing` with a store key).

### 4.7 Buttons, arming, and accidental-click prevention

| Rule | Value | Why |
|-------|-------|---------|
| Sign button arming | **600 ms** counted from the moment the window is visible **and** focused | Longer than the default double-click interval (500 ms on Windows and macOS): the second click of the site's "Sign" never lands on our button. |
| Re-arm | Whenever the window loses and regains focus, the certificate changes, the digest changes, or a queued request takes over | New content under the cursor demands a new reading. |
| Input during arming | Keys and clicks are **discarded** (except Esc) | An Enter key the user was typing on the site does not become a signature. |
| Valid click | Press **and** release inside the button, with the press after arming | Avoids "press before, release after". |
| Arming visual | Disabled button that transitions to enabled in `motion-base` (180 ms) when armed; no progress bar | Informs without drawing attention. |
| Double-click on a row | Only selects | No list gesture signs. |
| Window | Opaque, on top, position set by the app | The site controls nothing about the window. |

Primary button states: `Sign` (disabled → armed) · `Signing…` with egui's `Spinner` ·
on a recoverable error, `Try again`. In Choose mode: `Use this certificate` (`action.use_cert`).

Cancel stays enabled at all times, except during signing through the system or the reader's keypad (neither
API allows aborting); in that case it shows `Please wait…`.

**Decision (moving the selection):** "discarded during arming" applies to keys and clicks that land on a window
just shown or just refocused. Once the window has been armed since it last gained focus or took a request, a
selection change (click, ↑/↓, Home/End) is taken even while the re-arm it caused runs: a selection approves
nothing and re-arms Sign, and otherwise each arrow would wait 600 ms for the previous one. Every other key typed
before arming (Enter, Space, Tab, text, paste, IME) is dropped before any widget sees it and its text is wiped;
Esc and key releases always pass.

### 4.8 State machine

| State | What it shows | Exits |
|--------|--------------|--------|
| `loading_certs` | 2-row skeleton after 150 ms; "Looking for certificates…"; after 2 s, "Still reading {device}. Token drivers can take a few seconds." | → `choosing`, `empty` |
| `empty` | No code card; empty state + possible certificates ([§6](#6-possible-certificates)); "Open diagnostics" on the left of the footer; Sign disabled; focus on Cancel | → `choosing` (token inserted), Cancel → `NoCertificates` |
| `choosing` | List; pending digest shows "Preparing the document…"; for a new caller before Continue, the hint `code.continue_hint` and a "Continue" button ([§4.4](#44-verification-code), D11) | → `ready` (digest arrived), Cancel |
| `ready` | Code visible; Sign arms in 600 ms | → `signing`, Cancel → `UserCancelled` |
| `pin_error` | PIN block with message (§4.6) | → `signing`, `pin_locked`, Cancel |
| `pin_locked` | Lockout notice; row disabled | Choose another → `ready`; Cancel → `PinLocked` |
| `signing` | "Signing…" button; system/reader keypad hint; list and PIN disabled | → `success`, `pin_error`, `error` |
| `success` | Body swaps to a 48 px `check-circle` (success), "Signed", "The signature was sent to {site}."; the result has already been sent to the site | Closes by itself after **900 ms** |
| `error` | `danger` notice above the list with title, text, action, and a collapsed "Technical details" (copyable code) | Action from [§15](#15-errors); Cancel → error code |
| `site_cancelled` | "{site} cancelled the request." (the tab closed or navigated) | Closes after 1.5 s |
| `timeout` | After **5 min** without a decision | Closes; SDK receives `Timeout` |
| `blocked_origin` | Only through a bug (R5): `InsecureOrigin` error, no Sign | Close |

**Decision (`loading_certs` timing):** the 150 ms skeleton and the 2 s "Still reading {device}" are counted by the
UI model from the start of the listing (the request opening, or "Scan again"), so they appear in tests and screen
readers exactly as drawn. The device name comes from the host's `SlowListing` command; until the host sends it,
only "Looking for certificates…" shows.

**Why close on success:** the user's next step is on the site; 900 ms is enough to see that it worked.
The result is sent before the animation, so the site does not wait for it.

Live changes (PC/SC events):

- **Token inserted**: a new row enters at the end of the usable group with a height animation (`motion-base`);
  the selected row **does not move**; the button re-arms. The screen reader announces "Certificate found: {name}"
  (polite live region).
- **Token removed** with the row selected: the row stays in place, disabled, with "Removed. Plug the
  token back in." (`cert.reason.removed`); Sign is disabled. When it comes back, the row returns selected.
- **Token removed during `signing`**: `TokenRemoved` error.

### 4.9 Keyboard and focus

| Key | Effect |
|-------|--------|
| Esc | Cancel, always (including during arming). |
| Tab / Shift+Tab | List → "Details" of the selected row → PIN → show PIN → "Remember this site" → Cancel → Sign (in the platform's visual order). |
| ↑ / ↓, Home / End | Move the selection in the list, skipping disabled rows (which stay focusable for reading). |
| Enter on a row | Does not sign. Moves focus to the PIN (if the key asks for our PIN) or to the Sign button. |
| Enter in the PIN field | Signs (armed + valid length). |
| Enter / Space on the button | Activates the focused button. |
| Ctrl/⌘+C on the code | Copies the code (selectable text). |

Initial focus: (1) PIN field, if the preselected certificate asks for our PIN; (2) otherwise, the selected
row; (3) with no certificates, Cancel. **Never** on the Sign button.
**Why:** the window appears while the user may be typing; focus on Sign would turn a stray Enter
into a signature.

### 4.10 Choose mode and site permission

A **new** site (no remembered permission) always sees the checkbox:

```
[ ] Remember this site on this computer                         ← checkbox, control-sm
    It will be able to see which certificate you use without asking.    ← text-small fg-subtle
    Every signature still asks for your confirmation.
```

- **Unchecked by default.** **Why:** remembering gives the site a new power; the safe default is not to grant it.
- Appears in Sign mode (below the list) and in Choose mode.
- Disabled for IP and punycode (§4.3).
- What "remember" grants: `certificates()` without a window for the certificates already used on the site, and
  preselection of the last certificate used on the site. Signing **always** asks for confirmation.
- Revoke: Diagnostics › Browsers › "Allowed sites" ([§8.3](#83-browsers-tab)).

**Choose mode** (`certificates()` from an unremembered site, R2): same window, with these differences:

- Window title: "Choose certificate for {site} — WebeSign"; eyebrow "Certificate request";
  sentence "wants to know which certificate you will sign with." (`confirm.asks_select`).
- In place of the code card: neutral `info` notice "The site will receive the name, type, issuer and validity of the
  chosen certificate. Nothing is signed now." (`consent.select_shares`).
- There is no PIN.
- Primary button "Use this certificate"; same 600 ms arming.
- Success: "Certificate sent to {site}" for 900 ms and closes.

### 4.11 Queue, timeout, and a site that gave up

- **Queue**: requests that arrive while the window is open enter a queue (max 10; above that, `Busy`). The
  eyebrow shows "Signature request 1 of 3" (`confirm.eyebrow_queue`). When the current one finishes, the next takes over with a
  `motion-base` transition and re-arming. Requests from different sites never get mixed.
- **Timeout**: 5 min without a decision → closes with `Timeout`. Countdown in the footer during the last 30 s.
- **Site gave up**: the extension's port closes (tab closed or navigation) → `site_cancelled` state.

`TODO(gustavo)`: batch signing (several reports, one confirmation with N codes) conflicts with "confirmation on
every signature"; decide whether it goes in and with what limit.

---

## 5. Certificate list

This is the heart of the UX: the physician must recognize **their** certificate in one second, even on a
shared computer.

### 5.1 Row anatomy (72 px)

```
┌──────────────────────────────────────────────────────────────────────┐
│ (•)  Ana Beatriz Souza                              [ICP-Brasil A3]    │  line 1: text-body-strong fg · badge on the right
│      CPF •••.456.789-•• · AC SOLUTI Multipla v5                        │  line 2: text-small fg-muted
│      [identification-card] Card in reader · Expires in 23 days  Details│  line 3: text-small; colored validity; "Details" only on the selected row
└──────────────────────────────────────────────────────────────────────┘
 padding 10/16 · lines 20 + 16 + 16 with no extra space (= 72) · radio 16 px · gap 12 · the whole row is the click target
```

- Selected: `accent-soft` background, filled `accent` radio with an `on-accent` dot. No colored side bar.
- Hover: `bg-hover`. Keyboard focus: 2 px `focus` ring inside the row.
- Disabled: text in `fg-subtle`, no radio, reason on line 3 in place of the validity (e.g. `x-circle`
  "Expired on 10 May 2026" in `danger`).
- Text that does not fit is truncated with "…" at the end (name and issuer); the full text goes in the hover
  tooltip and in the accessible name.
- On line 3, **the validity is never cut**: the location shrinks instead ("Token SafeNet eToken 5110 · via
  dri…"). **Why:** the validity is state and decides the choice; the location is secondary identification.

### 5.2 Holder name

1. Take the subject's `CN`. If missing, `givenName + surname`; if missing, `O`.
2. ICP-Brasil: remove the `:{11 or 14 digits}` suffix (`ANA BEATRIZ SOUZA:12345678909` → `ANA BEATRIZ SOUZA`).
3. If the name is all uppercase, convert to Title Case: the particles `da, das, de, di, do, dos, du, e,
   del, la, van, von, y` in lowercase (except at the start); corporate suffixes `LTDA → Ltda`, `S.A.`, `ME`,
   `EPP`, `EIRELI` preserved in their official spelling. Otherwise, keep as is.
4. Never invent a professional title: the certificate has no "Dr."; the row shows "Ana Beatriz Souza".

### 5.3 Type (badge)

A single badge per row, neutral (`bg-sunken`, `fg-muted`, `radius-sm`, `text-caption`). **Why neutral:** color
is reserved for state (validity, error); type is not state.

First rule that matches:

| # | Condition | Badge (pt-BR / en) |
|---|----------|-------------------|
| 1 | Cartão de Cidadão issuer (issuer `CN` contains "Cartão de Cidadão") | "Cartão de Cidadão" / "Cartão de Cidadão" |
| 2 | DNIe issuer (issuer `O` = "DIRECCION GENERAL DE LA POLICIA") | "DNIe" / "DNIe" |
| 3 | ICP-Brasil policy `2.16.76.1.2.{1,2,3,4}.*` | "ICP-Brasil A1…A4" |
| 4 | ICP-Brasil policy `2.16.76.1.2.{101..104}.*` | "ICP-Brasil S1…S4" |
| 5 | Subject with `O=ICP-Brasil` and no recognized policy | "ICP-Brasil" |
| 6 | `qcStatements` with `QcCompliance` (0.4.0.1862.1.1) **and** `QcSSCD` (0.4.0.1862.1.4) | "Qualificado eIDAS" / "Qualified eIDAS" |
| 7 | `QcCompliance` without `QcSSCD` | "eIDAS" / "eIDAS" |
| 8 | Anything else | "Certificado" / "Certificate" |

`TODO(gustavo)`: check the OID table against the current DOC-ICP-04 before release.
The national issuer table (rows 1–2) lives in the app as data (`cert_kinds.rs`), not in devices.json,
because it describes certificates and not devices.

### 5.4 Short issuer

The issuer's `CN`; if missing, `O`. No invented abbreviations ("AC SOLUTI Multipla v5", "AC Certisign RFB G5").
Truncated at the end.

### 5.5 Document (CPF/CNPJ)

| Source | Display | Why |
|-------|----------|---------|
| ICP-Brasil individual: `otherName 2.16.76.1.3.1` (8-digit birth date + 11-digit CPF + …) | `CPF •••.456.789-••` | The government's standard mask (shows the 4th to 9th digit): distinguishes people without exposing the whole CPF. |
| ICP-Brasil company: `otherName 2.16.76.1.3.3` (14-digit CNPJ) | `CNPJ 12.345.678/0001-90` in full | The CNPJ is public data at the Receita. |
| Subject `serialNumber` with an ETSI prefix (`PNOPT-`, `IDCPT-`, `IDCES-`, …) | `ID •••••123` (last 3) | A national ID number is personal data. |
| None | Line 2 with the issuer only | n/a |

Screen reader: "CPF partially hidden, 456 789" (`cert.doc_cpf_a11y`). The full CPF never appears anywhere
in the interface, logs, or diagnostics.

### 5.6 Validity

| Situation | Text | Color | Icon |
|----------|-------|-----|-------|
| More than 30 days | "Valid until 14 Mar 2027" | `fg-muted` | n/a |
| 8 to 30 days | "Expires in 23 days" | `warning` | `clock` |
| 2 to 7 days | "Expires in 5 days" | `danger` | `clock` |
| Tomorrow / today | "Expires tomorrow" / "Expires today" | `danger` | `clock` |
| Expired | "Expired on 10 May 2026" (disabled row) | `danger` | `x-circle` |
| Not yet valid | "Valid from 1 Oct 2026" (disabled row) | `warning` | `clock` |

Dates in local time; format per language (`dd/mm/yyyy` in pt; `14 Mar 2027` in en).
**Why 30 days:** it is the practical lead time to renew an A3 (scheduling validation at the CA).

### 5.7 Where the certificate is (origin in plain language)

| Technical situation | Text | Icon |
|------------------|-------|-------|
| Software key in the system store (imported A1) | "On this computer" | `desktop` |
| Hardware key, known device (devices.json, `kind: token`) | "Token SafeNet eToken 5110" | `usb` |
| Key on a card in a reader | "Card in reader" (+ reader name in Details) | `identification-card` |
| Hardware key without a safe mapping | "Token or card" | `usb` |
| Any of the above, accessed through the driver (PKCS#11) | appends " · via driver" | n/a |

**Why the "· via driver":** it explains why the PIN field appears in our window (and not the system's);
beyond that, the technical path stays in Details.

### 5.8 What goes into the list

**Hidden** (never in the list; counted in Diagnostics › Certificates):

1. No associated private key.
2. `basicConstraints cA=true`.
3. `keyUsage` present without `digitalSignature` or `nonRepudiation`.
4. `extendedKeyUsage` present with only unrelated purposes (`serverAuth`, `codeSigning`, `timeStamping`,
   `OCSPSigning`). These pass: absence of EKU, `anyExtendedKeyUsage`, `clientAuth`, `emailProtection`,
   `documentSigning` (1.3.6.1.5.5.7.3.36), MS Document Signing (1.3.6.1.4.1.311.10.3.12).
5. Sibling authentication certificate: same holder **and** same device as a certificate with
   `nonRepudiation`, itself having only `digitalSignature` (this is the case for the Cartão de Cidadão and the DNIe, which
   carry a login certificate and a signing certificate). **Why:** it keeps the physician from choosing the login one and
   producing a non-qualified signature.

**Disabled** (visible in the "Can't sign" group, with a reason): expired, not yet valid, PIN locked,
incompatible with the algorithm requested by the site (R6), removed while the window was open.
**Why show the expired one:** "my certificate vanished" is the most common support ticket, and it is almost always
expiry; showing it with the reason solves it without support.

### 5.9 Ordering and initial selection

1. Last certificate used **on this site** (only for a remembered site).
2. Remaining usable ones by most recent use on any site.
3. Never used: A3/qualified before A1/software; then name (A→Z); then longest validity.
4. "Can't sign ({n})" group at the end, collapsed.

Initial selection: item 1; otherwise the first usable one; otherwise none. While the window is open, new
rows enter at the end and **nothing reorders** (against wrong clicks).

### 5.10 Grouping

In the Confirmation there are **no** groups by technical origin; only the collapsed "Can't sign" group.
**Why:** the physician thinks "my certificate", not "Windows store vs driver".
In Diagnostics › Certificates there are groups by origin (that is where IT looks).

### 5.11 Deduplication

Key: SHA-256 of the certificate's DER. The same certificate seen by the system and by the driver becomes **one**
row, using the system path (decision 2 of the project brief). Details shows "Also available through the token
driver". If signing through the system fails with a driver error (not PIN, not cancellation), the error offers
"Try through the token driver" (`errors.driver_failure.alt_path`).

### 5.12 Certificate details

The "Details" link (only on the selected row) expands the row in `motion-base` with label/value pairs in
`text-small`:

| Label | Value |
|--------|-------|
| Holder | Full CN as it is in the certificate |
| Issuer | Issuer DN (CN, O, C) |
| Validity | "3 Mar 2025 to 22 Oct 2026" |
| Usage | "Digital signature, Non-repudiation" |
| Key | "RSA 2048" / "ECDSA P-256" |
| Fingerprint (SHA-256) | 16 groups of 4 hex in `text-mono`, with a `copy` button |
| Access | "Windows (certificate store)" / "macOS Keychain" / "Token driver: C:\Windows\System32\eTPKCS11.dll" |
| (if deduplicated) | "Also available through the token driver" |

"View in system" button (`cert.details.view_in_system`): opens the OS viewer
(`CryptUIDlgViewContext` on Windows, `SFCertificatePanel` on Mac, `gcr-viewer` on Linux if present).

**Decision:** the button is not shown yet. Every OS viewer needs the certificate's DER, and the window only
receives the parsed `CertInfo`; the host holds the DER (`KeySnapshot.certificates`). Proposed contract: a
`UiEvent::ViewCertificate { key, fingerprint }` answered by the engine, which opens the viewer with the DER and
our window as the parent. The window never receives certificate bodies.

### 5.13 Many certificates

With more than 6 usable ones, a "Filter by name, ID number or issuer" field appears above the list
(`certs.filter_placeholder`, `magnifying-glass` icon). It filters by name, issuer, and the visible CPF digits.
No result: "No certificate matches “{query}”".

---

## 6. Possible certificates

A detected device that looks like a token or card but did not bring a certificate to the list.

### 6.1 What counts as a "possible certificate"

| Source | Included if | Dropped if |
|-------|----------|---------|
| USB (`nusb`) | VID:PID is in devices.json **or** it has a class `0x0B` (CCID) interface | any listed certificate was mapped to it (R8) |
| PC/SC reader | a card is present (ATR read) | same |

Ordinary USB devices (mouse, keyboard) never appear. When the certificate → device mapping is
uncertain, the hint does **not** appear in the Confirmation (avoids a false alarm) and appears in Diagnostics as
"We can't tell if it has certificates".

### 6.2 In the Confirmation

**Empty list**: the empty state becomes an actionable card (bg-surface, border, radius-lg, padding 16):

```
 [usb in a warning-soft circle 32]  SafeNet eToken 5110 connected
                                    No certificate showed up on this token. To use it,
                                    install SafeNet Authentication Client.
                                    [download-simple Download for Windows ↗]  [arrow-clockwise I installed it, scan again]
```

- The card's primary button opens the current OS's link in the default browser; the secondary one re-reads drivers and
  store (`common.refresh`).
- No link for the OS: "Look for the software on the website of the authority that issued your certificate."
  (`possible.no_link`).
- Unknown card in a reader: "There's a card in Identiv uTrust 2700 R, but we don't recognize it. If it holds a
  certificate, install the card vendor's software." + [Open diagnostics].
- On Mac, if the device needs the Add-on ([§7](#7-add-on-macos)), the card becomes two steps:
  "1. Install SafeNet Authentication Client" and "2. Install the WebeSign Add-on".
- With no device detected: only the empty state "No certificates found / Plug in your token or insert your card.
  The list updates by itself." + "Open diagnostics" link.

**List with certificates**: a compact 40 px row at the end of the list:
`[usb] SafeNet eToken 5110 connected with no certificates · How to fix` → expands the same card inside the list.
**Why:** the real case is "I renewed and got a new token, but the driver is not installed" while the old A1
keeps showing up.

### 6.3 In Diagnostics

Devices tab, on the device's row, with the same text and button ([§8.4](#84-devices-tab)).

---

## 7. Add-on (macOS)

It only exists if proof 3 shows that the App Store sandbox blocks PKCS#11. Everything below applies only to the
Mac App Store build.

**On-screen name:** "WebeSign Add-on" (pt-BR: "Complemento WebeSign"). Icon `puzzle-piece`.
Never use "sandbox", "Mach service", "helper", "background", or "daemon" in the interface.

When to suggest it (any of):

1. A detected device whose devices.json says `macos.cryptotokenkit: false`.
2. The user added a driver manually in Diagnostics.
3. A known device connected, with no certificates, with the vendor driver installed (the `pkcs11.macos` file exists).

Base text (`complement.body_needed`): "App Store apps can't use some token drivers. The free WebeSign Add-on, from the
same project, bridges that gap. It only runs when WebeSign asks for it."
**Why this text:** it explains the cause without blaming anyone, says it is from the same project (trust), and answers
in advance the fear of a "program running hidden".

| State | Where | Visual | Action |
|--------|------|--------|------|
| Absent and needed | Confirmation (possible-certificate card, step 2) and Diagnostics › Devices (card at the top) | `warning` notice | [Download Add-on] → site page with the `.dmg` |
| Installed and current | Diagnostics › Devices | `success` row "Add-on active · version 1.2.0" | n/a |
| Outdated | Diagnostics › Devices; Confirmation only if the error comes from it | `warning` notice "Update the Add-on (1.1.0 installed, 1.2.0 required)" | [Update Add-on] → opens the Add-on, which triggers Sparkle's check |
| Installed, not responding | Diagnostics › Devices | `warning` notice "The Add-on is installed but didn't respond." | [Open Add-on] |
| Not needed | n/a | nothing is shown | n/a |

---

## 8. Diagnostics window

### 8.1 Structure

| Item | Decision | Why |
|------|---------|---------|
| Size | **760 × 540** logical px, fixed; minimize allowed, maximize not | Fits 1366×768 at 100%; width for sidebar + 560 of content. |
| Opening | App icon in the Start menu/Launchpad/app menu; "Open diagnostics" in the popup and in errors | Single entry point for "something doesn't work". |
| Initial tab | The first with a red light; otherwise yellow; otherwise Browsers. Coming from "no certificates", Devices | Opens where the problem is. |
| Layout | 200 px sidebar (bg-surface, right border) + content (bg-canvas, padding 24, scrolls) | 4 tabs with status stay readable vertically. |

Sidebar, from top to bottom:

1. Brand (`seal-check` fill in a 24 px `accent` square, `radius-md`) + "WebeSign" (`text-title`) + version (`text-small`, `fg-subtle`).
2. **Overall traffic light** (chip): green "Ready to sign", yellow "Needs attention", red "Can't sign yet".
3. Tabs: icon + label + status icon on the right. Height 36, `radius-md`, selected in `accent-soft` with `accent-fg` text.
4. Footer: wide secondary button [copy Copy diagnostics].

Content header: tab title (`text-headline`) + subtitle (`text-body`, `fg-muted`) + action on the right
[arrow-clockwise Scan again] (F5 / Ctrl+R).

**Traffic light**: color + shape + text, never color alone:

| Level | Icon (fill) | Color | Means |
|-------|--------------|-----|-----------|
| Green | `check-circle` | `success` | Works. |
| Yellow | `warning` | `warning` | Something to fix, but signing is possible (or it is just a notice). |
| Red | `x-circle` | `danger` | Prevents signing. |
| Gray | `circle-dashed` (regular) | `fg-subtle` | Not applicable / no data. |

Rules per tab:

| Tab | Green | Yellow | Red |
|-----|-------|---------|----------|
| Browsers | ≥ 1 browser with a connected extension and every installed one without problems | ≥ 1 connected, but another without the extension/outdated | No browser connected, or app registration missing in all |
| Devices | Every detected device brought a certificate (or none detected) | Device with no certificate with a hint; added driver that does not load; outdated Add-on | `pcscd` stopped on Linux |
| Certificates | ≥ 1 usable and no usable one expires in ≤ 30 days | Some usable one expires in ≤ 30 days | No usable one |
| Help | no traffic light | n/a | n/a |

Overall = worst among the tabs.

### 8.2 Getting started

Until they are all green (and until the first test signature), a strip appears above the content of any tab,
with `list-checks` and "Getting started":

`[✓] App installed · [!] Browser extension · [✓] Certificate found · [ ] Test signature [Test a signature]`

"Test a signature" opens `{HOMEPAGE}test/` (the project site) in the default browser: the page generates 32 random bytes, calls
`sign()`, shows the **same verification code**, and the result ("It worked. Signed with Ana Beatriz Souza,
ICP-Brasil A3."). **Why:** it teaches the habit of checking the code before the first real report.
"Hide" (`onboarding.dismiss`) hides the strip forever.

### 8.3 Browsers tab

Subtitle: "Where the extension is installed and connected to the app."

**"Browsers on this computer" section**: one row per installed browser (Chrome, Edge, Firefox, Brave,
Chromium, Safari; detected by installation path/registry/LaunchServices, never by profile folders):

| State | Row | Action |
|--------|-------|------|
| Connected | `check-circle` "Extension 1.4.2 connected · 3 min ago" | n/a |
| Extension not detected | `warning` "Extension not detected" + hint "If Chrome shows “New extension added”, click Enable." (Chrome/Edge) | [Install extension ↗] opens the right store **in that** browser |
| Extension outdated | `warning` "Extension 1.2.0 · needs 1.3.0 or newer" + "Restart Chrome to update." | n/a |
| App registration missing | `x-circle` "Chrome can't find the WebeSign app" | [Repair] rewrites the manifest; afterwards "Repaired. Restart Chrome." |
| Safari with the extension off | `warning` "Extension turned off in Safari" | [Open Safari settings] (`SFSafariApplication.showPreferencesForExtension`) |
| Firefox Snap (Linux) | `info` "Firefox is installed as a Snap: on your first signature, allow access when the system asks." | n/a |

"Connected" uses the recorded pings (decision 8 of the project brief) with relative time ("3 min ago", "yesterday, 17:40").
Empty: "No supported browser found. WebeSign works with Chrome, Edge, Firefox, Brave and Safari."

**"Allowed sites" section**: origin with the registrable domain highlighted, "Remembered on 12 Sep 2026 · last
used today, 14:02", [Revoke] button (secondary). Revoking asks for inline confirmation (the button becomes "Confirm
revoke" for 4 s). Empty: "No remembered sites. When you tick “Remember this site” while signing, it shows up
here."

### 8.4 Devices tab

Subtitle: "Tokens, cards and drivers WebeSign can see right now." The list updates live (PC/SC).

1. **(Mac App Store only)** Add-on card at the top, when relevant ([§7](#7-add-on-macos)).
2. **(Linux only)** "Card service (pcscd)" row: `check-circle` "Running" or `x-circle` "Stopped" with the
   copyable command `sudo systemctl enable --now pcscd.socket`.
3. **Tokens and cards**: name (devices.json or "Smart card device"), `USB 0529:0620` in
   `text-mono` `fg-subtle`, and status: `check-circle` "2 certificates" · `warning` "No certificates · install
   SafeNet Authentication Client" [Download for Windows ↗] · `circle-dashed` "We can't tell if it has certificates".
4. **Card readers**: reader name; "No card" (gray) or "Card: G&D StarSign · 1 certificate" with
   the ATR in `text-mono` (truncated in the middle, `copy` button).
5. **Token drivers**: module name (`C_GetInfo.libraryDescription`), path in `text-mono`, status
   (`check-circle` "Loaded · 1 token", `circle-dashed` "Loaded · no token connected", `x-circle`
   "Failed to load: file not found"), origin ("Found automatically" / "Added by you"
   [Remove]). Below: [plus Add driver…] with "Only use this if your token vendor tells you to." It opens a file
   picker (`.dll`/`.dylib`/`.so`), loads it right away, and shows the result on the row itself.
   **Decision:** the picker is the OS's own: `IFileOpenDialog` (Windows), `NSOpenPanel` (macOS), `kdialog` on KDE
   and `zenity` elsewhere on Linux (`.so` also matches versioned names such as `libx.so.1`). Only when neither
   Linux helper exists (e.g. a Flatpak sandbox) the button reveals an inline path field ("Path to the driver file").

Empty: "No token or reader connected. Plug your token into a USB port; this list updates by itself."

### 8.5 Certificates tab

Subtitle: "Every certificate this computer offers, including the ones that can't sign."

- Action bar: [file-plus Import .pfx file…] + text "Windows imports the file and stores the
  certificate securely. You'll need the file's password."
  - Windows: `CryptUIWizImport` with our window as the parent (the Windows wizard does everything).
  - Mac: `NSOpenPanel` (titled "Choose the .pfx file to import", `.pfx`/`.p12`) → opens the file in Keychain Access,
    which asks for the password and imports into the login keychain.
  - Linux: button hidden; text "On Linux, use a certificate on a token or card." `TODO(gustavo)`: path for A1 on Linux.
  - When focus returns to the window, the list is re-read.
- Groups by origin: "In Windows" / "In the Mac Keychain" / "Through the token driver". Rows identical to those in the
  Confirmation (no radio), with "Details" on all of them.
- Collapsed "Can't sign ({n})" group with each one's reason: "Expired on …", "Not yet valid",
  "No private key on this computer", "Made for login, not for signing".
- Empty: "No certificates on this computer. Plug in your token, insert your card or import a .pfx file."

### 8.6 Help tab

Subtitle: "Quick answers and how to report a problem." (`help.subtitle`), as in the mockups.

1. **Common questions** (accordion; the first one open):
   - "My certificate doesn't show up" → checklist: token plugged in? driver installed (link to Devices)?
     certificate expired (link to Certificates)? A1 imported?
   - "I got the PIN wrong / the token locked" → explains the PUK, where to unlock, contacting the CA.
   - "What does WebeSign send to sites?" → only the signature of the verification code and the chosen
     certificate; never the PIN; never the document.
   - "Keyboard shortcuts" → table from §4.9 and §8.8.
2. **Report a problem**: exact preview of the diagnostics (`bg-sunken` box, `text-mono`, 8 lines,
   scrolling) + [copy Copy diagnostics] + [arrow-square-out Open support page ↗] (the project's issues).
   Text: "This is exactly what will be copied. It includes no names, ID numbers, sites or serial numbers."
3. **About**: "WebeSign 1.4.0 · protocol 1 · GPL-3.0-or-later" + [Source code ↗] [Privacy ↗].

### 8.7 "Copy diagnostics": exact content

Plain text, in English (it is for support and public issues), generated by a pure, testable function.

```text
WebeSign diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  chrome 129.0 · extension 1.4.2 · host registered · last ping 2026-09-29T14:02Z
  edge 129.0 · extension not seen · host registered
  firefox 131.0 · extension 1.4.2 · host registered · last ping 2026-09-28T20:40Z
devices:
  usb 0529:0620 safenet-etoken-5110 · certs 0
  reader "Identiv uTrust 2700 R" · atr 3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:.. · certs 1
pkcs11:
  %ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll · loaded · slots 1 · tokens 1
  %USERPROFILE%\Downloads\wdpkcs_icp.dll · failed: file not found (user-added)
certificates:
  usable 3 (os 3, pkcs11 0, deduplicated 1) · hidden 2 (expired 1, login-only 1)
  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 3
  expiring<=30d 2
complement: n/a
recent errors (last 20):
  2026-09-29T14:05Z sign PinIncorrect pkcs11 CKR_PIN_INCORRECT
```

**Included:** versions, OS, language, scale, render backend, browsers and extension state, VID:PID, reader
name, the matched device (or, for an unknown card, its ATR with the historical bytes masked), module paths with the user folder replaced by a variable (`%USERPROFILE%`, `~`), certificate counts
by type/algorithm/situation, technical error codes.
**Not included:** name, CPF, CNPJ, email, certificate serial number or fingerprint, token/USB serial
number, computer name, user name, sites (including remembered ones), digests, verification
codes. **Why:** the text ends up in a public issue; VID:PID identifies a model, not a person.

A card is shown by its matched device (`card safenet-etoken-5110`). For a card `devices.json` does not know, the line
shows the ATR up to its last interface byte (TS, T0, TA/TB/TC/TD) and `..` for every historical byte and for TCK
(as in the example above): the historical bytes of some cards carry a chip serial number, which is personal data.
If the ATR cannot be parsed, only its length is shown (`atr 19 bytes`). The same rule applies to any ATR on screen.

**Decision:** on Windows the `os:` line reads `Windows 11 (build 22631)` / `Windows 10 (build 19045)`, from
`RtlGetVersion` (Windows 11 is build ≥ 22000); the marketing version ("23H2") lives only in the registry and the
build number already identifies it.

### 8.8 Keyboard in Diagnostics

Ctrl/⌘+1…4 switches tab · ↑/↓ in the sidebar · F5 or Ctrl/⌘+R scans again · Ctrl/⌘+Shift+C copies the
diagnostics · Ctrl/⌘+W closes.

---

## 9. Extension popup

Plain TS + CSS, no framework, **< 15 KB** in total (HTML + CSS + JS + inline SVG icons).
Width **320 px**, height by content (max 480). Font: system stack (`--ws-font-popup`).
**Why the system font:** embedding Inter would cost ~50 KB, more than the whole popup, and the popup lives
inside the browser's interface, where the system font looks native.

Anatomy: header (20 px brand + "WebeSign") → status card (32 px icon + `text-title` title +
`text-body` `fg-muted` text) → wide primary button (`control-lg`) → secondary text action → footer
(`text-small` `fg-subtle`): "Extension 1.4.2 · App 1.4.0" and a "Privacy" link.

| State | Detection | Icon | Title | Text | Primary | Secondary |
|--------|----------|-------|--------|-------|----------|------------|
| `checking` | connection in progress (shown only after 150 ms) | spinner | "Checking…" | n/a | n/a | n/a |
| `ready` | host answered with version ≥ `MIN_APP_VERSION` | `check-circle` success | "Ready to sign" | "The WebeSign app 1.4.0 is connected to this browser." | Open diagnostics | n/a |
| `missing` | `connectNative` failed ("host not found") | `download-simple` accent | "The WebeSign app isn't installed" | "The extension needs the app on your computer to sign." | Download for Windows | "Already installed? Activate the app" |
| `outdated` | version < `MIN_APP_VERSION` or old protocol | `warning` warning | "Update the WebeSign app" | "You have version 1.1.0. This browser needs 1.3.0 or newer." | Update in Microsoft Store | n/a |
| `error` | host started but did not answer within 3 s / closed | `x-circle` danger | "The app didn't respond" | "Try again. If it keeps happening, restart your computer." + technical code in `text-mono` | Try again | Download again |
| `unsupported` | `runtime.getPlatformInfo().os` ∉ {win, mac, linux} | `info` | "WebeSign doesn't work on this system yet" | "Use a computer running Windows, macOS or Linux." | n/a | n/a |

**App installed but never opened (host not registered):** for the extension this is the same as "not installed"
(`connectNative` fails the same way). Resolution:

1. The package registers the `websign:` URL scheme **at installation** (MSIX `windows.protocol`, `CFBundleURLTypes`
   on Mac, `.desktop` with `x-scheme-handler/websign` on Linux). This does not require opening the app.
2. "Already installed? Activate the app" opens `https://<site>/activate/` in a tab. That page calls `websign:activate`
   (the browser asks "Open WebeSign?"), the app writes the manifests of all browsers and shows the
   Diagnostics window with "Getting started".
3. The `/activate/` page uses the extension's announcement on the page to show live "Looking for the app… → Ready".

**Why a page and not the popup:** the popup closes when the browser shows the protocol prompt;
the page stays open and follows the result.

Other rules: "Open diagnostics" sends `{type: "openDiagnostics"}` through native messaging and closes the popup;
download links pick the OS through `runtime.getPlatformInfo()` (Windows → Microsoft Store; Mac → Mac App
Store; Linux → the site's download page with .deb/.rpm); the extension icon gets the "!" badge (`warning`)
in `missing`/`outdated`/`error` and loses it in `ready`.

---

## 10. First run per operating system

| OS | What happens at installation | First open | Steps the user sees |
|----|------------------------------|-------------------|-------------------------|
| **Windows** (Microsoft Store, MSIX) | Registers the execution alias and the `websign:` scheme; nothing runs | The Store shows "Open". The app writes HKCU (host in every `NativeMessagingHosts` key + extension pre-registration) and opens Diagnostics with "Getting started" | 1. Extension: Chrome/Edge show "New extension added" → "click Enable"; Firefox → [Install extension ↗]. 2. Certificate found. 3. [Test a signature]. |
| **macOS** (Mac App Store) | Registers the `websign:` scheme; nothing runs | The app writes the manifests (Chrome, Edge, Brave, Firefox) and opens Diagnostics | 1. Safari: [Open Safari settings] to turn the extension on; when using it on the site, choose "Always Allow on This Website". Chrome/Edge/Firefox: [Install extension ↗]. 2. Certificate (and Add-on, if needed, §7). 3. [Test a signature]. |
| **Linux** (.deb/.rpm) | `postinst` writes system manifests (`/etc/opt/chrome/native-messaging-hosts`, `/usr/lib/mozilla/native-messaging-hosts`, …) and enables `pcscd.socket` | From the app menu, opens Diagnostics | 1. [Install extension ↗]; Firefox Snap: portal notice. 2. Is `pcscd` running? 3. Certificate. 4. [Test a signature]. |

Common rule: the app **never** opens by itself at login and does not stay resident (decision 4 of the project brief). Diagnostics
is the only "welcome screen".

---

## 11. Design tokens

Single source for `app/src/ui/theme/` (egui), `design/tokens.css` (popup and site), and for the mockups.
Names: `kebab-case` in the document and in CSS (`--ws-bg-canvas`), `snake_case` in Rust (`tokens.bg_canvas`).
An app test reads `design/tokens.css` with `include_str!` and compares each `--ws-*` with the Rust constant.
**Why a test and not a generator:** it keeps both files hand-readable and costs a 40-line test.

Visual direction: cool neutrals slightly pulled toward the brand blue; a single accent, "ink blue"
(the color of a signing pen), used only for the primary action, selection, and focus. Semantic color (green, amber,
red) is separate from the accent and reserved for state.

### 11.1 Colors

| Token | Light | Dark | Use |
|-------|-------|--------|-----|
| `bg-canvas` | `#F5F6F9` | `#0E1016` | Window background (body) |
| `bg-surface` | `#FFFFFF` | `#161922` | Header, footer, cards, list, sidebar |
| `bg-sunken` | `#EDEFF4` | `#0B0D12` | Fields, identicon well, badges, diagnostics box |
| `bg-hover` | `#F1F3F8` | `#1E222D` | Hover of rows and secondary buttons |
| `accent-soft` | `#EEF0FD` | `#1A1F3C` | Selected row/tab |
| `border` | `#DCE0E8` | `#2A2F3C` | Dividers and card outlines (decorative) |
| `border-strong` | `#848D9F` | `#6B7488` | Outline of field, radio, checkbox (≥ 3:1) |
| `fg` | `#141722` | `#E7E9EF` | Main text |
| `fg-muted` | `#545C6D` | `#A3AAB9` | Secondary text |
| `fg-subtle` | `#636B7D` | `#8C94A6` | Metadata, placeholders, scheme/subdomain |
| `accent` | `#3346D1` | `#4E5EE4` | Primary button background, checked radio |
| `accent-hover` | `#2A3BB8` | `#4555DA` | Primary hover |
| `accent-pressed` | `#2332A0` | `#3E4ECC` | Pressed |
| `accent-fg` | `#3346D1` | `#9DA8FF` | Links, accent icons, selected tab text |
| `on-accent` | `#FFFFFF` | `#FFFFFF` | Text over `accent` |
| `focus` | `#3346D1` | `#9DA8FF` | Focus ring (2 px) |
| `success` | `#127A41` | `#5BCB8D` | Success text/icon |
| `success-soft` | `#E6F5EC` | `#10281B` | Success chip/notice background |
| `success-border` | `#A8DCBE` | `#1F5C3B` | Success notice border |
| `warning` | `#935300` | `#EFB35E` | Attention text/icon |
| `warning-soft` | `#FFF2D9` | `#2A1F0E` | Attention chip/notice background |
| `warning-border` | `#EFCB8A` | `#5C4418` | Attention notice border |
| `danger` | `#BE242B` | `#FF868B` | Error text/icon |
| `danger-soft` | `#FDEBEC` | `#321519` | Error notice background |
| `danger-border` | `#F2B3B6` | `#6B2429` | Error notice border; a field's error border uses `danger` |
| `shadow-color` | `#1417221F` | `#0000008C` | Popover/tooltip shadow color |

Identicon palette (same in both themes, index 0–7): `id-0 #D6453A` · `id-1 #C4610A` · `id-2 #3B8A3B` ·
`id-3 #12857B` · `id-4 #2D78D2` · `id-5 #5A55D6` · `id-6 #9549C4` · `id-7 #C63E7B`. All ≥ 3:1 against
`bg-surface` and `bg-sunken` in both themes.

Measured contrast (WCAG 2.x):

| Pair | Light | Dark | Minimum |
|-----|-------|--------|--------|
| `fg` on `bg-surface` | 17.9 | 14.5 | 4.5 |
| `fg-muted` on `bg-canvas` | 6.2 | 8.2 | 4.5 |
| `fg-subtle` on the worst background (light: `bg-sunken`; dark: `accent-soft`) | 4.65 | 5.1 | 4.5 |
| `accent-fg` on `accent-soft` | 6.3 | 7.0 | 4.5 |
| `on-accent` on `accent` | 7.2 | 5.2 | 4.5 |
| `on-accent` on `accent-hover` | 8.7 | 5.9 | 4.5 |
| `success` / `warning` / `danger` on the `*-soft` | 4.8 / 5.5 / 5.3 | 7.7 / 8.7 / 7.2 | 4.5 |
| `border-strong` on `bg-surface` | 3.3 | 3.7 | 3.0 (component) |
| `focus` on `bg-surface` | 7.2 | 7.9 | 3.0 |

### 11.2 Typography

| Family | CSS token | Embedded files | Why |
|---------|-----------|--------------------|---------|
| Inter | `--ws-font-ui: "Inter", system-ui, -apple-system, "Segoe UI", Roboto, sans-serif` | Inter Regular 400, Medium 500, SemiBold 600 (OFL) | Project brief decision; great at small sizes. |
| JetBrains Mono | `--ws-font-mono: "JetBrains Mono", ui-monospace, "SF Mono", Consolas, monospace` | Regular 400 and Medium 500, ASCII **subset** (~20 KB) (OFL) | Distinguishes 0/O and 1/l in the verification code; egui has no tabular numerals in Inter. |
| System | `--ws-font-popup: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", sans-serif` | none | Popup only ([§9](#9-extension-popup)). |

| Token | Size/line (px) | Weight | Family | Use |
|-------|--------------------|------|---------|-----|
| `text-caption` | 12/16 | 500 | ui | Section labels, badges, chips |
| `text-small` | 12/16 | 400 | ui | Metadata, help, popup footer |
| `text-body` | 14/20 | 400 | ui | Default text |
| `text-body-strong` | 14/20 | 600 | ui | Holder name, browser/device name |
| `text-button` | 14/20 | 500 | ui | Buttons |
| `text-title` | 16/22 | 600 | ui | Section titles, popup title, long origin |
| `text-headline` | 20/26 | 600 | ui | Origin in the Confirmation, tab title |
| `text-code` | 18/24 | 500 | mono | Verification code (extra 0.5 px letter spacing) |
| `text-mono` | 12/16 | 400 | mono | VID:PID, ATR, paths, fingerprint |

No all-caps labels (uppercase in pt-BR gets heavy and the screen reader spells out acronyms).
**Why 14 body text:** the audience includes older physicians on office monitors; 12 is too small.

### 11.3 Spacing, sizes, and radii

| Token | Value | | Token | Value |
|-------|-------|-|-------|-------|
| `space-1` | 4 | | `control-sm` | 28 (checkbox, row buttons) |
| `space-2` | 8 | | `control-md` | 32 (fields, regular buttons) |
| `space-3` | 12 | | `control-lg` | 36 (footer and popup buttons) |
| `space-4` | 16 | | `row-cert` | 72 |
| `space-5` | 20 | | `row-compact` | 40 |
| `space-6` | 24 | | `row-diag` | 56 |
| `space-8` | 32 | | `icon-sm` / `icon-md` / `icon-lg` | 16 / 20 / 32 |
| `radius-sm` | 4 (badges, identicon) | | `identicon` | 40 |
| `radius-md` | 6 (buttons, fields, rows, tabs) | | `sidebar` | 200 |
| `radius-lg` | 10 (cards, list, notices, popup) | | `window-confirm` | 480 × 600 |
| `radius-full` | half the height (chips) | | `window-diagnostics` | 760 × 540 |
| | | | `popup-width` | 320 |

Shadows: only `shadow-popover` = offset 0/4, blur 16, spread 0, color `shadow-color` (tooltips and
menus). Cards use a border, not a shadow. **Why:** the window already has the OS shadow; inner shadows on everything
flatten the hierarchy.

### 11.4 Motion

| Token | Value | Use |
|-------|-------|-----|
| `motion-fast` | 120 ms | hover, press, focus ring |
| `motion-base` | 180 ms | expand/collapse, row entry, arming the button, tab switch |
| `motion-slow` | 260 ms | success check |
| `easing` | cubic-out (`emath::easing::cubic_out`) | all |
| `delay-arming` | 600 ms | Sign button |
| `delay-loading` | 150 ms | only shows loading if it takes longer than this |
| `delay-slow-hint` | 2000 ms | "Still reading {device}…" |
| `hold-success` | 900 ms | success before closing |
| `hold-site-cancelled` | 1500 ms | "site cancelled" before closing |
| `timeout-request` | 300 s | request without a decision |

Reduced motion: if the OS asks for it (`SPI_GETCLIENTAREAANIMATION` on Windows,
`accessibilityDisplayShouldReduceMotion` on Mac, GNOME's `enable-animations`, `prefers-reduced-motion` in the
popup), all animation durations become 0. Safety timers (`delay-arming`) and state timers
do not change. egui is in reactive mode: animation only repaints while `ctx.animate_*` is in progress.

### 11.5 Map to egui (`theme.rs`)

| egui field | Token |
|------------|-------|
| `Visuals::dark_mode` | system theme (`ctx.options.theme_preference = System`) |
| `panel_fill` | `bg-canvas` |
| `window_fill` | `bg-surface` |
| `extreme_bg_color` (`TextEdit` background) | `bg-sunken` |
| `faint_bg_color` | `bg-hover` |
| `code_bg_color` | `bg-sunken` |
| `hyperlink_color` | `accent-fg` |
| `warn_fg_color` / `error_fg_color` | `warning` / `danger` |
| `selection.bg_fill` / `selection.stroke` | `accent-soft` / 1 px `accent-fg` |
| `text_cursor.stroke` | 2 px `accent-fg` |
| `widgets.noninteractive` | `bg_fill` `bg-surface`, `bg_stroke` 1 px `border`, `fg_stroke` `fg` |
| `widgets.inactive` | `weak_bg_fill` `bg-surface`, `bg_stroke` 1 px `border-strong`, `fg_stroke` `fg`, radius `radius-md` |
| `widgets.hovered` | `weak_bg_fill` `bg-hover`, `bg_stroke` 1 px `border-strong`, `expansion` 0 |
| `widgets.active` | `weak_bg_fill` `bg-sunken`, `bg_stroke` 1 px `fg-subtle`, `expansion` 0 |
| `widgets.open` | same as `hovered` |
| `window_corner_radius` / `menu_corner_radius` | `radius-lg` / `radius-md` |
| `window_shadow` / `popup_shadow` | none (the OS draws it) / `shadow-popover` |
| `window_stroke` | 1 px `border` |
| `Spacing::item_spacing` | (`space-2`, `space-2`) |
| `Spacing::button_padding` | (`space-3`, 0) with height from `control-*` |
| `Spacing::interact_size` | (32, `control-md`) |
| `Spacing::icon_width` / `icon_spacing` | `icon-sm` / `space-2` |
| `Spacing::indent` | `space-4` |
| `TextStyle::Small / Body / Button / Heading / Monospace` | `text-small / text-body / text-button / text-headline / text-mono` |
| `TextStyle::Name("caption" \| "body-strong" \| "title" \| "code")` | the tokens of the same name |

Weights: register `FontFamily::Name("inter-medium")` and `FontFamily::Name("inter-semibold")` families
(egui does not synthesize weight). Line height through `TextFormat::line_height`. The primary button, chips,
radio, identicon, and focus ring are custom widgets in `app/src/ui/widgets/` (one per file), because
egui has no ready-made versions of these visuals.

---

## 12. Icons

Phosphor 2.1 through `egui-phosphor` (constants in `SCREAMING_SNAKE_CASE`, e.g. `regular::SEAL_CHECK`). In the
popup and on the site, inline SVG of the same files (`@phosphor-icons/core`). **Regular** weight by default;
**fill** only for status. Size 16 in a line of text, 20 in icon buttons and tabs, 32 in status cards.

| Concept | Icon | Weight |
|----------|-------|------|
| Provisional app brand (`TODO(gustavo)`: final logo) | `seal-check` | fill |
| Signature request (eyebrow) | `signature` | regular |
| Certificate request (Choose mode) | `certificate` | regular |
| Site with https | `lock-simple` | regular |
| Local development site | `terminal-window` | regular |
| Verification code (accessible label / Details) | `hash` | regular |
| Certificate (generic, Certificates tab) | `certificate` | regular |
| On this computer | `desktop` | regular |
| USB token / Devices tab | `usb` | regular |
| Card in reader / card reader | `identification-card` | regular |
| Token driver (PKCS#11) | `plug` | regular |
| Add-on (macOS) | `puzzle-piece` | regular |
| PIN (field label, screen reader) | `password` | regular |
| Reader keypad (PIN pad) | `dots-nine` | regular |
| Token unlocked for this session | `lock-key-open` | regular |
| PIN locked | `lock-key` | fill |
| PIN privacy / Privacy | `shield-check` | regular |
| Show / hide PIN | `eye` / `eye-slash` | regular |
| Expires soon | `clock` | regular |
| Success / green light | `check-circle` | fill |
| Attention / yellow light | `warning` | fill |
| Error / red light | `x-circle` | fill |
| Information / new site | `info` | fill |
| Not applicable / no data | `circle-dashed` | regular |
| Browsers (tab) | `browsers` | regular |
| Browser (row) | `browser` | regular |
| Allowed sites | `globe` | regular |
| Certificates (tab) | `certificate` | regular |
| Help (tab) | `question` | regular |
| Diagnostics ("Open diagnostics" buttons) | `stethoscope` | regular |
| Getting started | `list-checks` | regular |
| Download | `download-simple` | regular |
| External link (suffix) | `arrow-square-out` | regular |
| Copy | `copy` | regular |
| Scan again / try again | `arrow-clockwise` | regular |
| Add driver | `plus` | regular |
| Import .pfx | `file-plus` | regular |
| Filter | `magnifying-glass` | regular |
| Expand / collapse | `caret-right` / `caret-down` | regular |
| Close | `x` | regular |
| Keyboard shortcuts | `keyboard` | regular |

**Why a generic icon for browsers:** Phosphor has neither Edge nor Brave; mixing brand logos with a
generic icon looks uneven, and the browser name already identifies it.

---

## 13. Copy and i18n

### 13.1 Format

- **One TOML file per language** in `i18n/`: `en.toml`, `pt-BR.toml`, `pt-PT.toml`, `es.toml`, `fr.toml`,
  `it.toml`, `de.toml`.
  **Why TOML:** it accepts comments for translators (JSON does not), it is readable, and Rust reads it effortlessly.
- `en.toml` is the **reference**: it defines the set of keys. CI fails if any locale has a missing or extra key,
  or a key with different placeholders.
- Keys: sections per area (`[confirm]`, `[cert]`, …), names in `snake_case`; the full key is the dotted path
  (`confirm.window_title`).
- Placeholders `{name}`. Plurals as a subtable with CLDR categories `one`/`many`/`other` (only `other` is
  mandatory). The app implements the plural rules of the 7 languages in a pure function (no dependency).

```toml
# i18n/pt-BR.toml
[confirm]
# Window title in the OS. {site} is the host, e.g. app.diagnos.health
window_title = "Assinar para {site} — WebeSign"

[cert.expires_in]
one = "Vence em {count} dia"
other = "Vence em {count} dias"
```

- **Rust:** `build.rs` reads `i18n/*.toml` and generates constants (`k::CONFIRM_WINDOW_TITLE`) and the
  per-language tables; a wrong key is a compile error. Usage: `tr(k::CERT_EXPIRES_IN).count(23)`.
- **Extension:** a WXT build step converts the `[popup]` and `[store]` sections to
  `_locales/<pt_BR|pt_PT|es|fr|it|de|en>/messages.json` (name `popup.ready_title` → `popup_ready_title`;
  `{version}` → `$version$` with `placeholders`). The popup uses `chrome.i18n.getMessage`, and the store listing is
  translated for free. Rule: popup strings have no plurals (`chrome.i18n` does not support them).
- **SDK:** the `[site.errors]` section generates `sdk/src/messages.gen.ts`, exported from a separate entry point
  (`@websign/sdk/messages`), so the SDK keeps zero dependencies and the site only ships it if it wants to.
- **Language:** app by the OS language (`sys-locale`); popup by the browser language. Fallback chain:
  `pt-BR → pt-PT → en`, `pt-PT → pt-BR → en`, `es → en`, `fr → en`, `it → en`, `de → en`, others → `en`.
- Dates and numbers: formatted by a dedicated per-language function (`dd/mm/yyyy`, `14 Mar 2027`), no ICU.

### 13.2 Strings (pt-BR and en)

`{…}` are placeholders. Plurals are shown as `one / other`. These tables are product copy and are kept as data;
the translations for es, fr, it, de, and pt-PT follow the same keys.

#### Common

| Key | pt-BR | en |
|-------|-------|----|
| `common.cancel` | Cancelar | Cancel |
| `common.close` | Fechar | Close |
| `common.copy` | Copiar | Copy |
| `common.copied` | Copiado | Copied |
| `common.retry` | Tentar de novo | Try again |
| `common.wait` | Aguarde… | Please wait… |
| `common.details` | Detalhes | Details |
| `common.technical_details` | Detalhes técnicos | Technical details |
| `common.open_diagnostics` | Abrir diagnóstico | Open diagnostics |
| `common.refresh` | Procurar de novo | Scan again |
| `common.download_for` | Baixar para {os} | Download for {os} |
| `common.learn_more` | Saiba mais | Learn more |
| `common.new_window_hint` | (abre no navegador) | (opens in your browser) |
| `os.windows` / `os.macos` / `os.linux` | Windows / macOS / Linux | Windows / macOS / Linux |
| `time.just_now` | agora há pouco | just now |
| `time.minutes_ago` | há {count} min | {count} min ago |
| `time.today_at` | hoje, {time} | today, {time} |
| `time.yesterday_at` | ontem, {time} | yesterday, {time} |

#### Confirmation: header and origin

| Key | pt-BR | en |
|-------|-------|----|
| `confirm.window_title` | Assinar para {site} — WebeSign | Sign for {site} — WebeSign |
| `confirm.window_title_select` | Escolher certificado para {site} — WebeSign | Choose certificate for {site} — WebeSign |
| `confirm.eyebrow` | Pedido de assinatura | Signature request |
| `confirm.eyebrow_select` | Pedido de certificado | Certificate request |
| `confirm.via_browser` | pelo {browser} | via {browser} |
| `confirm.asks_sign` | quer que você assine um documento. | wants you to sign a document. |
| `confirm.asks_select` | quer saber com qual certificado você vai assinar. | wants to know which certificate you will sign with. |
| `confirm.site_remembered` | Site com permissão | Allowed site |
| `confirm.site_new` | Site novo | New site |
| `confirm.site_new_a11y` | Primeira vez que este site pede algo neste computador | First time this site asks for anything on this computer |
| `confirm.via_app` | de um programa neste computador | from a program on this computer |
| `confirm.app_remembered` | Programa com permissão | Allowed program |
| `confirm.app_new` | Programa novo | New program |
| `confirm.app_new_a11y` | Primeira vez que este programa pede algo neste computador | First time this program asks for anything on this computer |
| `caller.signed_by` | Assinado por {signer} | Signed by {signer} |
| `caller.unverified` | Programa não verificado. Só continue se você mesmo o abriu. | Unverified program. Only continue if you started it yourself. |
| `confirm.eyebrow_queue` | Pedido de assinatura {current} de {total} | Signature request {current} of {total} |
| `confirm.inside_frame` | Dentro da página de {top_site} | Inside a page from {top_site} |
| `origin.warn_idn` | Endereço com caracteres especiais. Confira letra por letra. | Address with special characters. Check it letter by letter. |
| `origin.shown_as` | Aparece como {unicode} | Displays as {unicode} |
| `origin.warn_ip` | Endereço numérico, sem nome de site | Numeric address with no site name |
| `origin.warn_ip_local` | Endereço numérico da rede local | Local network numeric address |
| `origin.warn_localhost` | Site local de desenvolvimento | Local development site |
| `origin.blocked_http` | Este site não usa conexão segura (https). Por segurança, o WebeSign não assina para ele. | This site doesn't use a secure connection (https). For your safety, WebeSign won't sign for it. |

#### Confirmation: code, list, and certificate

| Key | pt-BR | en |
|-------|-------|----|
| `code.label` | Código de conferência | Verification code |
| `code.help` | Confira se o site mostra o mesmo código. | Check that the site shows the same code. |
| `code.preparing` | Preparando o documento… | Preparing the document… |
| `code.a11y` | Código de conferência: {spelled} | Verification code: {spelled} |
| `code.continue_hint` | Escolha o certificado e clique em Continuar para ver o código de conferência. | Choose the certificate and click Continue to see the verification code. |
| `certs.label_sign` | Assinar com | Sign with |
| `certs.label_select` | Escolha o certificado | Choose a certificate |
| `certs.loading` | Procurando certificados… | Looking for certificates… |
| `certs.loading_slow` | Ainda lendo {device}. Drivers de token podem levar alguns segundos. | Still reading {device}. Token drivers can take a few seconds. |
| `certs.empty_title` | Nenhum certificado encontrado | No certificates found |
| `certs.empty_body` | Conecte o token ou insira o cartão no leitor. A lista atualiza sozinha. | Plug in your token or insert your card. The list updates by itself. |
| `certs.filter_placeholder` | Filtrar por nome, CPF ou emissor | Filter by name, ID number or issuer |
| `certs.filter_empty` | Nenhum certificado com “{query}” | No certificate matches “{query}” |
| `certs.unusable_group` | Não podem assinar ({count}) | Can't sign ({count}) |
| `certs.found_a11y` | Certificado encontrado: {name} | Certificate found: {name} |
| `cert.doc_cpf` | CPF {masked} | CPF {masked} |
| `cert.doc_cpf_a11y` | CPF parcialmente oculto, {visible} | CPF partially hidden, {visible} |
| `cert.doc_cnpj` | CNPJ {cnpj} | CNPJ {cnpj} |
| `cert.doc_generic` | Documento {masked} | ID {masked} |
| `cert.kind.icp` | ICP-Brasil {class} | ICP-Brasil {class} |
| `cert.kind.icp_plain` | ICP-Brasil | ICP-Brasil |
| `cert.kind.eidas_qscd` | Qualificado eIDAS | Qualified eIDAS |
| `cert.kind.eidas` | eIDAS | eIDAS |
| `cert.kind.pt_cc` | Cartão de Cidadão | Cartão de Cidadão |
| `cert.kind.es_dnie` | DNIe | DNIe |
| `cert.kind.generic` | Certificado | Certificate |
| `cert.where.computer` | Neste computador | On this computer |
| `cert.where.token_named` | Token {device} | Token {device} |
| `cert.where.card_in_reader` | Cartão no leitor | Card in reader |
| `cert.where.unknown_hw` | Token ou cartão | Token or card |
| `cert.via_driver` | pelo driver | via driver |
| `cert.valid_until` | Válido até {date} | Valid until {date} |
| `cert.expires_in` (one / other) | Vence em {count} dia / Vence em {count} dias | Expires in {count} day / Expires in {count} days |
| `cert.expires_tomorrow` | Vence amanhã | Expires tomorrow |
| `cert.expires_today` | Vence hoje | Expires today |
| `cert.expired_on` | Venceu em {date} | Expired on {date} |
| `cert.valid_from` | Válido a partir de {date} | Valid from {date} |
| `cert.reason.incompatible` | Não compatível com este pedido | Not compatible with this request |
| `cert.reason.pin_locked` | PIN bloqueado | PIN locked |
| `cert.reason.removed` | Removido. Conecte o token de novo. | Removed. Plug the token back in. |
| `cert.reason.no_key` | Sem chave privada neste computador | No private key on this computer |
| `cert.reason.login_only` | Feito para login, não para assinatura | Made for login, not for signing |
| `cert.details.subject` | Titular | Holder |
| `cert.details.issuer` | Emissor | Issuer |
| `cert.details.validity` | Validade | Validity |
| `cert.details.validity_range` | {from} a {to} | {from} to {to} |
| `cert.details.usage` | Uso | Usage |
| `cert.details.usage_sign` | Assinatura digital | Digital signature |
| `cert.details.usage_nonrep` | Não repúdio | Non-repudiation |
| `cert.details.key` | Chave | Key |
| `cert.details.fingerprint` | Impressão digital (SHA-256) | Fingerprint (SHA-256) |
| `cert.details.access` | Acesso | Access |
| `cert.details.access_windows` | Windows (repositório de certificados) | Windows (certificate store) |
| `cert.details.access_macos` | Keychain do macOS | macOS Keychain |
| `cert.details.access_driver` | Driver do token: {path} | Token driver: {path} |
| `cert.details.also_via_driver` | Também acessível pelo driver do token | Also available through the token driver |
| `cert.details.view_in_system` | Ver no sistema | View in system |

#### Confirmation: PIN, permission, buttons, and states

| Key | pt-BR | en |
|-------|-------|----|
| `pin.label_token` | PIN do token | Token PIN |
| `pin.label_card` | PIN do cartão | Card PIN |
| `pin.length_hint` | {min} a {max} caracteres | {min} to {max} characters |
| `pin.show` / `pin.hide` | Mostrar PIN / Ocultar PIN | Show PIN / Hide PIN |
| `pin.privacy` | O PIN fica neste computador e não passa pelo navegador. | Your PIN stays on this computer and never goes through the browser. |
| `pin.os_prompt` | O {os} vai pedir o PIN numa janela própria. | {os} will ask for your PIN in its own window. |
| `pin.os_prompt_now` | Digite o PIN na janela do {os}. | Enter your PIN in the {os} window. |
| `pin.pinpad_before` | Depois de clicar em Assinar, digite o PIN no teclado do leitor. | After you click Sign, enter your PIN on the reader's keypad. |
| `pin.pinpad_now` | Digite o PIN no teclado do leitor. | Enter your PIN on the reader's keypad. |
| `pin.unlocked_session` | Token desbloqueado nesta sessão | Token unlocked for this session |
| `pin.incorrect` | PIN incorreto. | Incorrect PIN. |
| `pin.incorrect_low` | PIN incorreto. Restam poucas tentativas antes de o token bloquear. | Incorrect PIN. Only a few attempts left before the token locks. |
| `pin.incorrect_final` | PIN incorreto. Última tentativa: se errar de novo, o token bloqueia. | Incorrect PIN. Last attempt: one more mistake locks the token. |
| `pin.locked_title` | PIN bloqueado | PIN locked |
| `pin.locked_body` | O token bloqueou depois de muitas tentativas erradas. Desbloqueie com o PUK no {tool} ou procure a {issuer}. | The token locked after too many wrong attempts. Unlock it with the PUK in {tool} or contact {issuer}. |
| `pin.locked_body_generic` | O token bloqueou depois de muitas tentativas erradas. Desbloqueie com o PUK no programa do fabricante ou procure a autoridade certificadora. | The token locked after too many wrong attempts. Unlock it with the PUK in the vendor's software or contact your certificate authority. |
| `consent.remember` | Lembrar este site neste computador | Remember this site on this computer |
| `consent.remember_app` | Lembrar este programa neste computador | Remember this program on this computer |
| `consent.remember_help` | Ele poderá saber qual certificado você usa sem perguntar. Cada assinatura continua pedindo sua confirmação. | It will be able to see which certificate you use without asking. Every signature still asks for your confirmation. |
| `consent.remember_disabled` | Endereços numéricos ou com caracteres especiais não podem ser lembrados. | Numeric or special-character addresses can't be remembered. |
| `consent.select_shares` | O site vai receber nome, tipo, emissor e validade do certificado escolhido. Nada é assinado agora. | The site will receive the name, type, issuer and validity of the chosen certificate. Nothing is signed now. |
| `action.sign` | Assinar | Sign |
| `action.signing` | Assinando… | Signing… |
| `action.use_cert` | Usar este certificado | Use this certificate |
| `action.continue` | Continuar | Continue |
| `footer.expires_in` | Este pedido expira em {seconds} s | This request expires in {seconds} s |
| `state.success_title` | Assinado | Signed |
| `state.success_body` | A assinatura foi enviada para {site}. | The signature was sent to {site}. |
| `state.select_success` | Certificado enviado para {site} | Certificate sent to {site} |
| `state.site_cancelled` | {site} cancelou o pedido. | {site} cancelled the request. |

#### Possible certificates and Add-on

| Key | pt-BR | en |
|-------|-------|----|
| `possible.title` | {device} conectado | {device} connected |
| `possible.body_driver_token` | Nenhum certificado apareceu neste token. Para usá-lo, instale o {driver}. | No certificate showed up on this token. To use it, install {driver}. |
| `possible.body_driver_card` | Nenhum certificado apareceu neste cartão. Para usá-lo, instale o {driver}. | No certificate showed up on this card. To use it, install {driver}. |
| `possible.body_unknown_card` | Há um cartão no leitor {reader}, mas não o reconhecemos. Se ele tem certificado, instale o programa do fabricante do cartão. | There's a card in {reader}, but we don't recognize it. If it holds a certificate, install the card vendor's software. |
| `possible.no_link` | Procure o programa no site da autoridade certificadora que emitiu o seu certificado. | Look for the software on the website of the authority that issued your certificate. |
| `possible.rescan` | Já instalei, procurar de novo | I installed it, scan again |
| `possible.inline` | {device} conectado sem certificados | {device} connected with no certificates |
| `possible.inline_action` | Como resolver | How to fix |
| `possible.step` | {n}. {text} | {n}. {text} |
| `possible.step_install_driver` | Instale o {driver} | Install {driver} |
| `possible.step_install_complement` | Instale o Complemento WebeSign | Install the WebeSign Add-on |
| `complement.name` | Complemento WebeSign | WebeSign Add-on |
| `complement.title_needed` | Este token precisa do Complemento para Mac | This token needs the Mac Add-on |
| `complement.body_needed` | Apps da App Store não podem usar alguns drivers de token. O Complemento WebeSign, gratuito e do mesmo projeto, faz essa ponte. Ele só funciona quando o WebeSign pede. | App Store apps can't use some token drivers. The free WebeSign Add-on, from the same project, bridges that gap. It only runs when WebeSign asks for it. |
| `complement.download` | Baixar Complemento | Download Add-on |
| `complement.ok` | Complemento ativo · versão {version} | Add-on active · version {version} |
| `complement.outdated` | Atualize o Complemento ({installed} instalado, precisa da {required}) | Update the Add-on ({installed} installed, {required} required) |
| `complement.update` | Atualizar Complemento | Update Add-on |
| `complement.not_responding` | O Complemento está instalado, mas não respondeu. | The Add-on is installed but didn't respond. |
| `complement.open` | Abrir Complemento | Open Add-on |

#### Diagnostics

| Key | pt-BR | en |
|-------|-------|----|
| `diag.window_title` | Diagnóstico — WebeSign | Diagnostics — WebeSign |
| `diag.status.ok` | Pronto para assinar | Ready to sign |
| `diag.status.attention` | Precisa de atenção | Needs attention |
| `diag.status.blocked` | Ainda não dá para assinar | Can't sign yet |
| `diag.status.na` | Não se aplica | Not applicable |
| `diag.tab.browsers` | Navegadores | Browsers |
| `diag.tab.devices` | Dispositivos | Devices |
| `diag.tab.certs` | Certificados | Certificates |
| `diag.tab.help` | Ajuda | Help |
| `diag.copy` | Copiar diagnóstico | Copy diagnostics |
| `diag.copied` | Diagnóstico copiado. Não inclui nomes, CPF nem sites. | Diagnostics copied. No names, ID numbers or sites included. |
| `onboarding.title` | Primeiros passos | Getting started |
| `onboarding.step_app` | App instalado | App installed |
| `onboarding.step_extension` | Extensão no navegador | Browser extension |
| `onboarding.step_cert` | Certificado encontrado | Certificate found |
| `onboarding.step_test` | Teste de assinatura | Test signature |
| `onboarding.test_button` | Testar assinatura | Test a signature |
| `onboarding.dismiss` | Ocultar | Hide |
| `browsers.subtitle` | Onde a extensão está instalada e conectada ao app. | Where the extension is installed and connected to the app. |
| `browsers.section` | Navegadores neste computador | Browsers on this computer |
| `browsers.connected` | Extensão {version} conectada · {when} | Extension {version} connected · {when} |
| `browsers.not_detected` | Extensão não detectada | Extension not detected |
| `browsers.install` | Instalar extensão | Install extension |
| `browsers.external_prompt_hint` | Se o {browser} mostrar “Nova extensão adicionada”, clique em Ativar. | If {browser} shows “New extension added”, click Enable. |
| `browsers.ext_outdated` | Extensão {version} · precisa da {required} ou mais nova | Extension {version} · needs {required} or newer |
| `browsers.ext_outdated_hint` | Reinicie o {browser} para atualizar. | Restart {browser} to update. |
| `browsers.host_missing` | O {browser} não encontra o app WebeSign | {browser} can't find the WebeSign app |
| `browsers.repair` | Reparar | Repair |
| `browsers.repaired` | Reparado. Reinicie o {browser}. | Repaired. Restart {browser}. |
| `browsers.safari_disabled` | Extensão desativada no Safari | Extension turned off in Safari |
| `browsers.safari_open_settings` | Abrir ajustes do Safari | Open Safari settings |
| `browsers.snap_hint` | Firefox instalado como Snap: na primeira assinatura, permita o acesso quando o sistema perguntar. | Firefox is installed as a Snap: on your first signature, allow access when the system asks. |
| `browsers.none` | Nenhum navegador compatível encontrado. O WebeSign funciona com Chrome, Edge, Firefox, Brave e Safari. | No supported browser found. WebeSign works with Chrome, Edge, Firefox, Brave and Safari. |
| `sites.section` | Sites com permissão | Allowed sites |
| `sites.apps_section` | Programas com permissão | Allowed programs |
| `sites.apps_empty` | Nenhum programa lembrado. | No remembered programs. |
| `sites.row` | Lembrado em {date} · último uso {when} | Remembered on {date} · last used {when} |
| `sites.revoke` | Revogar | Revoke |
| `sites.revoke_confirm` | Confirmar revogação | Confirm revoke |
| `sites.revoked` | Permissão de {site} revogada. | {site} is no longer allowed. |
| `sites.empty` | Nenhum site lembrado. Quando você marcar “Lembrar este site” numa assinatura, ele aparece aqui. | No remembered sites. When you tick “Remember this site” while signing, it shows up here. |
| `devices.subtitle` | Tokens, cartões e drivers que o WebeSign enxerga agora. | Tokens, cards and drivers WebeSign can see right now. |
| `devices.section_tokens` | Tokens e cartões | Tokens and cards |
| `devices.section_readers` | Leitores de cartão | Card readers |
| `devices.section_drivers` | Drivers de token | Token drivers |
| `devices.certs_found` (one / other) | {count} certificado / {count} certificados | {count} certificate / {count} certificates |
| `devices.no_certs` | Sem certificados · instale o {driver} | No certificates · install {driver} |
| `devices.unknown_certs` | Não sabemos se tem certificados | We can't tell if it has certificates |
| `devices.generic_ccid` | Dispositivo de cartão inteligente | Smart card device |
| `devices.reader_empty` | Sem cartão | No card |
| `devices.reader_card` | Cartão: {card} | Card: {card} |
| `devices.reader_card_unknown` | Cartão não reconhecido | Unrecognized card |
| `devices.driver_loaded` (one / other) | Carregado · {count} token / Carregado · {count} tokens | Loaded · {count} token / Loaded · {count} tokens |
| `devices.driver_no_token` | Carregado · nenhum token conectado | Loaded · no token connected |
| `devices.driver_failed` | Não carregou: {reason} | Failed to load: {reason} |
| `devices.driver_auto` | Encontrado automaticamente | Found automatically |
| `devices.driver_user` | Adicionado por você | Added by you |
| `devices.driver_add` | Adicionar driver… | Add driver… |
| `devices.driver_add_help` | Use só se o fabricante do token pedir. | Only use this if your token vendor tells you to. |
| `devices.driver_remove` | Remover | Remove |
| `devices.pcscd` | Serviço de cartões (pcscd) | Card service (pcscd) |
| `devices.pcscd_ok` | Ativo | Running |
| `devices.pcscd_stopped` | Parado. Rode: {command} | Stopped. Run: {command} |
| `devices.empty` | Nenhum token ou leitor conectado. Conecte o token na porta USB; esta lista atualiza sozinha. | No token or reader connected. Plug your token into a USB port; this list updates by itself. |
| `certs_tab.subtitle` | Todos os certificados que este computador oferece, inclusive os que não servem para assinar. | Every certificate this computer offers, including the ones that can't sign. |
| `certs_tab.group_windows` | No Windows | In Windows |
| `certs_tab.group_macos` | No Keychain do Mac | In the Mac Keychain |
| `certs_tab.group_driver` | Pelo driver do token | Through the token driver |
| `certs_tab.hidden_group` | Não servem para assinar ({count}) | Can't sign ({count}) |
| `certs_tab.import` | Importar arquivo .pfx… | Import .pfx file… |
| `certs_tab.import_help_windows` | O Windows importa o arquivo e guarda o certificado com segurança. Você vai precisar da senha do arquivo. | Windows imports the file and stores the certificate securely. You'll need the file's password. |
| `certs_tab.import_help_macos` | O Acesso às Chaves importa o arquivo e guarda o certificado com segurança. Você vai precisar da senha do arquivo. | Keychain Access imports the file and stores the certificate securely. You'll need the file's password. |
| `certs_tab.import_linux` | No Linux, use o certificado em token ou cartão. | On Linux, use a certificate on a token or card. |
| `certs_tab.empty` | Nenhum certificado neste computador. Conecte o token, insira o cartão ou importe um arquivo .pfx. | No certificates on this computer. Plug in your token, insert your card or import a .pfx file. |
| `help.faq_title` | Perguntas comuns | Common questions |
| `help.q_missing` | Meu certificado não aparece | My certificate doesn't show up |
| `help.a_missing` | Confira: o token está conectado? O driver do fabricante está instalado (veja Dispositivos)? O certificado venceu (veja Certificados)? Se é um arquivo .pfx, ele foi importado? | Check: is the token plugged in? Is the vendor's driver installed (see Devices)? Has the certificate expired (see Certificates)? If it's a .pfx file, was it imported? |
| `help.q_pin` | Errei o PIN / o token bloqueou | I got the PIN wrong / the token locked |
| `help.a_pin` | Depois de algumas tentativas erradas, o token bloqueia para proteger você. Para desbloquear, use o PUK (código de desbloqueio que veio com o token) no programa do fabricante. Sem o PUK, procure a autoridade certificadora. | After a few wrong attempts the token locks to protect you. To unlock it, use the PUK (the unlock code that came with the token) in the vendor's software. Without the PUK, contact your certificate authority. |
| `help.q_privacy` | O que o WebeSign envia para os sites? | What does WebeSign send to sites? |
| `help.a_privacy` | Só o certificado que você escolheu e a assinatura do código de conferência. Nunca o PIN, nunca o documento. | Only the certificate you chose and the signature of the verification code. Never your PIN, never the document. |
| `help.q_shortcuts` | Atalhos de teclado | Keyboard shortcuts |
| `help.report_title` | Relatar um problema | Report a problem |
| `help.report_preview` | Isto é exatamente o que será copiado. Não inclui nomes, CPF, sites nem números de série. | This is exactly what will be copied. It includes no names, ID numbers, sites or serial numbers. |
| `help.open_support` | Abrir página de suporte | Open support page |
| `help.about_title` | Sobre | About |
| `help.about_line` | WebeSign {version} · protocolo {protocol} · GPL-3.0-or-later | WebeSign {version} · protocol {protocol} · GPL-3.0-or-later |
| `help.source` | Código-fonte | Source code |
| `help.privacy` | Privacidade | Privacy |

#### Extension popup

| Key | pt-BR | en |
|-------|-------|----|
| `popup.checking` | Verificando… | Checking… |
| `popup.ready_title` | Tudo pronto para assinar | Ready to sign |
| `popup.ready_body` | O app WebeSign {version} está conectado a este navegador. | The WebeSign app {version} is connected to this browser. |
| `popup.open_diagnostics` | Abrir diagnóstico | Open diagnostics |
| `popup.missing_title` | Falta instalar o app WebeSign | The WebeSign app isn't installed |
| `popup.missing_body` | A extensão precisa do app no computador para assinar. | The extension needs the app on your computer to sign. |
| `popup.download` | Baixar para {os} | Download for {os} |
| `popup.activate` | Já instalei? Ativar o app | Already installed? Activate the app |
| `popup.outdated_title` | Atualize o app WebeSign | Update the WebeSign app |
| `popup.outdated_body` | Você tem a versão {installed}. Este navegador precisa da {required} ou mais nova. | You have version {installed}. This browser needs {required} or newer. |
| `popup.update_in` | Atualizar na {store} | Update in {store} |
| `popup.error_title` | O app não respondeu | The app didn't respond |
| `popup.error_body` | Tente de novo. Se continuar, reinicie o computador. | Try again. If it keeps happening, restart your computer. |
| `popup.retry` | Tentar de novo | Try again |
| `popup.redownload` | Baixar de novo | Download again |
| `popup.unsupported_title` | O WebeSign ainda não funciona neste sistema | WebeSign doesn't work on this system yet |
| `popup.unsupported_body` | Use um computador com Windows, macOS ou Linux. | Use a computer running Windows, macOS or Linux. |
| `popup.footer_versions` | Extensão {ext} · App {app} | Extension {ext} · App {app} |
| `popup.privacy` | Privacidade | Privacy |
| `store.microsoft` | Microsoft Store | Microsoft Store |
| `store.apple` | Mac App Store | Mac App Store |
| `store.linux` | página de download | download page |

The error texts are in [§15](#15-errors).

---

## 14. Accessibility

| Area | Rule |
|------|------|
| AccessKit | Always on (eframe's `accesskit` feature). Every widget has a role and a name; icon-only buttons have a name (`common.*`) and a tooltip. Root node language = the interface language, for the right voice. |
| List | `RadioGroup` role named "Sign with"; each row a `RadioButton` with a full name: "Ana Beatriz Souza, ICP-Brasil A3, CPF partially hidden 456 789, AC SOLUTI Multipla v5, Card in reader, expires in 23 days". A disabled row stays focusable, with `disabled` and the reason in the name. |
| Live regions | Polite: certificate found/removed, code ready, "Signed". Assertive: incorrect PIN, PIN locked, errors. |
| Origin | The origin and its warnings form a single text node read on open ("Signature request from app.diagnos.health, via Google Chrome, allowed site"; here with the browser's full name). The "New site" chip is read by its full accessible name (`confirm.site_new_a11y`). |
| Contrast | AA in both themes (table in §11.1); components and focus ≥ 3:1. |
| Focus | 2 px `focus` ring with a 2 px offset, on every focusable element, always visible when navigating by keyboard; never removed. |
| Keyboard | Everything operable without a mouse (§4.9, §8.8); no focus trap; Esc always exits. |
| Targets | Minimum 32 × 32 px (above WCAG 2.2 AA's 24 × 24); 72 px rows; icon buttons with 20 px visuals have a 32 px area. |
| Color | No state by color alone: the traffic light has an icon shape and text; validity has text; a field error has a message. |
| Larger text | Windows "Make text bigger" multiplies font sizes; the windows' body scrolls, so nothing is cut off. |
| Motion | §11.4. |
| Popup | Semantic HTML: `<main>`, status card with `role="status"` and `aria-live="polite"`, real `<button>`, initial focus on the primary button, `:focus-visible` with the `focus` token. |
| Verification | NVDA + Windows, VoiceOver + macOS, Orca + GNOME. `TODO(gustavo)`: confirm that egui's AccessKit exposes AT-SPI correctly to Orca before promising accessible Linux. |

---

## 15. Errors

They mirror the SDK's typed errors (`error.code`). If the SDK track uses another name, the SDK's wins and this
table is adjusted. Cancellation rule: when the window is closed, the SDK receives the code of the **last visible
blocker** (`NoCertificates`, `PinLocked`, `CertificateUnavailable`); with no blocker, `UserCancelled`.

Where it appears: **J** = Confirmation window · **P** = popup · **S** = site (suggested text in
`site.errors.*`, exported by the SDK) · **D** = developers only (English, in `error.message`).

The pt-BR and en columns are product copy (title — text) and are kept as data.

| Code | When | Where | pt-BR (title — text) | en (title — text) | Action |
|--------|--------|------|------------------------|---------------------|------|
| `ExtensionMissing` | The page did not receive the extension's announcement | S | Instale a extensão WebeSign — Para assinar neste navegador, instale a extensão gratuita. | Install the WebeSign extension — To sign in this browser, install the free extension. | Button with `installUrl()` |
| `AppMissing` | `connectNative` failed | S, P | Instale o app WebeSign — A extensão precisa do app no computador para assinar. | Install the WebeSign app — The extension needs the app on your computer to sign. | Download · "Already installed? Activate the app" |
| `AppOutdated` | App < `MIN_APP_VERSION` or old protocol | S, P | Atualize o app WebeSign — Você tem a versão {installed}; é preciso a {required} ou mais nova. | Update the WebeSign app — You have version {installed}; {required} or newer is required. | Update in the store |
| `ExtensionOutdated` | App requires a newer protocol than the extension's | S | Atualize a extensão WebeSign — Reinicie o navegador para ela se atualizar. | Update the WebeSign extension — Restart your browser so it updates. | n/a |
| `InsecureOrigin` | Non-local `http` origin, `file:`, etc. | S, D (J only through a bug) | Site sem conexão segura — Este site não usa https. Por segurança, o WebeSign não assina para ele. | Insecure site — This site doesn't use https. For your safety, WebeSign won't sign for it. | n/a (D: "Call the SDK from an https origin or localhost.") |
| `UserCancelled` | Cancel, Esc, or X with no visible blocker | S | Assinatura cancelada. | Signature cancelled. | The site decides |
| `Timeout` | 5 min without a decision | J, S | O pedido expirou — Ninguém respondeu em 5 minutos. Volte ao site e tente de novo. | The request expired — Nobody answered for 5 minutes. Go back to the site and try again. | n/a |
| `NoCertificates` | Window closed with an empty list | J (empty state), S | Nenhum certificado encontrado — Conecte o token ou insira o cartão e tente de novo. | No certificates found — Plug in your token or insert your card and try again. | Open diagnostics |
| `CertificateUnavailable` | The requested/chosen certificate vanished (token removed, key deleted) | J, S | Certificado indisponível — O certificado escolhido não está mais acessível. Conecte o token de novo ou escolha outro. | Certificate unavailable — The chosen certificate is no longer available. Plug the token back in or choose another one. | Focus on the list |
| `CertificateNotValid` | The site asked for an expired or not-yet-valid certificate | J (disabled row), S | Certificado fora da validade — Este certificado venceu ou ainda não começou a valer. Escolha outro ou renove com a sua autoridade certificadora. | Certificate out of validity — This certificate has expired or isn't valid yet. Choose another one or renew it with your certificate authority. | Open diagnostics |
| `InvalidRequest` | Digest with a size different from the declared hash, unknown hash, invalid parameters | D | n/a | "Digest is 20 bytes; SHA-256 requires 32." | The window does not open |
| `UnsupportedAlgorithm` | The requested algorithm does not exist for the key/driver (e.g. RSASSA-PSS on an old CSP) | J, S | Este certificado não assina desse jeito — O site pediu {algorithm}, que este certificado ou driver não oferece. Escolha outro certificado. | This certificate can't sign that way — The site asked for {algorithm}, which this certificate or driver doesn't support. Choose another certificate. | Focus on the list; the row becomes "Not compatible with this request" |
| `PinIncorrect` | Wrong PIN | J (does not reach the site) | see `pin.incorrect*` (§13.2) | n/a | Field cleared and focused |
| `PinLocked` | PIN locked | J, S | PIN bloqueado — see `pin.locked_body`. Site: "O PIN do seu token está bloqueado. Desbloqueie com o PUK e tente de novo." | PIN locked — see `pin.locked_body`. Site: "Your token PIN is locked. Unlock it with the PUK and try again." | Choose another certificate |
| `TokenRemoved` | Token left during signing | J, S | O token foi removido — Conecte o token de novo e clique em Tentar de novo. | The token was removed — Plug the token back in and click Try again. | Try again (enables when the token returns) |
| `DriverFailure` | CNG/CAPI/PKCS#11/Keychain error that is neither PIN nor cancellation | J, S | O driver do token falhou — O {driver} não respondeu como esperado. Tente de novo; se continuar, reconecte o token. | The token driver failed — {driver} didn't respond as expected. Try again; if it keeps happening, reconnect the token. | Try again · "Try through the token driver" (if there is an alternative path, §5.11) · Open diagnostics · Technical details: `CKR_DEVICE_ERROR (0x00000030)`, `NTE_BAD_KEYSET (0x80090016)` |
| `Busy` | Queue with 10 requests | S | Há pedidos esperando confirmação — Conclua os pedidos abertos na janela do WebeSign. | Requests are waiting for confirmation — Finish the open requests in the WebeSign window. | n/a |
| `Internal` | Any unexpected app failure | J, S | Algo deu errado no WebeSign — Copie os detalhes e abra o diagnóstico para relatar. | Something went wrong in WebeSign — Copy the details and open diagnostics to report it. | Copy details · Open diagnostics |

Keys: `errors.<snake_code>.title` and `errors.<snake_code>.body` for the window; `site.errors.<snake_code>.title`
and `.body` for the site; actions use the common keys.

Error visual in the window: `danger-soft` notice with a `danger-border` border, `radius-lg`, padding 12/16, 20 px
`x-circle` (fill) icon, `text-body-strong` title, `text-small` text, collapsed "Technical details" with the
code in `text-mono` and a `copy` button. It sits above the list, so the user can choose another certificate.

---

## 16. Reference test cases

Pure functions that the tests must cover before the code (TDD). "Today" in the validity cases = 2026-09-29,
local time zone. Output strings are shown in the `en` locale.

### 16.1 Verification code (`fingerprint`)

| Digest (first bytes, hex) | `text` | `colorIndex` | Identicon rows (1 = lit) |
|-------------------------------|--------|--------------|----------------------------------|
| `7F3A9C21E0B455D8…` | `7F3A 9C21 E0B4 55D8` | 3 | `00100 11011 01010 10101 11011` |
| SHA-256("") = `E3B0C44298FC1C14…` | `E3B0 C442 98FC 1C14` | 7 | `00100 00000 11011 00000 11011` |
| SHA-256("abc") = `BA7816BF8F01CFEA…` | `BA78 16BF 8F01 CFEA` | 5 | `01110 01010 00000 00100 11111` |
| 32 zero bytes | `0000 0000 0000 0000` | 0 | all off |

### 16.2 Origin (`format_origin`)

| Input | Highlight (registrable) | Remaining, dimmed | Warning | Can remember |
|---------|------------------------|------------------|--------|--------------|
| `https://app.diagnos.health` | `diagnos.health` | `https://app.` | none | yes |
| `https://app.diagnos.health:443` | `diagnos.health` | `https://app.` (default port hidden) | none | yes |
| `https://laudos.clinicasaolucas.med.br` | `clinicasaolucas.med.br` | `https://laudos.` | none | yes |
| `https://diagnos.health.cadastro-medico.com` | `cadastro-medico.com` | `https://diagnos.health.` | none | yes |
| `https://xn--dignos-4nf.health` | `xn--dignos-4nf.health` | `https://` | `idn` + "Displays as diаgnos.health" | no |
| `https://192.168.0.20:8443` | `192.168.0.20` | `https://`, `:8443` | `ip_local` | no |
| `https://203.0.113.7` | `203.0.113.7` | `https://` | `ip` | no |
| `http://localhost:5173` | `localhost` | `http://`, `:5173` | `localhost` | yes |
| `http://laudos.exemplo.com` | n/a | n/a | blocked (`InsecureOrigin`) | n/a |
| `ftp://files.exemplo.com` | n/a | n/a | blocked (`InsecureOrigin`) | n/a |
| `null`, `data:…`, `about:blank` | n/a | n/a | refused (malformed) | n/a |
| `https://exa<mple.com`, `https://host:`, `https://host.com.` | n/a | n/a | refused (malformed) | n/a |
| `https://127.1`, `https://0x7f.0.0.1` | n/a | n/a | refused (malformed; browsers never send these) | n/a |

### 16.3 Holder name (`display_name`)

| CN | Result |
|----|-----------|
| `ANA BEATRIZ SOUZA:12345678909` | `Ana Beatriz Souza` |
| `JOAO DA SILVA DOS SANTOS:98765432100` | `Joao da Silva dos Santos` |
| `CLINICA SOUZA IMAGEM LTDA:12345678000190` | `Clinica Souza Imagem Ltda` |
| `E SOUZA COMERCIO ME` | `E Souza Comercio ME` (a particle at the start stays capitalized) |
| `Marta Sofia Carvalho` | `Marta Sofia Carvalho` (not all caps: kept) |

### 16.4 Document (`display_document`)

| Input | Result |
|---------|-----------|
| otherName 2.16.76.1.3.1 = `01021985` + `12345678909` + … | `CPF •••.456.789-••` |
| otherName 2.16.76.1.3.3 = `12345678000190` | `CNPJ 12.345.678/0001-90` |
| serialNumber `IDCPT-12345123` | `ID •••••123` |
| nothing | (no document) |

### 16.5 Validity (`validity_label`)

| notBefore → notAfter | Text | Tone |
|----------------------|-------|-----|
| … → 2026-10-30 (31 days) | Valid until 30 Oct 2026 | muted |
| … → 2026-10-29 (30 days) | Expires in 30 days | warning |
| … → 2026-10-22 | Expires in 23 days | warning |
| … → 2026-10-06 (7 days) | Expires in 7 days | danger |
| … → 2026-09-30 | Expires tomorrow | danger |
| … → 2026-09-29 23:59 | Expires today | danger |
| … → 2026-05-10 | Expired on 10 May 2026 (disabled) | danger |
| 2026-10-01 → … | Valid from 1 Oct 2026 (disabled) | warning |

Days = difference between **local calendar dates**, not 24 h periods.

### 16.6 List (`build_cert_list`)

Input: Ana's A3 through Windows **and** through the driver (same DER); Ana's A1; the clinic's A1; an old expired A3;
the Cartão de Cidadão login certificate with its signing sibling; a certificate with no private key.
Expected output: 4 usable (A3 through Windows with `also_via_driver`, the Cartão de Cidadão signing certificate, Ana's A1, the clinic's A1), 1 disabled
(expired A3), hidden: the Cartão de Cidadão login and the one with no key. With "last used on this site" = Ana's A1,
it comes first and selected; inserting a new token while the window is open adds the row at the end without moving the
selection.

---

## 17. Open items

- `TODO(gustavo)`: approve the `sign({ prepare })` API and the semantics of `certificates()` (R1, R2) with the SDK track.
- `TODO(gustavo)`: Diagnos shows the verification code and the identicon next to "Waiting for confirmation" (R3).
- `TODO(gustavo)`: window focus on Windows when the host is already running (§4.1); include in proof 1.
- `TODO(gustavo)`: batch signing: in or out, and with what limit (§4.11).
- `TODO(gustavo)`: policy filter requested by the site (e.g. ICP-Brasil only) (R6).
- `TODO(gustavo)`: check ICP-Brasil policy OIDs against the current DOC-ICP-04 (§5.3).
- `TODO(gustavo)`: keep the token authenticated during the session (without `CKA_ALWAYS_AUTHENTICATE`) or ask for the PIN on every signature (§4.6).
- `TODO(gustavo)`: path for A1 (.pfx) on Linux (§8.5).
- `TODO(gustavo)`: final logo (today `seal-check` in an `accent` square).
- `TODO(gustavo)`: egui accessibility on Linux with Orca (§14).
- `TODO(gustavo)`: es, fr, it, de, and pt-PT copy (format ready, §13.1).
