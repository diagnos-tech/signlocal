#!/bin/sh
# WebeSign installer for Linux and macOS (POSIX sh).
#
#   sh install.sh --version <v> [--yes] [--prefix DIR] [--system]
#                 [--no-register] [--dry-run]
#   sh install.sh --uninstall [--yes] [--prefix DIR] [--dry-run]
#
# It downloads the artifact and SHA256SUMS of release v<v>, verifies the hash
# (any doubt aborts before the system is touched), installs, and runs
# `websign install`. Contract: docs/architecture/packaging-and-release.md and
# scripts/install/README.md. Tests source this file with WEBSIGN_INSTALL_LIB=1.
set -eu

# Identifiers come from project.toml; `cargo xtask check release` keeps them equal.
SLUG=websign
APP_NAME=WebeSign
BUNDLE_ID=dev.websign.app
REPO=${WEBSIGN_REPO:-diagnos-tech/web-esign}

VERSION=${WEBSIGN_VERSION:-}
YES=0
PREFIX=
SYSTEM=0
REGISTER=1
DRY_RUN=0
UNINSTALL=0
WORK=

say() { printf '%s\n' "$*" >&2; }
die() { say "install.sh: error: $*"; exit 1; }

usage() {
    say "usage: sh install.sh --version <v> [--yes] [--prefix DIR] [--system] [--no-register] [--dry-run]"
    say "       sh install.sh --uninstall [--yes] [--prefix DIR] [--dry-run]"
}

# Prints a command instead of running it under --dry-run.
run() {
    if [ "$DRY_RUN" = 1 ]; then
        say "+ $*"
    else
        "$@"
    fi
}

cleanup() { [ -n "$WORK" ] && rm -rf "$WORK"; return 0; }

# The EXIT trap does the cleanup; a signal must also stop the script, which a
# plain INT/TERM handler would not do in sh.
on_signal() { exit 130; }

# ---- detection ----------------------------------------------------------

detect_os() {
    case "${WEBSIGN_UNAME_S:-$(uname -s)}" in
        Linux) echo linux ;;
        Darwin) echo macos ;;
        *) die "unsupported operating system (Windows uses install.ps1)" ;;
    esac
}

# Prints x64 or arm64.
detect_arch() {
    case "${WEBSIGN_UNAME_M:-$(uname -m)}" in
        x86_64 | amd64) echo x64 ;;
        aarch64 | arm64) echo arm64 ;;
        *) die "unsupported CPU architecture: ${WEBSIGN_UNAME_M:-$(uname -m)}" ;;
    esac
}

# Reads ID and ID_LIKE from os-release WITHOUT sourcing it (it is a shell
# file; running it would execute whatever it contains). Prints deb, rpm or
# other: the package formats we ship.
detect_distro() {
    file=${WEBSIGN_OS_RELEASE:-/etc/os-release}
    [ -r "$file" ] || { echo other; return 0; }
    ids=$(sed -n -e 's/^ID=//p' -e 's/^ID_LIKE=//p' "$file" | tr -d '"'"'" | tr '\n' ' ')
    for id in $ids; do
        case "$id" in
            debian | ubuntu) echo deb; return 0 ;;
            fedora | rhel | centos | rocky | almalinux) echo rpm; return 0 ;;
        esac
    done
    echo other
}

# ---- download and verification -----------------------------------------

have() { command -v "$1" >/dev/null 2>&1; }

# curl over HTTPS only (redirects included), TLS 1.2 or later.
get() { curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 "$@"; }

# get() with the bearer token passed through a config on stdin, so it never
# shows in the process list the way a -H argument would.
get_auth() {
    printf 'header = "Authorization: Bearer %s"\n' "$WEBSIGN_GITHUB_TOKEN" | get -K - "$@"
}

