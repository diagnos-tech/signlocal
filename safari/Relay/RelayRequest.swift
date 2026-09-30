// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// One message from the extension's background, validated. The contract is
/// `safari/SPEC.md` §Relay messages: exact keys, nothing else, so a typo or a
/// stranger's shape is refused here instead of reaching the host.
public enum RelayRequest: Equatable {
    /// Start a host process: a new session.
    case open
    /// Forward one protocol message (its JSON, re-encoded) to the host.
    case send(SessionID, Data)
    /// Wait up to `wait` seconds for host output.
    case poll(SessionID, wait: TimeInterval)
    /// End the session: the host's stdin closes, then the process is stopped.
    case close(SessionID)

    /// Value of the `relay` key this appex speaks.
    public static let version = 1

    /// The protocol's client message types (`ClientMessage` in
    /// `websign-protocol`); a test keeps this list equal to the generated one.
    public static let clientTypes: Set<String> = [
        "hello", "status", "choose", "sign.begin", "sign.digest", "cancel", "diagnostics.open",
    ]

    /// Validates what Safari handed to the appex.
    public static func parse(_ value: Any?) throws -> RelayRequest {
        guard let object = value as? [String: Any],
              JSONNumber.integer(object["relay"]) == version,
              let op = object["op"] as? String
        else { throw RelayError.badRequest }
        switch op {
        case "open":
            try requireKeys(object, [])
            return .open
        case "send":
            try requireKeys(object, ["session", "message"])
            return .send(try session(object), try envelope(object["message"]))
        case "poll":
            try requireKeys(object, ["session", "wait"])
            let maxWait = Int(RelayLimits.maxPollWait * 1000)
            guard let wait = JSONNumber.integer(object["wait"]), (0...maxWait).contains(wait)
            else { throw RelayError.badRequest }
            return .poll(try session(object), wait: TimeInterval(wait) / 1000)
        case "close":
            try requireKeys(object, ["session"])
            return .close(try session(object))
        default:
            throw RelayError.badRequest
        }
    }

    /// Exactly `relay`, `op` and `extra`: unknown keys are refused (D12).
    private static func requireKeys(_ object: [String: Any], _ extra: Set<String>) throws {
        guard Set(object.keys) == extra.union(["relay", "op"]) else { throw RelayError.badRequest }
    }

    private static func session(_ object: [String: Any]) throws -> SessionID {
        guard let id = SessionID(object["session"]) else { throw RelayError.badRequest }
        return id
    }

    /// A protocol envelope by shape only (the host validates the content):
    /// an object with a request id and a known client type, within the size
    /// limit once encoded.
    private static func envelope(_ value: Any?) throws -> Data {
        guard let message = value as? [String: Any],
              JSONSerialization.isValidJSONObject(message),
              let id = message["id"] as? String,
              (1...RelayLimits.maxRequestIdBytes).contains(id.utf8.count),
              let type = message["type"] as? String,
              clientTypes.contains(type),
              let body = try? JSONSerialization.data(withJSONObject: message),
              body.count <= RelayLimits.maxMessageBytes
        else { throw RelayError.badRequest }
        return body
    }
}

/// A session's name: a random UUID the appex made, so a background can only
/// address sessions it was told about.
public struct SessionID: Hashable {
    public let text: String

    /// A new, random ID.
    public init() { text = UUID().uuidString }

    /// Parses an ID received from the background; only canonical UUIDs.
    public init?(_ value: Any?) {
        guard let text = value as? String, text.utf8.count == 36,
              let uuid = UUID(uuidString: text), uuid.uuidString == text
        else { return nil }
        self.text = text
    }
}

/// Why a request got no session reply. The raw values are the wire codes.
public enum RelayError: String, Error {
    /// Not a relay message this appex understands.
    case badRequest = "BadRequest"
    /// The session does not exist (any more), or belongs to another profile.
    case noSession = "NoSession"
    /// ``RelayLimits/maxSessions`` hosts are already running.
    case tooManySessions = "TooManySessions"
    /// The bundled host is missing or could not be started.
    case hostMissing = "HostMissing"
}

/// JSON numbers as Safari and JSONSerialization hand them over (NSNumber),
/// read strictly: booleans are not numbers, fractions are not integers.
enum JSONNumber {
    static func integer(_ value: Any?) -> Int? {
        guard let number = value as? NSNumber, !isBoolean(number) else { return nil }
        let double = number.doubleValue
        guard double.rounded() == double, abs(double) <= 1e15 else { return nil }
        return Int(double)
    }

    private static func isBoolean(_ number: NSNumber) -> Bool {
        #if canImport(Darwin)
        return CFGetTypeID(number) == CFBooleanGetTypeID()
        #else
        return String(cString: number.objCType) == "c"
        #endif
    }
}
