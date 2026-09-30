//! The device a key lives on, named without personal data.

use websign_devices::hints::DeviceDatabase;
use websign_keystores::DeviceLink;
use websign_ui_model::certs::DeviceLabel;

/// A name that is safe to show: the model from `devices.json` when the token
/// is known, else the model the token itself reports (never its label, which
/// often holds the holder's name).
pub(super) fn device_label(
    link: Option<&DeviceLink>,
    hints: Option<&DeviceDatabase>,
) -> Option<DeviceLabel> {
    match link? {
        DeviceLink::Reader { name } => Some(DeviceLabel::CardInReader {
            reader: name.clone(),
        }),
        DeviceLink::CryptoTokenKit { .. } => Some(DeviceLabel::Unknown),
        DeviceLink::Pkcs11Token {
            model,
            manufacturer,
        } => {
            let model = model.trim();
            if model.is_empty() {
                return Some(DeviceLabel::Unknown);
            }
            let known = hints.and_then(|database| {
                database
                    .devices
                    .iter()
                    .find(|hint| hint.name.to_lowercase().contains(&model.to_lowercase()))
            });
            let name = match known {
                Some(hint) => hint.name.clone(),
                None => format!("{} {model}", manufacturer.trim()).trim().to_owned(),
            };
            Some(DeviceLabel::Token { name })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(model: &str, manufacturer: &str) -> DeviceLink {
        DeviceLink::Pkcs11Token {
            model: model.into(),
            manufacturer: manufacturer.into(),
        }
    }

    #[test]
    fn readers_and_unknown_tokens_get_safe_labels() {
        let reader = DeviceLink::Reader {
            name: "Reader 0".into(),
        };
        assert_eq!(
            device_label(Some(&reader), None),
            Some(DeviceLabel::CardInReader {
                reader: "Reader 0".into()
            })
        );
        assert_eq!(device_label(None, None), None);
        assert_eq!(
            device_label(Some(&token("", "Acme")), None),
            Some(DeviceLabel::Unknown)
        );
    }

    #[test]
    fn a_token_falls_back_to_the_model_it_reports() {
        assert_eq!(
            device_label(Some(&token("eToken 5110", "SafeNet")), None),
            Some(DeviceLabel::Token {
                name: "SafeNet eToken 5110".into()
            })
        );
    }

    #[test]
    fn a_known_token_uses_the_name_from_devices_json() {
        let database: DeviceDatabase = serde_json::from_str(
            r#"{"version":1,"devices":[{"id":"x","name":"SafeNet eToken 5110","kind":"token","match":{"usb":[],"atr":[]}}]}"#,
        )
        .unwrap();
        assert_eq!(
            device_label(Some(&token("eToken 5110", "SafeNet Inc.")), Some(&database)),
            Some(DeviceLabel::Token {
                name: "SafeNet eToken 5110".into()
            })
        );
    }
}
