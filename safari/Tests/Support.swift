// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
import XCTest
@testable import WebeSignRelay

/// Fake hosts: standard tools that behave like a host in one way each.
enum FakeHost {
    /// Echoes every frame back, like a host answering each message with itself.
    static let echo = HostCommand(executable: URL(fileURLWithPath: "/bin/cat"), arguments: [])

    static func shell(_ script: String) -> HostCommand {
        HostCommand(executable: URL(fileURLWithPath: "/bin/sh"), arguments: ["-c", script])
    }

    static let missing = HostCommand(
        executable: URL(fileURLWithPath: "/nonexistent/websign"), arguments: [])
}

/// A protocol-shaped message; the relay checks only its shape.
func envelope(_ id: String = "1", type: String = "status") -> [String: Any] {
    ["v": 1, "id": id, "type": type]
}

extension Relay {
    convenience init(host: HostCommand, idleTimeout: TimeInterval = 60) {
        self.init(command: { host }, idleTimeout: idleTimeout, exitGrace: 0.3)
    }

    /// Sends `message` and waits for the reply.
    func call(_ message: [String: Any], profile: String = "") -> [String: Any] {
        let done = DispatchSemaphore(value: 0)
        var answer: [String: Any] = [:]
        handle(message, profile: profile) { reply in
            answer = reply
            done.signal()
        }
        XCTAssertEqual(done.wait(timeout: .now() + 20), .success, "no reply")
        return answer
    }

    func open(profile: String = "") -> String {
        let reply = call(["relay": 1, "op": "open"], profile: profile)
        XCTAssertEqual(reply["open"] as? Bool, true, "\(reply)")
        return reply["session"] as? String ?? ""
    }

    func send(_ session: String, _ message: [String: Any]) -> [String: Any] {
        call(["relay": 1, "op": "send", "session": session, "message": message])
    }

    func poll(_ session: String, wait: Int = 5000, profile: String = "") -> [String: Any] {
        call(["relay": 1, "op": "poll", "session": session, "wait": wait], profile: profile)
    }

    /// Polls until the host has said `count` messages or the session closed.
    func collect(_ session: String, count: Int) -> (messages: [[String: Any]], open: Bool) {
        var messages: [[String: Any]] = []
        for _ in 0..<20 {
            let reply = poll(session, wait: 1000)
            messages += reply["messages"] as? [[String: Any]] ?? []
            let open = reply["open"] as? Bool ?? false
            if !open || messages.count >= count { return (messages, open) }
        }
        return (messages, true)
    }
}
