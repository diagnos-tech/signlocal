use super::*;

#[test]
fn names_read_cn_o_ou_c() {
    let dn = subject_of(&[
        rdn(COUNTRY, PRINTABLE, b"BR"),
        rdn(ORG, UTF8, "ICP-Brasil".as_bytes()),
        rdn(OU, UTF8, b"first"),
        rdn(OU, UTF8, b"second"),
        rdn("2.5.4.7", UTF8, b"ignored locality"),
        rdn(CN, UTF8, "João da Silva".as_bytes()),
    ]);
    assert_eq!(dn.country.as_deref(), Some("BR"));
    assert_eq!(dn.organization.as_deref(), Some("ICP-Brasil"));
    assert_eq!(dn.organizational_units, ["first", "second"]);
    assert_eq!(dn.common_name.as_deref(), Some("João da Silva"));
}

#[test]
fn names_honour_every_string_type() {
    let cn = |tag, bytes: &[u8]| subject_of(&[rdn(CN, tag, bytes)]).common_name;
    assert_eq!(cn(UTF8, "Jão".as_bytes()).as_deref(), Some("Jão"));
    assert_eq!(cn(PRINTABLE, b"Joao").as_deref(), Some("Joao"));
    assert_eq!(
        cn(IA5, b"joao@example.test").as_deref(),
        Some("joao@example.test")
    );
    assert_eq!(
        cn(BMP, &[0x00, 0x4a, 0x00, 0xe3, 0x00, 0x6f]).as_deref(),
        Some("Jão")
    );
    assert_eq!(cn(TELETEX, &[0x4a, 0xe3, 0x6f]).as_deref(), Some("Jão"));
    // NumericString and VisibleString are not in the supported set.
    assert_eq!(cn(0x12, b"123"), None);
    assert_eq!(cn(0x1a, b"visible"), None);
}

#[test]
fn repeated_single_valued_attributes_keep_the_first() {
    let dn = subject_of(&[
        rdn(CN, UTF8, b"first"),
        rdn(CN, UTF8, b"second"),
        rdn(ORG, UTF8, b"org one"),
        rdn(ORG, UTF8, b"org two"),
        rdn(COUNTRY, PRINTABLE, b"BR"),
        rdn(COUNTRY, PRINTABLE, b"PT"),
    ]);
    assert_eq!(dn.common_name.as_deref(), Some("first"));
    assert_eq!(dn.organization.as_deref(), Some("org one"));
    assert_eq!(dn.country.as_deref(), Some("BR"));
}

#[test]
fn issuer_uses_the_same_rules() {
    let info = parse(&TestCert::new().issuer(&[
        rdn(COUNTRY, PRINTABLE, b"BR"),
        rdn(ORG, UTF8, b"AC Raiz"),
        rdn(OU, UTF8, b"Unit"),
        rdn(CN, UTF8, b"AC Intermediaria"),
    ]));
    assert_eq!(info.issuer.common_name.as_deref(), Some("AC Intermediaria"));
    assert_eq!(info.issuer.organization.as_deref(), Some("AC Raiz"));
    assert_eq!(info.issuer.organizational_units, ["Unit"]);
    assert_eq!(info.issuer.country.as_deref(), Some("BR"));
}

#[test]
fn an_empty_subject_is_fine() {
    let dn = subject_of(&[]);
    assert_eq!(dn, DistinguishedName::default());
}

#[test]
fn display_name_falls_back_step_by_step() {
    let holder = parse(&icp_cert());
    assert_eq!(holder.display_name(), "ANA BEATRIZ SOUZA");

    let cn_only =
        parse(&TestCert::new().subject(&[rdn(ORG, UTF8, b"Org"), rdn(CN, UTF8, b"The CN")]));
    assert_eq!(cn_only.display_name(), "The CN");

    let org_only = parse(&TestCert::new().subject(&[rdn(ORG, UTF8, b"Only Org")]));
    assert_eq!(org_only.display_name(), "Only Org");

    let nothing = parse(&TestCert::new().subject(&[]));
    assert_eq!(nothing.display_name(), nothing.fingerprint.to_hex()[..16]);
    assert_eq!(nothing.display_name().len(), 16);
}

#[test]
fn icp_without_a_cn_falls_through_to_the_organization() {
    let cert = TestCert::new()
        .subject(&[rdn(ORG, UTF8, b"Org")])
        .extension(policies(&["2.16.76.1.2.1"]));
    let info = parse(&cert);
    assert_eq!(
        info.icp_brasil.as_ref().and_then(|i| i.holder_name.clone()),
        None
    );
    assert_eq!(info.display_name(), "Org");
}
