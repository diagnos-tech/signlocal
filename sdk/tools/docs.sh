#!/usr/bin/env sh
# Generates the API reference into site/api/ (committed), or with --check
# regenerates it into a temporary folder and fails when site/api/ differs,
# so CI can catch a source change whose docs were not rebuilt.
set -eu
tools=$(cd "$(dirname "$0")" && pwd)
cd "$tools"
bun install --frozen-lockfile --silent
if [ "${1:-}" = "--check" ]; then
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  ./node_modules/.bin/typedoc --logLevel Warn --out "$tmp"
  if ! diff -r "$tmp" "$tools/../../site/api" > /dev/null; then
    echo "site/api/ is out of date: run 'bun run docs' in sdk/ and commit the result." >&2
    exit 1
  fi
  echo "site/api/ is up to date."
else
  ./node_modules/.bin/typedoc --logLevel Warn
  echo "wrote site/api/"
fi
