//! Linux install locations. Sources: `docs/research/pkcs11-modules.md`.

use super::{KnownModule, module};

#[rustfmt::skip]
pub const MODULES: &[KnownModule] = &[
    module("Thales/SafeNet", "SafeNet Authentication Client (eToken 5100/5110/5300)", &[
        "/usr/lib/libeToken.so",
        "/usr/lib64/libeToken.so",
        "/usr/lib/libeTPkcs11.so",
        "/usr/lib64/libeTPkcs11.so",
        "/usr/local/lib/libeTPkcs11.so",
    ]),
    module("A.E.T./G&D", "SafeSign Identity Client (StarSign token)", &[
        "/usr/lib/libaetpkss.so",
        "/usr/lib/libaetpkss.so.3",
        "/usr/lib64/libaetpkss.so",
    ]),
    module("Watchdata", "Watchdata ProxKey (ICP-Brasil)", &[
        "/usr/lib/watchdata/ICP/lib/libwdpkcs_icp.so",
        "/usr/lib/watchdata/lib/libwdpkcs.so",
        "/opt/watchdata/lib64/libwdpkcs.so",
        "/usr/local/lib64/libwdpkcs.so",
        "/usr/local/lib/libwdpkcs.so",
        "/usr/lib/libwdpkcs.so",
        "/usr/lib/WatchData/ProxKey/lib/libwdpkcs_SignatureP11.so",
    ]),
    module("Feitian", "ePass2003", &[
        "/opt/ePass2003-Castle-*/x64/redist/libcastle.so.1.0.0",
        "/usr/lib/libcastle.so.1.0.0",
        "/usr/lib/libcastle_v2.so.1.0.0",
    ]),
    module("Feitian", "ePassNG (ePass2000/3000)", &[
        "/usr/lib/libepsng_p11.so",
        "/usr/local/ngsrv/libepsng_p11.so.1",
    ]),
    module("OpenSC", "OpenSC", &[
        "/usr/lib/x86_64-linux-gnu/pkcs11/opensc-pkcs11.so",
        "/usr/lib/aarch64-linux-gnu/pkcs11/opensc-pkcs11.so",
        "/usr/lib64/pkcs11/opensc-pkcs11.so",
        "/usr/lib/pkcs11/opensc-pkcs11.so",
        "/usr/lib64/opensc-pkcs11.so",
        "/usr/lib/opensc-pkcs11.so",
        "/usr/local/lib/opensc-pkcs11.so",
    ]),
    module("AMA (Portugal)", "Cartao de Cidadao", &[
        "/usr/local/lib/libpteidpkcs11.so",
        "/usr/lib/libpteidpkcs11.so",
    ]),
    module("Policia Nacional (Spain)", "DNIe", &[
        "/usr/lib/libpkcs11-dnie.so",
        "/usr/lib64/libpkcs11-dnie.so",
        "/opt/FNMTpkcs11dnie/lib/libpkcs11-dnie.so",
    ]),
    module("Thales/Gemalto", "IDPrime (IDGo 800 / SAC)", &[
        "/usr/lib/libIDPrimePKCS11.so",
        "/usr/lib64/libIDPrimePKCS11.so",
        "/usr/lib/pkcs11/libIDPrimePKCS11.so",
    ]),
    module("Thales/Gemalto", "Classic Client (GemSafe)", &[
        "/usr/lib/pkcs11/libgclib.so",
        "/usr/lib/ClassicClient/libgclib.so",
    ]),
    module("Athena", "IDProtect", &[
        "/usr/lib/x64-athena/libASEP11.so",
        "/usr/lib/libASEP11.so",
        "/lib64/libASEP11.so",
    ]),
    module("Bit4id", "Universal Middleware", &[
        "/usr/lib/libbit4ipki.so",
        "/usr/lib/libbit4xpki.so",
    ]),
    module("Yubico", "YubiKey PIV (ykcs11)", &["/usr/local/lib/libykcs11.so"]),
    module("Oberthur/Cosmo", "Certisign Cosmo (AWP)", &["/usr/local/AWP/lib/libOcsCryptoki.so"]),
    module("SafeWeb", "SafeWeb/CIS token", &["/usr/lib/libcmP11.so"]),
    module("SoftHSM project", "SoftHSM2", &[
        "/usr/lib/softhsm/libsofthsm2.so",
        "/usr/lib/x86_64-linux-gnu/softhsm/libsofthsm2.so",
        "/usr/lib/aarch64-linux-gnu/softhsm/libsofthsm2.so",
        "/usr/lib64/softhsm/libsofthsm2.so",
        "/usr/local/lib/softhsm/libsofthsm2.so",
    ]),
];
