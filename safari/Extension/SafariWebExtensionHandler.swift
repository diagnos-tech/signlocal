// SafariWebExtensionHandler.swift — relay between the Safari web extension and the app.
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Designed and reviewed in docs/prototypes/2-mac.md §5; built with the store
// channel (docs/plan.md D10).
//
// Safari hands every browser.runtime.sendNativeMessage() to this extension. It forwards the
// message unchanged to the app over a Unix socket in the shared app group container, framed
// like native messaging, and returns the app's reply. The app owns everything else: the
// confirmation window, the signing, the OS PIN dialog. This file never parses the protocol.

import AppKit
import Foundation
import SafariServices
import os

private let appBundleID = "dev.websign.app"
private let appGroupID = "TEAMID.dev.websign"  // TODO(gustavo): real Team ID
private let socketName = "s.host"  // short: sun_path holds 104 bytes
private let startTimeout: TimeInterval = 10
private let maxReplyBytes = 1 << 20  // native messaging's host-to-browser limit
private let log = Logger(subsystem: "dev.websign.app.extension", category: "relay")

final class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {
    /// One request at a time: the app shows one confirmation window at a time anyway.
    private static let queue = DispatchQueue(label: "dev.websign.relay")

    func beginRequest(with context: NSExtensionContext) {
        let item = context.inputItems.first as? NSExtensionItem
        let message = item?.userInfo?[SFExtensionMessageKey]
        Self.queue.async {
            let reply: Any
            do {
                reply = try Relay.exchange(message)
            } catch {
                log.error("relay failed: \(String(describing: error), privacy: .public)")
                // Same shape as the app's own error replies (websign-protocol
                // `AppMessage::Error`), so the extension sees one format.
                let id = (message as? [String: Any])?["id"] ?? NSNull()
                reply = ["v": 1, "id": id, "type": "error",
                         "code": "AppMissing", "message": "\(error)"] as [String: Any]
            }
            let response = NSExtensionItem()
            response.userInfo = [SFExtensionMessageKey: reply]
            context.completeRequest(returningItems: [response], completionHandler: nil)
        }
    }
}

enum RelayError: Error { case notJSON, noAppGroup, appNotFound, tooLarge, closed, posix(Int32) }

enum Relay {
    static func exchange(_ message: Any?) throws -> Any {
        guard let message, JSONSerialization.isValidJSONObject(message) else { throw RelayError.notJSON }
        let fd = try connectToApp()
        defer { close(fd) }
        let body = try JSONSerialization.data(withJSONObject: message)
        var length = UInt32(body.count)  // native byte order, like native messaging
        try writeAll(Data(bytes: &length, count: 4) + body, to: fd)
        let header = try readExactly(4, from: fd)
        let replyLength = Int(header.withUnsafeBytes { $0.loadUnaligned(as: UInt32.self) })
        guard replyLength <= maxReplyBytes else { throw RelayError.tooLarge }
        return try JSONSerialization.jsonObject(with: readExactly(replyLength, from: fd))
    }

    /// Connects to the app's socket, starting the app once if nobody is listening yet.
    private static func connectToApp() throws -> Int32 {
        guard let group = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroupID)
        else { throw RelayError.noAppGroup }
        let path = group.appendingPathComponent(socketName).path
        let deadline = Date().addingTimeInterval(startTimeout)
        var launched = false
        while true {
            let fd = socket(AF_UNIX, SOCK_STREAM, 0)
            guard fd >= 0 else { throw RelayError.posix(errno) }
            if connectUnix(fd, path) == 0 { return fd }
            let failure = errno
            close(fd)
            guard failure == ENOENT || failure == ECONNREFUSED, Date() < deadline else {
                throw RelayError.posix(failure)
            }
            if !launched {
                try launchApp()
                launched = true
            }
            Thread.sleep(forTimeInterval: 0.1)
        }
    }

    private static func launchApp() throws {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: appBundleID)
        else { throw RelayError.appNotFound }
        let configuration = NSWorkspace.OpenConfiguration()
        configuration.activates = false  // the app raises its own confirmation window
        configuration.addsToRecentItems = false
        NSWorkspace.shared.openApplication(at: url, configuration: configuration)
    }

    private static func connectUnix(_ fd: Int32, _ path: String) -> Int32 {
        var address = sockaddr_un()
        address.sun_family = sa_family_t(AF_UNIX)
        let bytes = Array(path.utf8)
        guard bytes.count < MemoryLayout.size(ofValue: address.sun_path) else {
            errno = ENAMETOOLONG
            return -1
        }
        withUnsafeMutableBytes(of: &address.sun_path) { $0.copyBytes(from: bytes) }
        return withUnsafePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
            }
        }
    }

    private static func writeAll(_ data: Data, to fd: Int32) throws {
        try data.withUnsafeBytes { raw in
            var offset = 0
            while offset < raw.count {
                let written = write(fd, raw.baseAddress! + offset, raw.count - offset)
                guard written > 0 else { throw RelayError.posix(errno) }
                offset += written
            }
        }
    }

    private static func readExactly(_ count: Int, from fd: Int32) throws -> Data {
        var data = Data(count: count)
        try data.withUnsafeMutableBytes { raw in
            var offset = 0
            while offset < count {
                let got = read(fd, raw.baseAddress! + offset, count - offset)
                guard got > 0 else { throw got == 0 ? RelayError.closed : RelayError.posix(errno) }
                offset += got
            }
        }
        return data
    }
}
