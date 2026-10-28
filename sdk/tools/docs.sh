#!/usr/bin/env sh
# Generates the API reference with TypeDoc. Nothing generated is committed: the site's build
# (`bun run build` in site/) runs this into dist/api, and so does CI.
#
#   sh docs.sh                 writes site/dist/api
#   sh docs.sh --out <dir>     writes <dir> (relative to the current folder)
#
# Source links point at the commit being built (GITHUB_SHA in CI), else at `develop`.
set -eu
tools=$(cd "$(dirname "$0")" && pwd)
out="$tools/../../site/dist/api"
if [ "${1:-}" = "--out" ]; then
  [ -n "${2:-}" ] || { echo "usage: docs.sh [--out <dir>]" >&2; exit 2; }
  case $2 in
    /*) out=$2 ;;
    *) out="$(pwd)/$2" ;;
  esac
fi
cd "$tools"
bun install --frozen-lockfile --silent
./node_modules/.bin/typedoc --logLevel Warn --out "$out" \
  --sourceLinkTemplate "https://github.com/diagnos-tech/signlocal/blob/${GITHUB_SHA:-develop}/{path}#L{line}"
echo "wrote $out"
