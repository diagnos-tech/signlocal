//! The audit token of the process at the other end of stdin, when stdin is
//! a connected Unix socket (`LOCAL_PEERTOKEN`). Pipes have no peer: `None`.

/// A kernel audit token (`audit_token_t`): eight 32-bit words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditToken(pub [u32; 8]);

impl AuditToken {
    /// The process ID (`audit_token_to_pid`: word 5).
    pub fn pid(&self) -> i32 {
        self.0[5] as i32
    }

    /// The token's bytes, as `kSecGuestAttributeAudit` takes them.
    pub fn bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|word| word.to_ne_bytes()).collect()
    }
}

/// The peer's token for file descriptor 0.
pub fn stdin_audit_token() -> Option<AuditToken> {
    let mut words = [0u32; 8];
    let mut len = size_of_val(&words) as libc::socklen_t;
    // SAFETY: `words` is writable for `len` bytes and `len` is writable;
    // on anything but a Unix socket the call fails without writing.
    let status = unsafe {
        libc::getsockopt(
            libc::STDIN_FILENO,
            libc::SOL_LOCAL,
            libc::LOCAL_PEERTOKEN,
            words.as_mut_ptr().cast(),
            &mut len,
        )
    };
    (status == 0 && len as usize == size_of_val(&words)).then_some(AuditToken(words))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pid_is_word_five() {
        let token = AuditToken([0, 0, 0, 0, 0, 4242, 0, 7]);
        assert_eq!(token.pid(), 4242);
        assert_eq!(token.bytes().len(), 32);
    }
}
