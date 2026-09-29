use super::*;

#[test]
fn icp_brasil_certificate_is_summarised() {
    let info = parse(&icp_cert());
    let icp = info.icp_brasil.as_ref().expect("ICP-Brasil");
    assert_eq!(icp.level, Some(IcpLevel::A3));
    assert_eq!(icp.holder_name.as_deref(), Some("ANA BEATRIZ SOUZA"));
    assert_eq!(icp.cpf.as_deref(), Some("12345678901"));
    assert_eq!(icp.cnpj.as_deref(), Some("12345678000195"));
    assert_eq!(icp.masked_cpf().as_deref(), Some("***.456.789-**"));
    assert_eq!(icp.formatted_cnpj().as_deref(), Some("12.345.678/0001-95"));
    assert_eq!(info.display_name(), "ANA BEATRIZ SOUZA");
}

#[test]
fn icp_brasil_is_detected_by_policy_alone_or_by_other_name_alone() {
    let by_policy = parse(&TestCert::new().extension(policies(&["2.16.76.1.2.1.1"])));
    assert_eq!(
        by_policy.icp_brasil.as_ref().and_then(|i| i.level),
        Some(IcpLevel::A1)
    );

    let by_name = parse(
        &TestCert::new().extension(san_other_names(&[("2.16.76.1.3.2", tlv(0x0c, b"NAME"))])),
    );
    let icp = by_name.icp_brasil.expect("ICP-Brasil by otherName");
    assert_eq!(icp.level, None);
    assert_eq!(icp.cpf, None);

    let neither = parse(&TestCert::new().extension(policies(&["2.5.29.32.0"])));
    assert_eq!(neither.icp_brasil, None);
}

#[test]
fn qualified_statements_are_decoded() {
    let types = sequence(&[
        oid("0.4.0.1862.1.6.2"),
        oid("0.4.0.1862.1.6.1"),
        oid("0.4.0.1862.1.6.9"),
    ]);
    let info = parse(&TestCert::new().extension(qc_statements(&[
        ("0.4.0.1862.1.1", None),
        ("0.4.0.1862.1.4", None),
        ("0.4.0.1862.1.6", Some(types)),
        ("0.4.0.1862.1.99", Some(tlv(0x05, &[]))),
    ])));
    let qualified = info.qualified.expect("qualified");
    assert!(qualified.compliance);
    assert!(qualified.sscd);
    assert_eq!(qualified.types, [QcType::ESeal, QcType::ESign]);
}

#[test]
fn an_empty_qc_statement_list_still_marks_the_certificate_as_declaring_them() {
    let info = parse(&TestCert::new().extension(qc_statements(&[])));
    assert_eq!(info.qualified, Some(Qualified::default()));
}
