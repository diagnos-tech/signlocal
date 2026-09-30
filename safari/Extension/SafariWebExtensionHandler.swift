// SPDX-License-Identifier: GPL-3.0-or-later
//
// The Safari web extension's native side. Safari hands every
// browser.runtime.sendNativeMessage() of the extension's background to this
// class; the relay (Relay/, compiled into the same module) turns those
// one-shot messages into sessions with the bundled `websign` host
// (safari/SPEC.md). The host owns everything else: the confirmation window,
// the signing, the OS PIN dialog.

import Foundation
import SafariServices
import os

private let log = Logger(subsystem: "dev.websign.app.extension", category: "relay")

/// Named without the module prefix so `NSExtensionPrincipalClass` in the
/// Info.plist template does not depend on how the appex module is called.
@objc(SafariWebExtensionHandler)
final class SafariWebExtensionHandler: NSObject, NSExtensionRequestHandling {
    /// One relay per appex process: Safari creates a handler per message, but
    /// sessions (host processes) must outlive each of them.
    private static let relay = Relay(
        command: { try BundledHost.command(appex: Bundle.main.bundleURL) },
        // Fixed texts only (Relay never logs payloads), so they may be public.
        log: { event in log.info("\(event, privacy: .public)") })

    func beginRequest(with context: NSExtensionContext) {
        let item = context.inputItems.first as? NSExtensionItem
        let message = item?.userInfo?[SFExtensionMessageKey]
        Self.relay.handle(message, profile: Self.profile(of: item)) { reply in
            let response = NSExtensionItem()
            response.userInfo = [SFExtensionMessageKey: reply]
            context.completeRequest(returningItems: [response], completionHandler: nil)
        }
    }

    /// Safari 17+ names the profile the message comes from; sessions stay
    /// inside it. Earlier versions have one profile.
    private static func profile(of item: NSExtensionItem?) -> String {
        guard #available(macOS 14.0, *),
              let profile = item?.userInfo?[SFExtensionProfileKey] as? UUID
        else { return "" }
        return profile.uuidString
    }
}
