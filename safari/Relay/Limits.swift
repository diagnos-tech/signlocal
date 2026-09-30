// SPDX-License-Identifier: GPL-3.0-or-later

import Foundation

/// Every size and time bound of the relay. The host enforces the protocol's
/// own limits; these keep the appex itself safe from a buggy or hostile peer
/// on either side, and from a background page that went away mid-session.
public enum RelayLimits {
    /// Largest message in either direction: the host's frame limit
    /// (`websign-protocol` `framing::MAX_INCOMING` and `MAX_OUTGOING`).
    public static let maxMessageBytes = 1 << 20
    /// Host output held for a background that stopped polling. Past it the
    /// session ends: a queue nobody drains must not grow without bound.
    public static let maxPendingBytes = 4 << 20
    public static let maxPendingMessages = 64
    /// Host processes alive at once. One per background lifetime is normal;
    /// the rest covers an extension reload racing the idle close.
    public static let maxSessions = 4
    /// A session nobody touched for this long is ended. The extension polls
    /// every few seconds while a session is open and closes it after 60 s
    /// idle (`EXTENSION_IDLE_CLOSE`), so only an abandoned session (unloaded
    /// background, reloaded extension) gets here.
    public static let idleTimeout: TimeInterval = 120
    /// Longest poll a background may ask for. Safari's own limit on a
    /// pending native message is undocumented; ten seconds stays far inside
    /// what shipping extensions rely on.
    public static let maxPollWait: TimeInterval = 10
    /// Time the host gets to exit on its own after its stdin closes (its
    /// normal shutdown: cancel open requests, close the window) before it is
    /// terminated, and again before it is killed.
    public static let exitGrace: TimeInterval = 3
    /// `websign-protocol` `MAX_REQUEST_ID_LEN`.
    public static let maxRequestIdBytes = 64
}
