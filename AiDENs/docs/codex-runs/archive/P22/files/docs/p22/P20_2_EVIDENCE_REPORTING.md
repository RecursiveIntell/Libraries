# P20.2 Evidence and Reporting Requirements

Every phase report must include:

- phase ID and title;
- commands run;
- files changed;
- tests/checks passed;
- invariant validation result;
- unresolved risks;
- whether human injection prompt was applied;
- PASS/FAIL.

Final audit bundle must include:

```text
cargo-version.txt
cargo-metadata.json
cargo-tree.txt
fmt.log
check.log
test.log
clippy.log
p20_2_verify.log
package-integrity.json
testkit-purity.json
agency-eval-validation.log
test-agent-report.md
release-zip-recheck.log
known-limitations.md
final-auditor-handoff.md
```
