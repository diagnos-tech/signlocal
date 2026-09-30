//! Whether a certificate on a token has a private key, decided without a
//! login (the rule is explained in `listing`).

use websign_core::CertInfo;

use super::provider::KeyVisibility;

/// The three-way rule of the `listing` module documentation.
pub fn visibility(
    login_required: bool,
    key_ids: &[Vec<u8>],
    certificate_id: &[u8],
) -> KeyVisibility {
    if !key_ids.is_empty() {
        return if key_ids.iter().any(|id| id == certificate_id) {
            KeyVisibility::Visible
        } else {
            KeyVisibility::Absent
        };
    }
    if login_required {
        KeyVisibility::AfterLogin
    } else {
        KeyVisibility::Absent
    }
}

/// Whether a certificate is reported: it has a key, or may have a hidden one
/// and is not a CA certificate.
pub fn counts(keys: KeyVisibility, der: &[u8]) -> bool {
    match keys {
        KeyVisibility::Visible => true,
        KeyVisibility::AfterLogin => !CertInfo::from_der(der).is_ok_and(|info| info.is_ca),
        KeyVisibility::Absent => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&[u8]]) -> Vec<Vec<u8>> {
        list.iter().map(|id| id.to_vec()).collect()
    }

    #[test]
    fn a_visible_key_with_the_same_id_makes_the_certificate_count() {
        let keys = ids(&[&[1], &[2]]);
        assert_eq!(visibility(true, &keys, &[2]), KeyVisibility::Visible);
        assert_eq!(visibility(false, &keys, &[2]), KeyVisibility::Visible);
    }

    #[test]
    fn once_keys_are_visible_certificates_without_one_are_left_out() {
        // CA certificates stored next to the user's certificate on the token.
        let keys = ids(&[&[1]]);
        assert_eq!(visibility(true, &keys, &[9]), KeyVisibility::Absent);
        assert_eq!(visibility(true, &keys, &[]), KeyVisibility::Absent);
    }

    #[test]
    fn hidden_keys_are_assumed_when_a_login_is_required() {
        assert_eq!(visibility(true, &[], &[7]), KeyVisibility::AfterLogin);
    }

    #[test]
    fn no_keys_and_no_login_means_no_key() {
        assert_eq!(visibility(false, &[], &[7]), KeyVisibility::Absent);
    }
}