# The asset id of "$2" in the JSON release description "$1" (GitHub API).
asset_id() {
    tr ',' '\n' <"$1" | awk -v want="$2" '
        /"url": *"[^"]*\/releases\/assets\/[0-9]+"/ { id = $0; sub(/.*assets\//, "", id); sub(/".*/, "", id); next }
        /"url":/ { id = "" }
        /"name":/ { n = $0; sub(/.*"name": *"/, "", n); sub(/".*/, "", n); if (n == want && id != "") { print id; exit } }'
}

# Downloads asset "$1" of release v$VERSION into "$WORK". Source order: a
# local release folder (tests, offline), gh when logged in, a bearer token
# for the private repository, anonymous HTTPS.
fetch() {
    name=$1
    if [ -n "${WEBSIGN_RELEASE_DIR:-}" ]; then
        cp "$WEBSIGN_RELEASE_DIR/$name" "$WORK/$name" || die "$name is not in $WEBSIGN_RELEASE_DIR"
    elif have gh && gh auth status >/dev/null 2>&1; then
        gh release download "v$VERSION" --repo "$REPO" --pattern "$name" --dir "$WORK" --clobber \
            || die "gh could not download $name of v$VERSION"
    elif [ -n "${WEBSIGN_GITHUB_TOKEN:-}" ]; then
        have curl || die "curl is required"
        api="https://api.github.com/repos/$REPO/releases"
        get_auth -H "Accept: application/vnd.github+json" "$api/tags/v$VERSION" \
            -o "$WORK/release.json" || die "cannot read release v$VERSION (check the token)"
        id=$(asset_id "$WORK/release.json" "$name")
        [ -n "$id" ] || die "release v$VERSION has no file named $name"
        get_auth -H "Accept: application/octet-stream" "$api/assets/$id" \
            -o "$WORK/$name" || die "cannot download $name"
    else
        have curl || die "curl is required"
        get "https://github.com/$REPO/releases/download/v$VERSION/$name" -o "$WORK/$name" \
            || die "cannot download $name (private repository? run 'gh auth login' or set WEBSIGN_GITHUB_TOKEN)"
    fi
}

sha256_of() {
    if have sha256sum; then sha256sum "$1" | awk '{print $1}'
    elif have shasum; then shasum -a 256 "$1" | awk '{print $1}'
    elif have openssl; then openssl dgst -sha256 "$1" | awk '{print $NF}'
    else die "no SHA-256 tool found (sha256sum, shasum or openssl)"
    fi
}

# Fails unless "$1" (in $WORK) matches its line in $WORK/SHA256SUMS.
verify() {
    name=$1
    want=$(awk -v n="$name" '$2 == n || $2 == "*" n { print tolower($1); exit }' "$WORK/SHA256SUMS")
    [ -n "$want" ] || die "$name is not listed in SHA256SUMS"
    got=$(sha256_of "$WORK/$name")
    [ "$got" = "$want" ] || die "checksum mismatch for $name: refusing to install (delete the download)"
    say "verified $name"
}

fetch_verified() {
    fetch SHA256SUMS
    fetch "$1"
    verify "$1"
}

# ---- interaction --------------------------------------------------------

# Asks a yes/no question whose default is no. Unattended runs (--yes, or no
# terminal) take the default: questions guard choices --yes must never imply.
ask() {
    [ "$YES" = 1 ] && return 1
    [ -t 0 ] || return 1
    printf '%s [y/N] ' "$1" >&2
    read -r answer || return 1
    case "$answer" in y | Y | yes | YES) return 0 ;; *) return 1 ;; esac
}

register() {
    exe=$1
    if [ "$REGISTER" = 0 ]; then
        say "skipping browser registration (--no-register); run '$exe install' later"
    else
        run "$exe" install || say "registration reported a problem; run '$exe doctor'"
    fi
}

next_steps() {
    say ""
    say "$APP_NAME is installed. Last step: install the browser extension."
    say "Steps per browser: https://github.com/$REPO/blob/main/docs/install.md#browser-extension"
    say "(download websign-extension-$VERSION-chromium.zip or -firefox.zip from the same release)."
}

