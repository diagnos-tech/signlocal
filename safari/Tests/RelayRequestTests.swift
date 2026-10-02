// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
import XCTest
@testable import WebeSignRelay

final class RelayRequestTests: XCTestCase {
    private let session = UUID().uuidString

    private func refused(_ value: Any?, file: StaticString = #filePath, line: UInt = #line) {
        XCTAssertThrowsError(try RelayRequest.parse(value), file: file, line: line) { error in
            XCTAssertEqual(error as? RelayError, .badRequest, file: file, line: line)
        }
    }

    func testEveryOperationParses() throws {
        let id = try XCTUnwrap(SessionID(session))
        XCTAssertEqual(try RelayRequest.parse(["relay": 1, "op": "open"]), .open)
        XCTAssertEqual(
            try RelayRequest.parse(["relay": 1, "op": "poll", "session": session, "wait": 2500]),
            .poll(id, wait: 2.5))
        XCTAssertEqual(try RelayRequest.parse(["relay": 1, "op": "close", "session": session]), .close(id))
        guard case let .send(sent, body) = try RelayRequest.parse(
            ["relay": 1, "op": "send", "session": session, "message": envelope()])
        else { return XCTFail("not a send") }
        XCTAssertEqual(sent, id)
        let decoded = try JSONSerialization.jsonObject(with: body) as? [String: Any]
        XCTAssertEqual(decoded?["type"] as? String, "status")
    }

    func testTheShapeIsExact() {
        refused(nil)
        refused("open")
        refused(["op": "open"])
        refused(["relay": 2, "op": "open"])
        refused(["relay": true, "op": "open"])
        refused(["relay": 1.5, "op": "open"])
        refused(["relay": 1, "op": "open", "extra": 1])
        refused(["relay": 1, "op": "launch"])
        refused(["relay": 1, "op": "close"])
        refused(["relay": 1, "op": "poll", "session": session])
    }

    func testSessionsAreCanonicalUUIDs() {
        refused(["relay": 1, "op": "close", "session": "../../etc"])
        refused(["relay": 1, "op": "close", "session": session.lowercased()])
        refused(["relay": 1, "op": "close", "session": 7])
    }

    func testPollWaitsAreBounded() {
        refused(["relay": 1, "op": "poll", "session": session, "wait": -1])
        refused(["relay": 1, "op": "poll", "session": session, "wait": 10_001])
        refused(["relay": 1, "op": "poll", "session": session, "wait": false])
    }

    func testOnlyProtocolShapedMessagesAreForwarded() {
        let send = { (message: Any) -> [String: Any] in
            ["relay": 1, "op": "send", "session": self.session, "message": message]
        }
        refused(send("hello"))
        refused(send(["v": 1, "type": "status"]))
        refused(send(envelope("", type: "status")))
        refused(send(envelope(String(repeating: "x", count: 65))))
        refused(send(envelope(type: "sign.result")))
        refused(send(envelope(type: "shell")))
        var huge = envelope()
        huge["padding"] = String(repeating: "x", count: RelayLimits.maxMessageBytes)
        refused(send(huge))
    }

    /// The appex ships in the same bundle as the host, so its list of client
    /// types must be exactly the protocol's (the generated TypeScript).
    func testClientTypesMatchTheProtocol() throws {
        let generated = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("extension/src/generated/ClientMessage.ts")
        let text = try String(contentsOf: generated, encoding: .utf8)
        let pattern = try NSRegularExpression(pattern: #"\{ "type": "([a-z._]+)" \}"#)
        let range = NSRange(text.startIndex..., in: text)
        let types = pattern.matches(in: text, range: range).compactMap { match in
            Range(match.range(at: 1), in: text).map { String(text[$0]) }
        }
        XCTAssertFalse(types.isEmpty)
        XCTAssertEqual(Set(types), RelayRequest.clientTypes)
    }
}
