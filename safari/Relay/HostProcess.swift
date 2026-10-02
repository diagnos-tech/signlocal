// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation
#if canImport(Darwin)
import Darwin
#elseif canImport(Glibc)
import Glibc
#endif

/// What to start for each session. In the appex: the `websign` binary
/// bundled next to it, with browser-launch arguments (`BundledHost.swift`).
public struct HostCommand {
    public let executable: URL
    public let arguments: [String]

    public init(executable: URL, arguments: [String]) {
        self.executable = executable
        self.arguments = arguments
    }
}

/// One `websign` process speaking native messaging on its stdio.
///
/// Everything it reports arrives on the relay's queue. Writes go through a
/// queue of their own: a pipe blocks once the host stops reading, and that
/// must never stall the other sessions.
final class HostProcess {
    private let process = Process()
    private let input = Pipe()
    private let output = Pipe()
    private let writes = DispatchQueue(label: "dev.websign.relay.write")
    private let events: DispatchQueue
    private let grace: TimeInterval

    /// Starts `command`. `onChunk` gets stdout as it comes; `onEnd` runs once,
    /// when stdout ends or the process exits, whichever is first.
    init(
        _ command: HostCommand,
        events: DispatchQueue,
        grace: TimeInterval = RelayLimits.exitGrace,
        onChunk: @escaping (Data) -> Void,
        onEnd: @escaping () -> Void
    ) throws {
        self.events = events
        self.grace = grace
        process.executableURL = command.executable
        process.arguments = command.arguments
        process.standardInput = input
        process.standardOutput = output
        // The host logs to its own file; its stderr is never relayed, so no
        // payload can leak into Safari's or the system's logs through it.
        process.standardError = FileHandle.nullDevice
        var ended = false
        let end = {
            guard !ended else { return }
            ended = true
            onEnd()
        }
        // Output still in the pipe when the process exits is delivered first:
        // the reader below posts it before this delayed end runs.
        process.terminationHandler = { _ in
            events.asyncAfter(deadline: .now() + 0.2) { end() }
        }
        try process.run()
        let reader = output.fileHandleForReading
        DispatchQueue.global(qos: .userInitiated).async {
            while let chunk = Self.readSome(reader.fileDescriptor) {
                events.async { onChunk(chunk) }
            }
            events.async { end() }
        }
    }

    /// Queues one frame for the host's stdin. A failed write means the host
    /// is gone, which its end event reports.
    func write(_ frame: Data) {
        let handle = input.fileHandleForWriting
        writes.async { try? handle.write(contentsOf: frame) }
    }

    /// Closes stdin (the host's own way to finish: it cancels what is open
    /// and exits), then terminates and finally kills a host that lingers.
    func stop() {
        let handle = input.fileHandleForWriting
        writes.async { try? handle.close() }
        let process = self.process
        events.asyncAfter(deadline: .now() + grace) {
            guard process.isRunning else { return }
            process.terminate()
            self.events.asyncAfter(deadline: .now() + self.grace) {
                if process.isRunning { kill(process.processIdentifier, SIGKILL) }
            }
        }
    }

    var isRunning: Bool { process.isRunning }

    /// Whatever the pipe holds, blocking until something arrives; `nil` at
    /// the end of the stream. POSIX `read` because FileHandle's reads either
    /// wait for a full count or raise Objective-C exceptions on errors.
    private static func readSome(_ fd: Int32) -> Data? {
        var buffer = [UInt8](repeating: 0, count: 64 * 1024)
        while true {
            let count = read(fd, &buffer, buffer.count)
            if count > 0 { return Data(buffer[0..<count]) }
            if count < 0 && errno == EINTR { continue }
            return nil
        }
    }
}
