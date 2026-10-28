#!/bin/sh
# Renders the release page body: template.md with {{version}}, {{tag}},
# {{repo}} filled in and {{artifacts}} replaced by a table built from
# SHA256SUMS, so the page lists exactly the files that were hashed.
#
#   sh packaging/release-notes/render.sh <version> <SHA256SUMS> [owner/repo] > notes.md
set -eu

[ $# -ge 2 ] || { echo "usage: render.sh <version> <SHA256SUMS> [owner/repo]" >&2; exit 2; }
version=${1#v}
sums=$2
repo=${3:-diagnos-tech/signlocal}
template=$(dirname "$0")/template.md

[ -s "$sums" ] || { echo "render.sh: $sums is missing or empty" >&2; exit 1; }

table=$(awk -v v="$version" '
    function describe(name) {
        if (name == "install.sh") return "installer, macOS and Linux"
        if (name == "install.ps1") return "installer, Windows"
        if (name ~ /windows-x64\.zip$/) return "Windows x64"
        if (name ~ /windows-arm64\.zip$/) return "Windows arm64"
        if (name ~ /macos-universal\.zip$/) return "macOS (Apple silicon and Intel)"
        if (name ~ /_amd64\.deb$/) return "Debian, Ubuntu (amd64)"
        if (name ~ /_arm64\.deb$/) return "Debian, Ubuntu (arm64)"
        if (name ~ /x86_64\.rpm$/) return "Fedora, RHEL family (x86_64)"
        if (name ~ /aarch64\.rpm$/) return "Fedora, RHEL family (aarch64)"
        if (name ~ /linux-x64\.tar\.gz$/) return "Linux x64 tarball (install.sh)"
        if (name ~ /linux-arm64\.tar\.gz$/) return "Linux arm64 tarball (install.sh)"
        if (name ~ /extension-.*-chromium\.zip$/) return "extension: Chrome, Edge, Brave"
        if (name ~ /extension-.*-firefox\.zip$/) return "extension: Firefox"
        return ""
    }
    BEGIN { print "| File | For | SHA-256 |"; print "|---|---|---|" }
    NF == 2 {
        name = $2; sub(/^\*/, "", name)
        printf "| `%s` | %s | `%s` |\n", name, describe(name), $1
        count++
    }
    END { if (count == 0) exit 1 }
' "$sums") || { echo "render.sh: no entries in $sums" >&2; exit 1; }

# The table goes through a file: awk -v would interpret its backslashes.
table_file=$(mktemp)
trap 'rm -f "$table_file"' EXIT
printf '%s\n' "$table" > "$table_file"

awk -v version="$version" -v tag="v$version" -v repo="$repo" -v table_file="$table_file" '
    $0 == "{{artifacts}}" {
        while ((getline line < table_file) > 0) print line
        next
    }
    {
        gsub(/\{\{version\}\}/, version)
        gsub(/\{\{tag\}\}/, tag)
        gsub(/\{\{repo\}\}/, repo)
        if ($0 ~ /\{\{[a-z_]+\}\}/) { print "render.sh: unknown placeholder: " $0 > "/dev/stderr"; failed = 1 }
        print
    }
    END { exit failed }
' "$template"
