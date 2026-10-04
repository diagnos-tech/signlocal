// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
import XCTest
@testable import SignLocalRelay

final class HostProcessTests: XCTestCase {
    /// A host that ignores both its closed stdin and SIGTERM is still gone
    /// once both graces have passed.
    func testAStubbornHostIsKilled() throws {
        let queue = DispatchQueue(label: "test")
        let ended = expectation(description: "ended")
        let host = try HostProcess(
            FakeHost.shell("trap '' TERM; while :; do sleep 1; done"),
            events: queue, grace: 0.3,
            onChunk: { _ in }, onEnd: { ended.fulfill() })
        XCTAssertTrue(host.isRunning)
        queue.sync { host.stop() }
        wait(for: [ended], timeout: 10)
        // The exit is reaped asynchronously; give it a moment to show.
        let deadline = Date().addingTimeInterval(5)
        while host.isRunning && Date() < deadline { Thread.sleep(forTimeInterval: 0.05) }
        XCTAssertFalse(host.isRunning)
    }

    /// Closing stdin is enough for a well-behaved host.
    func testClosingStdinEndsAWellBehavedHost() throws {
        let queue = DispatchQueue(label: "test")
        let ended = expectation(description: "ended")
        let host = try HostProcess(
            FakeHost.echo, events: queue, grace: 30,
            onChunk: { _ in }, onEnd: { ended.fulfill() })
        queue.sync { host.stop() }
        wait(for: [ended], timeout: 5)
    }
}
