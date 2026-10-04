// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
import XCTest
@testable import SignLocalRelay

final class RelayTests: XCTestCase {
    func testMessagesReachTheHostAndItsRepliesComeBack() {
        let relay = Relay(host: FakeHost.echo)
        let session = relay.open()
        var echoed: [[String: Any]] = []
        for id in ["a", "b"] {
            // A send's reply carries whatever the host said by then.
            let reply = relay.send(session, envelope(id))
            XCTAssertEqual(reply["session"] as? String, session)
            echoed += reply["messages"] as? [[String: Any]] ?? []
        }
        let (messages, open) = relay.collect(session, count: 2 - echoed.count)
        XCTAssertEqual((echoed + messages).map { $0["id"] as? String }, ["a", "b"])
        XCTAssertTrue(open)
    }

    func testAPollWithNothingToSayWaitsThenAnswersEmpty() {
        let relay = Relay(host: FakeHost.echo)
        let session = relay.open()
        let started = Date()
        let reply = relay.poll(session, wait: 300)
        XCTAssertGreaterThanOrEqual(Date().timeIntervalSince(started), 0.25)
        XCTAssertEqual((reply["messages"] as? [Any])?.count, 0)
        XCTAssertEqual(reply["open"] as? Bool, true)
    }

    func testClosingEndsTheSession() {
        let relay = Relay(host: FakeHost.echo)
        let session = relay.open()
        XCTAssertEqual(relay.call(["relay": 1, "op": "close", "session": session])["open"] as? Bool, false)
        XCTAssertEqual(relay.poll(session)["error"] as? String, "NoSession")
    }

    func testAHostThatExitsClosesTheSessionAfterItsLastWords() {
        let relay = Relay(host: FakeHost.shell(#"printf '\030\000\000\000{"id":"x","type":"done"}'"#))
        let session = relay.open()
        let (messages, open) = relay.collect(session, count: 2)
        XCTAssertEqual(messages.map { $0["type"] as? String }, ["done"])
        XCTAssertFalse(open)
        XCTAssertEqual(relay.poll(session)["error"] as? String, "NoSession")
    }

    func testAHostThatBreaksFramingIsDropped() {
        let relay = Relay(host: FakeHost.shell(#"printf '\377\377\377\377'; sleep 5"#))
        let session = relay.open()
        let (messages, open) = relay.collect(session, count: 1)
        XCTAssertTrue(messages.isEmpty)
        XCTAssertFalse(open)
    }

    func testAHostThatSendsNonEnvelopesIsDropped() {
        let relay = Relay(host: FakeHost.shell(#"printf '\002\000\000\000[]'; sleep 5"#))
        let (messages, open) = relay.collect(relay.open(), count: 1)
        XCTAssertTrue(messages.isEmpty)
        XCTAssertFalse(open)
    }

    func testAMissingHostIsReported() {
        let relay = Relay(host: FakeHost.missing)
        XCTAssertEqual(relay.call(["relay": 1, "op": "open"])["error"] as? String, "HostMissing")
    }

    func testSessionsBelongToTheirProfile() {
        let relay = Relay(host: FakeHost.echo)
        let session = relay.open(profile: "A")
        XCTAssertEqual(relay.poll(session, wait: 0, profile: "B")["error"] as? String, "NoSession")
        XCTAssertEqual(relay.poll(session, wait: 0, profile: "A")["open"] as? Bool, true)
    }

    func testTheNumberOfHostsIsBounded() {
        let relay = Relay(host: FakeHost.echo)
        for _ in 0..<RelayLimits.maxSessions { _ = relay.open() }
        XCTAssertEqual(relay.call(["relay": 1, "op": "open"])["error"] as? String, "TooManySessions")
    }

    func testABacklogNobodyCollectsEndsTheSession() {
        let frame = #"printf '\030\000\000\000{"id":"x","type":"done"}'"#
        let count = RelayLimits.maxPendingMessages + 1
        let relay = Relay(host: FakeHost.shell("for i in $(seq \(count)); do \(frame); done; sleep 5"))
        let session = relay.open()
        Thread.sleep(forTimeInterval: 1)
        let (messages, open) = relay.collect(session, count: 1)
        XCTAssertTrue(messages.isEmpty)
        XCTAssertFalse(open)
    }

    func testAnAbandonedSessionIsReaped() {
        let relay = Relay(host: FakeHost.echo, idleTimeout: 0.4)
        let session = relay.open()
        Thread.sleep(forTimeInterval: 1.2)
        XCTAssertEqual(relay.poll(session, wait: 0)["error"] as? String, "NoSession")
    }

    func testAMalformedRequestGetsBadRequest() {
        let relay = Relay(host: FakeHost.echo)
        XCTAssertEqual(relay.call(["relay": 1, "op": "open", "x": 1])["error"] as? String, "BadRequest")
    }
}
