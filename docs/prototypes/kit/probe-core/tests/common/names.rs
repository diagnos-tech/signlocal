//! Distinguished names the fixtures are issued with (`gen/lib.sh`).

use probe_core::DistinguishedName;

pub fn dn(
    cn: Option<&str>,
    org: Option<&str>,
    units: &[&str],
    country: Option<&str>,
) -> DistinguishedName {
    DistinguishedName {
        common_name: cn.map(str::to_owned),
        organization: org.map(str::to_owned),
        organizational_units: units.iter().map(|u| (*u).to_owned()).collect(),
        country: country.map(str::to_owned),
    }
}

/// Subject of every ordinary fixture (`generate.sh`, `leaf_subject`).
pub fn leaf_dn(name: &str) -> DistinguishedName {
    dn(
        Some(&format!("Fixture {name}")),
        Some("signlocal Test Fixtures"),
        &["Unit A", "Unit B"],
        Some("BR"),
    )
}

pub fn root_dn() -> DistinguishedName {
    dn(
        Some("signlocal Test Root CA"),
        Some("signlocal Test Authority"),
        &["Fixtures Root"],
        Some("BR"),
    )
}
