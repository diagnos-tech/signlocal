//! Windows install locations. Sources: `docs/research/pkcs11-modules.md`.
//!
//! A 64-bit process can only load 64-bit modules, and 32-bit middleware lives
//! in `SysWOW64`, which is not listed on purpose.

use super::{KnownModule, module};

#[rustfmt::skip]
pub const MODULES: &[KnownModule] = &[
    module("Thales/SafeNet", "SafeNet Authentication Client (eToken)", &[
        r"%SystemRoot%\System32\eTPKCS11.dll",
    ]),
    module("Thales/Gemalto", "IDPrime (IDGo 800 / SAC)", &[
        r"%ProgramFiles%\SafeNet\Authentication\SAC\x64\IDPrimePKCS1164.dll",
        r"%ProgramFiles%\Gemalto\IDGo 800 PKCS#11\IDPrimePKCS1164.dll",
        r"%ProgramFiles(x86)%\Gemalto\IDGo 800 PKCS#11\IDPrimePKCS1164.dll",
    ]),
    module("A.E.T./G&D", "SafeSign Identity Client (StarSign token)", &[
        r"%SystemRoot%\System32\aetpkss1.dll",
    ]),
    module("Watchdata", "Watchdata ProxKey (ICP-Brasil)", &[
        r"%SystemRoot%\System32\Watchdata\Watchdata ICP CSP v1.0\WDPKCS.dll",
        r"%SystemRoot%\System32\Watchdata\Watchdata Brazil CSP v1.0\WDPKCS.dll",
        r"%SystemRoot%\System32\WDPKCS.dll",
        r"%SystemRoot%\System32\WDICP_P11_CCID_v34.dll",
        r"%SystemRoot%\System32\SignatureP11.dll",
    ]),
    module("Feitian", "ePass2003", &[r"%SystemRoot%\System32\eps2003csp11.dll"]),
    module("Feitian", "ePass3003", &[r"%SystemRoot%\System32\ShuttleCsp11_3003.dll"]),
    module("Feitian", "ePassNG (ePass2000/3000)", &[r"%SystemRoot%\System32\ngp11v211.dll"]),
    module("OpenSC", "OpenSC", &[
        r"%ProgramFiles%\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll",
    ]),
    module("AMA (Portugal)", "Cartao de Cidadao", &[r"%SystemRoot%\System32\pteidpkcs11.dll"]),
    module("Policia Nacional (Spain)", "DNIe", &[
        r"%SystemRoot%\System32\DNIe_P11_x64.dll",
        r"%SystemRoot%\System32\DNIe_P11.dll",
        r"%SystemRoot%\System32\DNIe_P11_priv.dll",
    ]),
    module("Thales/Gemalto", "Classic Client (GemSafe)", &[
        r"%SystemRoot%\System32\gclib.dll",
        r"%ProgramFiles%\Gemplus\GemSafe Libraries\BIN\gclib.dll",
    ]),
    module("Athena", "IDProtect", &[r"%SystemRoot%\System32\asepkcs.dll"]),
    module("Bit4id", "Universal Middleware", &[
        r"%SystemRoot%\System32\bit4ipki.dll",
        r"%SystemRoot%\System32\bit4xpki.dll",
        r"%SystemRoot%\System32\bit4opki.dll",
    ]),
    module("Oberthur/Cosmo", "Certisign Cosmo (OcsCryptoki)", &[
        r"%SystemRoot%\System32\OcsCryptoki.dll",
    ]),
    module("Yubico", "YubiKey PIV (ykcs11)", &[
        r"%ProgramFiles%\Yubico\Yubico PIV Tool\bin\libykcs11.dll",
    ]),
    module("SafeWeb", "SafeWeb/CIS token", &[r"%SystemRoot%\System32\cmP11.dll"]),
    module("SoftHSM project", "SoftHSM2", &[r"C:\SoftHSM2\lib\softhsm2-x64.dll"]),
];
