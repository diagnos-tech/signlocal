// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// One host process and the output it produced that the background has not
/// collected yet. Touched only on the relay's queue.
final class Session {
    let id: SessionID
    /// The Safari profile that opened it; other profiles cannot address it.
    let profile: String
    var host: HostProcess?
    private var reader = FrameReader()
    /// Host messages not yet collected, with their size as the host sent them.
    private var pending: [(message: [String: Any], bytes: Int)] = []
    private var pendingBytes = 0
    /// The host exited or broke the rules; what is pending is still delivered.
    private(set) var ended = false
    var lastUsed: Date
    /// The parked `poll`, answered when output arrives or its wait runs out.
    var waiter: (token: UUID, reply: ([String: Any]) -> Void)?

    init(id: SessionID, profile: String, now: Date) {
        self.id = id
        self.profile = profile
        lastUsed = now
    }

    /// Adds host output. Anything but well-framed, size-bounded JSON objects
    /// with a string `id` and `type` ends the session and drops what is
    /// pending: a host that breaks framing cannot be trusted for the rest.
    func receive(_ chunk: Data) {
        guard !ended else { return }
        do {
            for body in try reader.push(chunk) {
                guard let object = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                      object["id"] is String, object["type"] is String
                else { return fail() }
                pending.append((object, body.count))
                pendingBytes += body.count
            }
        } catch {
            return fail()
        }
        if pending.count > RelayLimits.maxPendingMessages || pendingBytes > RelayLimits.maxPendingBytes {
            fail()
        }
    }

    /// The host is gone (or must go); pending output stays deliverable.
    func end() {
        guard !ended else { return }
        ended = true
        host?.stop()
    }

    private func fail() {
        pending.removeAll()
        pendingBytes = 0
        end()
    }

    /// Takes the pending messages that fit in one reply: at least one, and
    /// no more than ``RelayLimits/maxMessageBytes`` in all unless the first
    /// alone is that large.
    func drain() -> [[String: Any]] {
        var taken: [[String: Any]] = []
        var bytes = 0
        while let next = pending.first {
            if !taken.isEmpty && bytes + next.bytes > RelayLimits.maxMessageBytes { break }
            pending.removeFirst()
            taken.append(next.message)
            bytes += next.bytes
            pendingBytes -= next.bytes
        }
        return taken
    }

    /// Whether the background may keep using this session.
    var isOpen: Bool { !(ended && pending.isEmpty) }

    var hasPending: Bool { !pending.isEmpty }
}