# ---- Linux --------------------------------------------------------------

# Prints the command prefix that gives root rights: nothing for root, sudo
# otherwise; fails when neither is available.
as_root() {
    if [ "$(id -u)" = 0 ]; then echo; elif have sudo; then echo sudo; else return 1; fi
}

# Whether this distribution can take our deb or rpm through its package manager.
package_usable() {
    family=$1
    [ "$family" = other ] && return 1
    as_root >/dev/null || return 1
    if [ "$family" = deb ]; then have apt-get; else have dnf || have yum; fi
}

# The default is the per-user tarball (no root). The system package is used
# with --system, or when the user says yes to the question; --yes never
# implies it, so an unattended run cannot escalate to root.
want_package() {
    family=$1
    if [ "$SYSTEM" = 1 ]; then
        package_usable "$family" \
            || die "--system needs a Debian/Ubuntu or Fedora/RHEL family system with apt-get or dnf, and root or sudo"
        return 0
    fi
    package_usable "$family" || return 1
    ask "Install the system package instead (uses sudo, registers every user)? No installs for your user only."
}

install_package() {
    family=$1; arch=$2
    if [ "$family" = deb ]; then
        if [ "$arch" = x64 ]; then a=amd64; else a=arm64; fi
        file="${SLUG}_${VERSION}_$a.deb"
    else
        if [ "$arch" = x64 ]; then a=x86_64; else a=aarch64; fi
        file="$SLUG-$VERSION-1.$a.rpm"
    fi
    fetch_verified "$file"
    root_cmd=$(as_root)
    if [ "$family" = deb ]; then
        manager=apt-get
    elif have dnf; then
        manager=dnf
    else
        manager=yum
    fi
    # $root_cmd is empty or "sudo": unquoted on purpose so empty adds no word.
    # shellcheck disable=SC2086
    run $root_cmd "$manager" install -y "$WORK/$file"
    register "$SLUG"
}

install_tarball() {
    arch=$1
    root=${PREFIX:-$HOME/.local}
    file="$SLUG-$VERSION-linux-$arch.tar.gz"
    fetch_verified "$file"
    run tar -xzf "$WORK/$file" -C "$WORK"
    src="$WORK/$SLUG-$VERSION-linux-$arch"
    run mkdir -p "$root/bin" "$root/share"
    run cp "$src/bin/$SLUG" "$root/bin/$SLUG"
    run chmod 0755 "$root/bin/$SLUG"
    run cp -R "$src/share/." "$root/share/"
    desktop="$root/share/applications/$SLUG.desktop"
    # The entry must start the copy just installed, which may not be on PATH.
    # Quoted (Desktop Entry spec) so a prefix with spaces works; awk, not sed,
    # so no character of the path is taken as syntax.
    if [ "$DRY_RUN" = 0 ]; then
        EXE="$root/bin/$SLUG" awk '/^Exec=/ { print "Exec=\"" ENVIRON["EXE"] "\""; next } { print }' \
            "$desktop" >"$desktop.new"
        mv "$desktop.new" "$desktop"
    fi
    register "$root/bin/$SLUG"
    case ":$PATH:" in *":$root/bin:"*) ;; *) say "add $root/bin to your PATH to run '$SLUG' from a terminal" ;; esac
    if [ "$REGISTER" = 1 ] && [ "$(id -u)" != 0 ] && have sudo; then
        say "To register the app for other users too: sudo $root/bin/$SLUG install --system"
    fi
}

install_linux() {
    arch=$(detect_arch)
    family=$(detect_distro)
    if want_package "$family" "$arch"; then
        install_package "$family" "$arch"
    else
        install_tarball "$arch"
    fi
}

