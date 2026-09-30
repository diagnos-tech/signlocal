#!/usr/bin/env sh
# Rebuilds site/assets/websign-sdk.js: the @websign/sdk ESM bundle (plus its
# localized error texts) that the test page imports. The built file is
# committed so the site works when served straight from a checkout; run this
# after any change under sdk/src and commit the result. The Pages workflow
# (.github/workflows/pages.yml) runs it again before every deploy.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
sdk="$here/../sdk"
out="$here/assets/websign-sdk.js"

(cd "$sdk" && bun run build)

entry="$sdk/dist/site-entry.js"
printf '%s\n' 'export * from "./index.js";' 'export * from "./messages.js";' > "$entry"

tmp=$(mktemp)
trap 'rm -f "$tmp" "$entry"' EXIT
bun build "$entry" --minify --target browser --format esm --outfile "$tmp"

{
  printf '%s\n' '/* @websign/sdk (Apache-2.0) bundled for this site with site/build-sdk.sh. Do not edit: rebuild. */'
  cat "$tmp"
} > "$out"
echo "wrote $out ($(wc -c < "$out") bytes)"
