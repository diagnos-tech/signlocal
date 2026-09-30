# WebeSign website

The public site at the `homepage` in [`project.toml`](../project.toml): what WebeSign is, downloads, privacy,
developer quickstarts, the `/activate` page the app opens after install, and the `/test` page that signs a
sample text in the browser. The SDK, the extension popup and the app link to these pages, so they must stay
at the same paths.

## Layout

Plain HTML with English text and no framework. `assets/i18n.js` swaps in the visitor's language from
`locales/*.json`. The only generated file is `assets/websign-sdk.js`, the SDK bundle the test page imports;
`build-sdk.sh` rebuilds it (needs `bun install` at the repository root). See [`SUMMARY.md`](SUMMARY.md).

## Preview and test

```sh
sh site/build-sdk.sh                  # after any change under sdk/src
python3 -m http.server -d site 8000   # then open http://localhost:8000/
bun test site/tests                   # key extractor and verifier
```

## Deploy

[`.github/workflows/pages.yml`](../.github/workflows/pages.yml) rebuilds the SDK bundle and publishes this
folder (without `tests/`, `build-sdk.sh` and the folder notes) to GitHub Pages on every push to `main` that
touches the site, the SDK, `i18n/` or `project.toml`, and on demand.

- TODO(gustavo): enable Pages in the repository settings with "Source: GitHub Actions"; the deploy job fails
  until then.
- TODO(gustavo): own subdomain (a `CNAME` file here plus DNS), see `project.toml`.
