//! macOS install locations. Sources: `docs/research/pkcs11-modules.md`.

use super::{KnownModule, module};

#[rustfmt::skip]
pub const MODULES: &[KnownModule] = &[
    module("Thales/SafeNet", "SafeNet Authentication Client", &[
        "/usr/local/lib/libeTPkcs11.dylib",
        "/Library/Frameworks/eToken.framework/Versions/A/libeToken.dylib",
    ]),
    module("A.E.T./G&D", "SafeSign Identity Client (StarSign token)", &[
        "/usr/local/lib/libaetpkss.dylib",
        "/Applications/tokenadmin.app/Contents/Frameworks/libaetpkss.dylib",
    ]),
    module("Watchdata", "Watchdata ProxKey", &[
        "/usr/local/lib/libwdpkcs.dylib",
        "/usr/lib/libwdpkcs.dylib",
        "/Applications/WatchKey USB Token Admin Tool.app/Contents/MacOS/lib/libWDP11_BR_GOV.dylib",
    ]),
    module("Feitian", "ePass2003", &["/usr/local/lib/libcastle.1.0.0.dylib"]),
    module("OpenSC", "OpenSC", &[
        "/Library/OpenSC/lib/opensc-pkcs11.so",
        "/usr/local/lib/opensc-pkcs11.so",
        "/opt/homebrew/lib/opensc-pkcs11.so",
        "/opt/homebrew/lib/pkcs11/opensc-pkcs11.so",
    ]),
    module("AMA (Portugal)", "Cartao de Cidadao", &["/usr/local/lib/libpteidpkcs11.dylib"]),
    module("Policia Nacional (Spain)", "DNIe", &["/Library/Libpkcs11-dnie/lib/libpkcs11-dnie.so"]),
    module("Athena", "IDProtect", &[
        "/Library/Application Support/Athena/libASEP11.dylib",
        "/usr/local/lib/libASEP11.dylib",
    ]),
    module("Bit4id", "Universal Middleware", &[
        "/Library/bit4id/pkcs11/libbit4ipki.dylib",
        "/Library/bit4id/pkcs11/libbit4xpki.dylib",
    ]),
    module("Yubico", "YubiKey PIV (ykcs11)", &[
        "/usr/local/lib/libykcs11.dylib",
        "/opt/homebrew/lib/libykcs11.dylib",
    ]),
    module("SoftHSM project", "SoftHSM2", &[
        "/opt/homebrew/lib/softhsm/libsofthsm2.so",
        "/usr/local/lib/softhsm/libsofthsm2.so",
    ]),
];
