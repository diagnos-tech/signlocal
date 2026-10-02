// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
#if canImport(Darwin)
import Darwin
#elseif canImport(Glibc)
import Glibc
#endif

/// Turns Safari's one-reply-per-message native messaging into the host's
/// long-lived stdio connection (`safari/SPEC.md`).
///
/// Safari gives the appex no connection and never lets it speak first: each
/// `sendNativeMessage` is one request with one reply. So the background opens
/// a session (one host process, like one `connectNative` port elsewhere),
/// sends protocol messages into it, and polls for what the host says. All
/// state lives on one serial queue; replies never block it.
public final class Relay {
    private let queue = DispatchQueue(label: "dev.websign.relay")
    private let command: () throws -> HostCommand
    private let idleTimeout: TimeInterval
    private let exitGrace: TimeInterval
    private let log: (String) -> Void
    private var sessions: [SessionID: Session] = [:]
    private var reaper: DispatchSourceTimer?

    /// `command` names the host to start per session. `log` receives fixed
    /// event texts only, never payloads.
    public init(
        command: @escaping () throws -> HostCommand,
        idleTimeout: TimeInterval = RelayLimits.idleTimeout,
        exitGrace: TimeInterval = RelayLimits.exitGrace,
        log: @escaping (String) -> Void = { _ in }
    ) {
        // A write to a host that just exited must fail with EPIPE, not kill
        // the appex (and every other session) with SIGPIPE.
        signal(SIGPIPE, SIG_IGN)
        self.command = command
        self.idleTimeout = idleTimeout
        self.exitGrace = exitGrace
        self.log = log
        let timer = DispatchSource.makeTimerSource(queue: queue)
        let every = max(idleTimeout / 4, 0.05)
        timer.schedule(deadline: .now() + every, repeating: every)
        timer.setEventHandler { [weak self] in self?.reapIdle() }
        timer.resume()
        reaper = timer
    }

    deinit {
        reaper?.cancel()
        for session in sessions.values { session.end() }
    }

    /// Handles one message from the background; `reply` is called exactly
    /// once, on the relay's queue. `profile` is Safari's profile identifier
    /// ("" before Safari 17).
    public func handle(_ message: Any?, profile: String, reply: @escaping ([String: Any]) -> Void) {
        queue.async {
            do {
                switch try RelayRequest.parse(message) {
                case .open:
                    reply(try self.open(profile: profile))
                case let .send(id, body):
                    let session = try self.session(id, profile: profile)
                    session.host?.write(Frame.encode(body))
                    self.answer(session, reply)
                case let .poll(id, wait):
                    self.park(try self.session(id, profile: profile), wait: wait, reply)
                case let .close(id):
                    let session = try self.session(id, profile: profile)
                    self.remove(session, why: "closed")
                    reply(RelayReply.session(id, messages: [], open: false))
                }
            } catch let error as RelayError {
                reply(RelayReply.failure(error))
            } catch {
                reply(RelayReply.failure(.badRequest))
            }
        }
    }

    private func open(profile: String) throws -> [String: Any] {
        guard sessions.count < RelayLimits.maxSessions else { throw RelayError.tooManySessions }
        let session = Session(id: SessionID(), profile: profile, now: Date())
        do {
            session.host = try HostProcess(
                try command(), events: queue, grace: exitGrace,
                onChunk: { [weak self, weak session] chunk in
                    guard let session else { return }
                    session.receive(chunk)
                    self?.wake(session)
                },
                onEnd: { [weak self, weak session] in
                    guard let session else { return }
                    session.end()
                    self?.wake(session)
                })
        } catch {
            log("host could not be started")
            throw RelayError.hostMissing
        }
        sessions[session.id] = session
        log("session opened")
        return RelayReply.session(session.id, messages: [], open: true)
    }

    private func session(_ id: SessionID, profile: String) throws -> Session {
        guard let session = sessions[id], session.profile == profile else { throw RelayError.noSession }
        session.lastUsed = Date()
        // A newer request supersedes a parked poll; it gets an empty answer.
        if let waiter = session.waiter {
            session.waiter = nil
            waiter.reply(RelayReply.session(id, messages: [], open: true))
        }
        return session
    }

    private func park(_ session: Session, wait: TimeInterval, _ reply: @escaping ([String: Any]) -> Void) {
        guard !session.hasPending, !session.ended, wait > 0 else { return answer(session, reply) }
        let token = UUID()
        session.waiter = (token, reply)
        queue.asyncAfter(deadline: .now() + wait) { [weak self, weak session] in
            guard let session, session.waiter?.token == token else { return }
            self?.wake(session)
        }
    }

    /// Answers the parked poll, if any, with what is pending now.
    private func wake(_ session: Session) {
        guard let waiter = session.waiter else { return }
        session.waiter = nil
        answer(session, waiter.reply)
    }

    private func answer(_ session: Session, _ reply: ([String: Any]) -> Void) {
        let messages = session.drain()
        let open = session.isOpen
        if !open { remove(session, why: "host ended") }
        reply(RelayReply.session(session.id, messages: messages, open: open))
    }

    private func remove(_ session: Session, why: String) {
        guard sessions.removeValue(forKey: session.id) != nil else { return }
        session.end()
        log("session ended: \(why)")
    }

    private func reapIdle() {
        let cutoff = Date().addingTimeInterval(-idleTimeout)
        for session in sessions.values where session.waiter == nil && session.lastUsed < cutoff {
            remove(session, why: "idle")
        }
    }
}
