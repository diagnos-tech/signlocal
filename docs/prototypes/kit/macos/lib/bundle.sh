# shellcheck shell=bash
# Wraps websign-probe in WebeSign.app, signed ad hoc with the Mac App Store
# entitlements, so the probe runs inside the App Sandbox exactly as the store
# app would (same bundle identifier, same container). Source it after
# common.sh.
#
# Keys that only a provisioning profile can grant are removed: with them, an
# ad hoc signed binary is killed at launch.

SANDBOX_FILES=$(cd "$(dirname "${BASH_SOURCE[0]}")/../sandbox" && pwd)
readonly PROFILE_ONLY_KEYS=(
    com.apple.application-identifier
    com.apple.developer.team-identifier
    com.apple.security.application-groups
    keychain-access-groups
)

# Builds DIR/WebeSign.app around PROBE and sets bundle_id, container (the
# sandbox's Data folder), app, exe and entitlements. Extra arguments go to
# codesign, e.g. --options runtime. Returns non-zero if codesign fails.
build_sandboxed_app() { # PROBE DIR [CODESIGN_FLAGS...]
    local probe=$1 dir=$2 key expected
    shift 2
    bundle_id=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$SANDBOX_FILES/Info.plist")
    expected=$(project_value macos_bundle_id)
    if [[ $bundle_id != "$expected" ]]; then
        echo "Info.plist says $bundle_id but project.toml says $expected" >&2
        return 1
    fi
    # shellcheck disable=SC2034 # read by the scripts that source this file
    container=$HOME/Library/Containers/$bundle_id/Data

    mkdir -p "$dir"
    entitlements=$dir/entitlements.plist
    cp "$SANDBOX_FILES/entitlements.mas.plist" "$entitlements"
    for key in "${PROFILE_ONLY_KEYS[@]}"; do
        /usr/libexec/PlistBuddy -c "Delete :$key" "$entitlements" 2>/dev/null &&
            echo "entitlement removed for the ad hoc build: $key"
    done

    app=$dir/WebeSign.app
    exe=$app/Contents/MacOS/websign-probe
    rm -rf "$app"
    mkdir -p "$app/Contents/MacOS"
    cp "$SANDBOX_FILES/Info.plist" "$app/Contents/Info.plist"
    cp "$probe" "$exe"
    resign_sandboxed_app "$@"
}

# Signs the bundle again, e.g. with the hardened runtime of a Developer ID
# build. A new signature changes the cdhash, and with it the keychain
# partition the binary belongs to.
resign_sandboxed_app() { # [CODESIGN_FLAGS...]
    codesign --force --sign - --entitlements "$entitlements" "$@" "$app" &&
        codesign --verify --strict "$app"
}
