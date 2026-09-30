// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// The two reply shapes of `safari/SPEC.md` §Relay messages.
enum RelayReply {
    /// Host output for `session`; `open: false` means the host is gone and
    /// the session with it.
    static func session(_ id: SessionID, messages: [[String: Any]], open: Bool) -> [String: Any] {
        ["relay": RelayRequest.version, "session": id.text, "messages": messages, "open": open]
    }

    static func failure(_ error: RelayError) -> [String: Any] {
        ["relay": RelayRequest.version, "error": error.rawValue]
    }
}
