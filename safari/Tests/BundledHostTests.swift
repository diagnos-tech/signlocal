// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
import XCTest
@testable import WebeSignRelay

/// The real host, from a packaged appex (`WEBSIGN_TEST_APPEX`), through the
/// same relay the handler uses: the launch shape is accepted and `hello`
/// comes back. Safari itself is the one piece left out (docs/compatibility.md).
final class BundledHostTests: XCTestCase {
    func testTheBundledHostAnswersHello() throws {
        guard let path = ProcessInfo.processInfo.environment["WEBSIGN_TEST_APPEX"] else {
            throw XCTSkip("set WEBSIGN_TEST_APPEX to a packaged .appex")
        }
        let appex = URL(fileURLWithPath: path)
        let bundled = try BundledHost.command(appex: appex)
        // The appex's copy is signed to inherit a sandbox, and macOS kills
        // such a binary when a process outside any sandbox (this test)
        // starts it. The app's own binary is the same build, signed plainly.
        let app = appex.deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("MacOS").appendingPathComponent(bundled.executable.lastPathComponent)
        let command = HostCommand(executable: app, arguments: bundled.arguments)
        let relay = Relay(command: { command })
        let session = relay.open()
        let hello: [String: Any] = [
            "v": 1, "id": "hello", "type": "hello",
            "client": ["name": "websign-extension", "version": try version(of: appex)],
            "protocols": ["min": 1, "max": 1],
            "browser": ["name": "safari", "version": "18.0", "reason": "page"],
        ]
        var messages = relay.send(session, hello)["messages"] as? [[String: Any]] ?? []
        if messages.isEmpty { messages = relay.collect(session, count: 1).messages }
        XCTAssertEqual(messages.first?["type"] as? String, "hello", "\(messages)")
        XCTAssertEqual(messages.first?["id"] as? String, "hello")
        _ = relay.call(["relay": 1, "op": "close", "session": session])
    }

    private func version(of appex: URL) throws -> String {
        let data = try Data(contentsOf: appex.appendingPathComponent("Contents/Info.plist"))
        let info = try PropertyListSerialization.propertyList(from: data, format: nil) as? [String: Any]
        return try XCTUnwrap(info?["CFBundleShortVersionString"] as? String)
    }

    func testAnIncompleteBundleIsHostMissing() {
        let empty = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(UUID().uuidString)
        XCTAssertThrowsError(try BundledHost.command(appex: empty)) { error in
            XCTAssertEqual(error as? RelayError, .hostMissing)
        }
    }
}
