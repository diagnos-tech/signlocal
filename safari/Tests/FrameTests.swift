// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
import XCTest
@testable import SignLocalRelay

final class FrameTests: XCTestCase {
    func testTheLengthIsLittleEndian() {
        XCTAssertEqual([UInt8](Frame.encode(Data("{}".utf8))), [2, 0, 0, 0, 0x7B, 0x7D])
    }

    func testFramesSurviveAnyChunking() throws {
        let stream = Frame.encode(Data("{\"a\":1}".utf8)) + Frame.encode(Data("[]".utf8))
        var reader = FrameReader()
        var frames: [Data] = []
        for byte in stream { frames += try reader.push(Data([byte])) }
        XCTAssertEqual(frames.map { String(decoding: $0, as: UTF8.self) }, ["{\"a\":1}", "[]"])
    }

    func testAnOversizedHeaderIsRefusedBeforeAllocating() {
        var reader = FrameReader()
        XCTAssertThrowsError(try reader.push(Data([0xFF, 0xFF, 0xFF, 0x7F]))) { error in
            XCTAssertEqual(error as? FrameError, .tooLarge(0x7FFF_FFFF))
        }
    }
}
