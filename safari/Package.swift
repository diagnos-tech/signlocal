// swift-tools-version:5.9
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The platform-neutral relay and its tests (`swift test`, also on Linux).
// The appex itself is compiled by `cargo xtask package` (xtask/src/package/
// safari.rs) from `Relay/` plus `Extension/` as one module, so the handler
// uses these types without an import.

import PackageDescription

let package = Package(
    name: "SignLocalSafari",
    platforms: [.macOS(.v13)],
    targets: [
        .target(name: "SignLocalRelay", path: "Relay", exclude: ["SUMMARY.md"]),
        .testTarget(
            name: "SignLocalRelayTests",
            dependencies: ["SignLocalRelay"],
            path: "Tests",
            exclude: ["SUMMARY.md"]
        ),
    ]
)