uninstall_linux() {
    root=${PREFIX:-$HOME/.local}
    exe=
    if [ -x "$root/bin/$SLUG" ]; then exe="$root/bin/$SLUG"; elif have "$SLUG"; then exe=$SLUG; fi
    [ -z "$exe" ] || run "$exe" uninstall || true
    run rm -f "$root/bin/$SLUG" "$root/share/applications/$SLUG.desktop" \
        "$root/share/metainfo/$BUNDLE_ID.metainfo.xml" \
        "$root/share/icons/hicolor/"*"/apps/$SLUG".*
    run rm -rf "$root/share/doc/$SLUG"
    if have dpkg && dpkg -s "$SLUG" >/dev/null 2>&1; then
        say "The system package is installed: sudo apt remove $SLUG"
    elif have rpm && rpm -q "$SLUG" >/dev/null 2>&1; then
        say "The system package is installed: sudo dnf remove $SLUG"
    fi
}

# ---- macOS --------------------------------------------------------------

macos_dest() {
    if [ -n "$PREFIX" ]; then echo "$PREFIX"
    elif [ -w /Applications ]; then echo /Applications
    else echo "$HOME/Applications"
    fi
}

install_macos() {
    file="$SLUG-$VERSION-macos-universal.zip"
    dest=$(macos_dest)
    fetch_verified "$file"
    if have ditto; then run ditto -x -k "$WORK/$file" "$WORK/app"; else run unzip -q "$WORK/$file" -d "$WORK/app"; fi
    run mkdir -p "$dest" "$HOME/.local/bin"
    run rm -rf "${dest:?}/$APP_NAME.app"
    run mv "$WORK/app/$APP_NAME.app" "$dest/$APP_NAME.app"
    # Only our own bundle: never touch other quarantined files.
    run xattr -dr com.apple.quarantine "$dest/$APP_NAME.app" 2>/dev/null || true
    run ln -sf "$dest/$APP_NAME.app/Contents/MacOS/$SLUG" "$HOME/.local/bin/$SLUG"
    register "$dest/$APP_NAME.app/Contents/MacOS/$SLUG"
    case ":$PATH:" in *":$HOME/.local/bin:"*) ;; *) say "add $HOME/.local/bin to your PATH to run '$SLUG' from a terminal" ;; esac
}

uninstall_macos() {
    link="$HOME/.local/bin/$SLUG"
    if [ -n "$PREFIX" ]; then
        set -- "$PREFIX"
    else
        set -- /Applications "$HOME/Applications"
    fi
    for dest in "$@"; do
        app="$dest/$APP_NAME.app"
        [ -d "$app" ] || continue
        run "$app/Contents/MacOS/$SLUG" uninstall || true
        run rm -rf "${app:?}" || say "cannot remove $app; move it to the Trash"
        if [ -L "$link" ]; then
            case "$(readlink "$link")" in "$app"/*) run rm -f "$link" ;; esac
        fi
    done
}

# ---- entry point --------------------------------------------------------

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --version) [ $# -ge 2 ] || die "--version needs a value"; VERSION=$2; shift ;;
            --prefix) [ $# -ge 2 ] || die "--prefix needs a value"; PREFIX=$2; shift ;;
            --yes | -y) YES=1 ;;
            --system) SYSTEM=1 ;;
            --no-register) REGISTER=0 ;;
            --dry-run) DRY_RUN=1 ;;
            --uninstall) UNINSTALL=1 ;;
            -h | --help) usage; exit 0 ;;
            *) usage; die "unknown option: $1" ;;
        esac
        shift
    done
    VERSION=${VERSION#v}
}

main() {
    parse_args "$@"
    os=$(detect_os)
    if [ "$UNINSTALL" = 1 ]; then
        "uninstall_$os"
        say "$APP_NAME removed. Remove the extension from your browser's extensions page."
        return 0
    fi
    [ -n "$VERSION" ] || die "--version is required: prereleases are not \"latest\" (see docs/install.md)"
    WORK=$(mktemp -d "${TMPDIR:-/tmp}/websign-install.XXXXXX")
    trap cleanup EXIT
    trap on_signal INT TERM
    "install_$os"
    next_steps
}

[ "${WEBSIGN_INSTALL_LIB:-}" = 1 ] || main "$@"
