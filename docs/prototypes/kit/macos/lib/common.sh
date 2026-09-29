# shellcheck shell=bash
# Helpers shared by the macOS proof scripts. Source it, do not run it.
#
# Written for the bash 3.2 that ships with macOS, so the scripts also run on
# a plain Mac: no mapfile, no associative arrays, and empty arrays expanded
# with ${a[@]+"${a[@]}"} (bash 3.2 treats "${a[@]}" of an empty array as
# unset under `set -u`).

WEBSIGN_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)

# A string value from the repository's project.toml, e.g. native_host.
project_value() { # KEY
    sed -n "s/^$1 = \"\(.*\)\"$/\1/p" "$WEBSIGN_ROOT/project.toml" | head -n1
}

# Starts a collapsible group in the GitHub Actions log (plain title elsewhere).
section() {
    if [[ -n ${GITHUB_ACTIONS:-} ]]; then
        echo "::endgroup::"
        echo "::group::$*"
    else
        printf '\n== %s ==\n' "$*"
    fi
}

# Prints a warning that GitHub Actions also shows as an annotation.
warn() {
    if [[ -n ${GITHUB_ACTIONS:-} ]]; then
        echo "::warning::$*"
    else
        echo "warning: $*" >&2
    fi
}

# Runs a command with a time limit. A keychain or PIN dialog that nobody will
# answer on a CI runner then fails the step instead of hanging it. The alarm
# survives exec, so the command itself is killed (exit status 142).
# Usage: with_timeout SECONDS COMMAND [ARGS...]
with_timeout() {
    local seconds=$1
    shift
    perl -e 'alarm shift @ARGV; exec @ARGV or die "cannot run $ARGV[0]: $!\n"' "$seconds" "$@"
}

# The first 16 hex digits of a certificate's SHA-256: what `websign-probe
# list` prints and what `--cert` accepts.
cert_fingerprint() { # CERT_PEM
    "$OPENSSL" x509 -in "$1" -outform DER | shasum -a 256 | cut -c1-16
}

# Picks an OpenSSL able to write PKCS#12 files `security import` reads:
# Homebrew's OpenSSL 3 when present, else the system LibreSSL.
find_openssl() {
    local brew_prefix
    if brew_prefix=$(brew --prefix openssl@3 2>/dev/null) && [[ -x $brew_prefix/bin/openssl ]]; then
        OPENSSL=$brew_prefix/bin/openssl
    else
        OPENSSL=$(command -v openssl)
    fi
    echo "openssl: $OPENSSL ($("$OPENSSL" version))"
}

# Writes NAME.key.pem, NAME.cert.pem and NAME.p12 into DIR: a self-signed
# end-entity certificate allowed to sign documents, like an A1 certificate.
# KEY is rsa2048, p256 or p384. The .p12 password is P12_PASSWORD.
make_identity() { # DIR NAME KEY
    local name=$2 key=$3 base=$1/$2
    case $key in
    rsa2048) "$OPENSSL" genrsa -out "$base.key.pem" 2048 2>/dev/null ;;
    p256) "$OPENSSL" ecparam -name prime256v1 -genkey -noout -out "$base.key.pem" ;;
    p384) "$OPENSSL" ecparam -name secp384r1 -genkey -noout -out "$base.key.pem" ;;
    *) echo "unknown key type $key" >&2 && return 1 ;;
    esac
    # A config file rather than -addext, which LibreSSL lacks.
    cat >"$base.cnf" <<EOF
[req]
distinguished_name = dn
prompt = no
[dn]
CN = $name
O = WebeSign CI
[ext]
basicConstraints = critical, CA:FALSE
keyUsage = critical, digitalSignature, nonRepudiation
EOF
    "$OPENSSL" req -new -x509 -sha256 -days 2 -key "$base.key.pem" \
        -config "$base.cnf" -extensions ext -out "$base.cert.pem"
    # macOS imports only the legacy PKCS#12 algorithms reliably; LibreSSL
    # uses them by default, OpenSSL 3 must be told.
    local legacy=()
    if "$OPENSSL" version | grep -q '^OpenSSL 3'; then
        legacy=(-keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1)
    fi
    "$OPENSSL" pkcs12 -export -inkey "$base.key.pem" -in "$base.cert.pem" \
        -name "$name" -passout "pass:$P12_PASSWORD" ${legacy[@]+"${legacy[@]}"} \
        -out "$base.p12"
}

# Creates an unlocked throwaway keychain in ~/Library/Keychains, the folder
# a sandboxed process can open keychains from, and puts it first in the
# user's search list. keychain_destroy undoes both. NAME ends in .keychain;
# macOS stores it as NAME-db.
keychain_create() { # NAME PASSWORD
    KEYCHAIN_PASSWORD=$2
    KEYCHAIN_PATH=$HOME/Library/Keychains/$1-db
    security delete-keychain "$KEYCHAIN_PATH" 2>/dev/null || true
    security create-keychain -p "$KEYCHAIN_PASSWORD" "$1"
    # No lock on sleep, no lock after a timeout.
    security set-keychain-settings "$KEYCHAIN_PATH"
    security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"

    SAVED_SEARCH_LIST=()
    local line
    while IFS= read -r line; do
        line=${line#"${line%%[![:space:]]*}"}
        line=${line#\"}
        line=${line%\"}
        [[ -n $line ]] && SAVED_SEARCH_LIST+=("$line")
    done < <(security list-keychains -d user)
    security list-keychains -d user -s "$KEYCHAIN_PATH" ${SAVED_SEARCH_LIST[@]+"${SAVED_SEARCH_LIST[@]}"}
    echo "search list: $(security list-keychains -d user | tr -s ' \n' ' ')"
}

# Imports a .p12 so any program may use the key without an access dialog.
keychain_import() { # P12
    security import "$1" -k "$KEYCHAIN_PATH" -f pkcs12 -P "$P12_PASSWORD" -A >/dev/null
}

# Since macOS 10.12 a key also carries a partition list, checked on top of
# its ACL: code outside the list gets a password dialog even with -A.
# Allows Apple tools, unsigned code and the given binaries (by cdhash, the
# partition of ad-hoc signed code such as a locally built Rust binary).
keychain_allow() { # BINARY...
    local partitions=apple-tool:,apple:,unsigned: binary cdhash
    for binary in "$@"; do
        cdhash=$(codesign -dvvv "$binary" 2>&1 | sed -n 's/^CDHash=//p' | head -n1)
        [[ -n $cdhash ]] && partitions+=",cdhash:$cdhash"
    done
    echo "partition list: $partitions"
    security set-key-partition-list -S "$partitions" -s -k "$KEYCHAIN_PASSWORD" \
        "$KEYCHAIN_PATH" >/dev/null
}

# Restores the search list saved by keychain_create and deletes the keychain.
keychain_destroy() {
    [[ -n ${KEYCHAIN_PATH:-} ]] || return 0
    if [[ -n ${SAVED_SEARCH_LIST+set} ]]; then
        security list-keychains -d user -s ${SAVED_SEARCH_LIST[@]+"${SAVED_SEARCH_LIST[@]}"} || true
    fi
    security delete-keychain "$KEYCHAIN_PATH" 2>/dev/null || true
    KEYCHAIN_PATH=
}
