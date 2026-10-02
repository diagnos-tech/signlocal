// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// Native messaging framing, as the host speaks it on stdio: a `u32` length
/// in little-endian order (every macOS CPU is little-endian, and
/// `websign-protocol` writes the machine's order), then that many bytes of
/// UTF-8 JSON.
public enum Frame {
    /// The frame for `body`; callers keep `body` within
    /// ``RelayLimits/maxMessageBytes``.
    public static func encode(_ body: Data) -> Data {
        let length = UInt32(body.count)
        var frame = Data((0..<4).map { UInt8(truncatingIfNeeded: length >> (8 * $0)) })
        frame.append(body)
        return frame
    }
}

/// Why a stream stopped making sense.
public enum FrameError: Error, Equatable {
    /// A header announced more than ``RelayLimits/maxMessageBytes``: the peer
    /// is broken or hostile, and the rest of the stream is unusable.
    case tooLarge(Int)
}

/// Splits a byte stream into frame bodies, whatever the chunk boundaries.
public struct FrameReader {
    private var buffer = Data()

    public init() {}

    /// Adds `chunk` and returns every frame it completed, in order.
    public mutating func push(_ chunk: Data) throws -> [Data] {
        buffer.append(chunk)
        var frames: [Data] = []
        while buffer.count >= 4 {
            let length = buffer.prefix(4).enumerated().reduce(0) { sum, byte in
                sum | Int(byte.element) << (8 * byte.offset)
            }
            guard length <= RelayLimits.maxMessageBytes else { throw FrameError.tooLarge(length) }
            guard buffer.count >= 4 + length else { break }
            frames.append(Data(buffer.dropFirst(4).prefix(length)))
            // A fresh Data restarts indices at zero for the next header.
            buffer = Data(buffer.dropFirst(4 + length))
        }
        return frames
    }
}
