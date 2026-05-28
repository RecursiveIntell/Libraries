#[test]
fn generated_reference_cases_match_the_reference_interpreter() {
    let cases = aidens_testkit::reference_cases();
    let report = aidens_testkit::evaluate_reference_cases(&cases);

    assert!(
        report.findings.is_empty(),
        "reference self-check findings: {:?}",
        report.findings
    );
    assert_eq!(report.case_count, cases.len());
}
