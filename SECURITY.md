# Security policy

SignLocal sits between websites or programs and the keys that make legally binding signatures, so we treat
every report seriously. The threat model is [`docs/architecture/security.md`](docs/architecture/security.md).

## Reporting a vulnerability

Do **not** open a public issue, discussion or pull request.

1. Report it privately through GitHub: the repository's **Security** tab → **Report a vulnerability**
   (GitHub private vulnerability reporting). Only the maintainers see it.
2. If you cannot use GitHub, e-mail TODO(gustavo): security contact address.

Please include the affected version (`websign version`), the operating system and browser, the steps to
reproduce, and what an attacker gains. Never send a real PIN, a private key, or a certificate or document
that identifies a person: reproduce with a test certificate (for example SoftHSM2 or a throwaway `.pfx`).

## What to expect

- An acknowledgement within 5 working days.
- A first assessment, and a fix plan when the issue is confirmed, within 30 days.
- A fixed release and a GitHub security advisory crediting you (unless you prefer otherwise). Please keep
  the details private until the advisory is published.

## Scope

In scope: the `websign` app, the browser extension, `@websign/sdk`, the Node and Rust clients, the install
scripts, the release artifacts and the website. Examples: a site or program obtaining a signature or a
certificate without the person's confirmation, a PIN or personal data reaching logs or another process,
bypassing the origin or caller checks, or tampering with installs or updates.

Out of scope: weaknesses of a token vendor's driver or of the operating system's key store (report them to
the vendor; we will help), and the warnings shown by unsigned prerelease builds (see
[`docs/install.md`](docs/install.md#why-unsigned)).

## Supported versions

Only the latest release receives security fixes while the project is in prerelease.
