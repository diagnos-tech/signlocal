# @websign/sdk

Let people sign on your website with the certificate they already have: a smart card, a USB token or a
certificate installed on their computer, through the WebeSign extension and app. Zero dependencies, ESM,
fully typed, under 5 KB gzipped, framework-agnostic.

```sh
bun add @websign/sdk     # or: npm install @websign/sdk
```

```ts
import { sign } from "@websign/sdk";

const { signature, certificate } = await sign({
  hash: "SHA-256",
  prepare: (certificate, { hash }) => crypto.subtle.digest(hash, signedAttributesFor(certificate)),
});
```

That is the whole happy path. `prepare` runs once the person has chosen a certificate in the WebeSign window
(PAdES and CAdES put the certificate inside the signed data) and returns the hash to sign: 32, 48 or 64 bytes
for SHA-256/384/512, as a `Uint8Array` or `ArrayBuffer`. It runs again if the person switches certificate;
only the last hash is signed. `signature` is raw (ECDSA r‖s, or the RSA block); put it in your CMS with
`certificate.der` and `certificate.chain`. The types follow your options: with `hash: "SHA-384"` and
`algorithm: "ECDSA"`, `prepare`'s context and the result are typed with exactly those values.

Guide with the errors table: [developers page](https://diagnos-tech.github.io/signlocal/developers.html).
API reference: [`site/api/`](https://diagnos-tech.github.io/signlocal/api/). Examples (vanilla, React, Vue,
PAdES with pdf-lib + PKI.js): [`examples/web/`](../examples/web/).

## Is WebeSign ready?

```ts
import { installUrl, onChange, status } from "@websign/sdk";
import { errorText } from "@websign/sdk/messages";

function show({ ready, problem }: Status) {
  signButton.hidden = !ready;
  installLink.hidden = problem !== "ExtensionMissing";
  installLink.href = installUrl(); // the extension store of this browser
  notice.textContent = errorText(problem)?.body ?? ""; // localized, empty when ready
}
show(await status()); // never rejects, never opens a window
onChange(show); // the person installs or updates WebeSign while the page is open
```

## Errors

Every rejection is a `WebSignError`: a stable `code`, a `message` (what happened) and a `hint` (what to do)
for you, a `docsUrl` to its row on the developers page, and `details` for version and driver errors. For the
person, `errorText(error)` from `@websign/sdk/messages` gives a title and the next step in 7 languages.

```ts
import { isWebSignError, sign } from "@websign/sdk";
import { errorText } from "@websign/sdk/messages";

try {
  await sign({ hash: "SHA-256", prepare });
} catch (error) {
  if (!isWebSignError(error)) throw error;
  if (error.code === "UserCancelled") return; // nothing to show
  console.warn(error.code, error.hint, error.docsUrl);
  const text = errorText(error); // fills {installed}/{required} from details
  if (text) showBanner(text.title, text.body);
}
```

Cancel with an `AbortSignal` (`sign({ ..., signal })`): the window closes and the promise rejects with
`Aborted`. If `prepare` throws, the request is cancelled and rejects with `Aborted`, your error as `cause`.

## Testing without the extension

`@websign/sdk/testing` installs a fake extension and app on `window` (browser, jsdom or happy-dom). Your code
runs unchanged, and signatures are real ECDSA P-256 / RSA-2048 made with public test keys, deterministic, so
you can verify and snapshot them.

```ts
import { installFakeWebSign, type FakeWebSign } from "@websign/sdk/testing";

let fake: FakeWebSign;
beforeEach(async () => (fake = await installFakeWebSign()));
afterEach(() => fake.uninstall());

it("signs", async () => {
  const result = await sign({ hash: "SHA-256", prepare: () => crypto.subtle.digest("SHA-256", data) });
  expect(await fake.verify(result, data)).toBe(true);
});

it("handles a cancel", async () => {
  fake.failNext("UserCancelled");
  await expect(signContract()).rejects.toMatchObject({ code: "UserCancelled" });
});
```

Setups: `scenario: "ready" | "extension-missing" | "app-missing" | "app-outdated" | "extension-outdated" |
"sdk-outdated"`, `certificates: [{ key: "RSA", notAfter, icpBrasil, eidas, … }]` (`[]` for none),
`remembered`, `latencyMs`. Scripting: `failNext(code, details?)`, `choose(i)`, `switchDuringNextSign(i)`,
`setScenario()`. Inspection: `certificates`, `requests`, `verify()`.

Never ship it: anyone can sign with its keys. It is a separate entry point, refuses non-local origins unless
`allowAnyOrigin: true`, and warns on the console at every install. For local development, load it behind a
dynamic import (`if (import.meta.env.DEV) await (await import("@websign/sdk/testing")).installFakeWebSign()`).

## Other calls

- `certificates({ algorithm?, signal? })`: the certificate the person chooses (never the computer's list).
  Pass it to `sign({ certificate })` to skip the choice.
- `fingerprint(digest)`: the verification code the app shows (`result.digest` is what was signed), to display
  next to your button.

## Notes

- Only messages from the page's own window and origin, from the extension's content script and with the
  expected shape are accepted; the SDK posts only to its own origin, never `"*"`, and never logs.
- The SDK speaks page protocol 1 (`ClientOutdated`: update `@websign/sdk`; `ExtensionOutdated`: the person
  updates the extension).
- Safe to import during server-side rendering: nothing touches `window` until a call.
- Batch signing is not offered: each signature is one confirmation.

## Development

- API guide: [`docs/architecture/web-api.md`](../docs/architecture/web-api.md). Contract: [`SPEC.md`](SPEC.md).
  Wire types in `src/generated`, the error texts in `src/messages.gen.ts` and `src/project.ts` come from
  `cargo xtask gen`.
- `bun run typecheck`, `bun run test`, `bun run build`, then `bun run size` (fails when the main entry
  reaches 5 KB gzip; reports the other entries).
- `bun run docs` regenerates the API reference into `site/api/` (TypeDoc, in `tools/`, which has its own
  lockfile because it needs the TypeScript 6 compiler API); `bun run docs:check` fails when it is stale.
- `bun run check:package` runs publint and are-the-types-wrong on the packed package.
- License: **Apache-2.0** ([`LICENSE`](LICENSE)).
