// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// The host command for an appex laid out by `cargo xtask package`
/// (`xtask/src/package/safari.rs`):
///
/// ```
/// <appex>/Contents/Info.plist          WebeSignHostExecutable, WebeSignHostExtensionID
/// <appex>/Contents/MacOS/websign       the app's own binary, signed to inherit the sandbox
/// <appex>/Contents/Resources/manifest.json   the WXT safari build
/// ```
///
/// The binary is a copy inside the appex, not the app's `Contents/MacOS/websign`:
/// a sandboxed appex may always read and execute its own bundle, while
/// reaching into the containing app's is not something the sandbox promises.
///
/// The arguments are the shape Firefox uses (manifest path, extension ID),
/// which the host's launch detection accepts for the ID in `project.toml`.
/// TODO(gustavo): switch to a Safari launch shape once `websign-host` has one
/// (`safari/SPEC.md` §Host launch), so diagnostics can tell Safari apart.
public enum BundledHost {
    public static func command(appex: URL) throws -> HostCommand {
        let contents = appex.appendingPathComponent("Contents")
        guard let data = try? Data(contentsOf: contents.appendingPathComponent("Info.plist")),
              let info = try? PropertyListSerialization.propertyList(from: data, format: nil)
                as? [String: Any],
              let executable = info["WebeSignHostExecutable"] as? String,
              let extensionID = info["WebeSignHostExtensionID"] as? String,
              !executable.contains("/")
        else { throw RelayError.hostMissing }
        let binary = contents.appendingPathComponent("MacOS").appendingPathComponent(executable)
        let manifest = contents.appendingPathComponent("Resources/manifest.json")
        guard FileManager.default.isExecutableFile(atPath: binary.path),
              FileManager.default.fileExists(atPath: manifest.path)
        else { throw RelayError.hostMissing }
        return HostCommand(executable: binary, arguments: [manifest.path, extensionID])
    }
}
