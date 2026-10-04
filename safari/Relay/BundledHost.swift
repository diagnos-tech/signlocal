// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// The host command for an appex laid out by `cargo xtask package`
/// (`xtask/src/package/safari.rs`):
///
/// ```
/// <appex>/Contents/Info.plist          SignLocalHostExecutable, SignLocalHostExtensionID
/// <appex>/Contents/MacOS/websign       the app's own binary, signed to inherit the sandbox
/// ```
///
/// The binary is a copy inside the appex, not the app's `Contents/MacOS/websign`:
/// a sandboxed appex may always read and execute its own bundle, while
/// reaching into the containing app's is not something the sandbox promises.
///
/// The arguments are the host's Safari launch shape: `--safari-web-extension`
/// and this appex's bundle ID (`SignLocalHostExtensionID`). The host accepts
/// that shape only when its own executable sits inside an `.appex`, so the
/// copy here is the only one it starts (`safari/SPEC.md` §4).
public enum BundledHost {
    /// `websign_host::launch::SAFARI_FLAG`.
    public static let safariFlag = "--safari-web-extension"

    public static func command(appex: URL) throws -> HostCommand {
        let contents = appex.appendingPathComponent("Contents")
        guard let data = try? Data(contentsOf: contents.appendingPathComponent("Info.plist")),
              let info = try? PropertyListSerialization.propertyList(from: data, format: nil)
                as? [String: Any],
              let executable = info["SignLocalHostExecutable"] as? String,
              let extensionID = info["SignLocalHostExtensionID"] as? String,
              !executable.contains("/")
        else { throw RelayError.hostMissing }
        let binary = contents.appendingPathComponent("MacOS").appendingPathComponent(executable)
        guard FileManager.default.isExecutableFile(atPath: binary.path) else {
            throw RelayError.hostMissing
        }
        return HostCommand(executable: binary, arguments: [safariFlag, extensionID])
    }
}
