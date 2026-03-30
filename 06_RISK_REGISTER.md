# Risk Register — V29

## Risk 1: WIRE-001 breaks existing serialized data
**Likelihood:** LOW (no production data persisted yet)
**Impact:** HIGH (deserialization failures on any persisted data)
**Mitigation:** Confirm no SQLite databases or JSON artifacts contain PascalCase enum variants from the 56 affected types. If any exist, add `#[serde(alias = "PascalCase")]` for backward compatibility.

## Risk 2: DOC-001 doc pass introduces incorrect documentation
**Likelihood:** MEDIUM (AI-assisted bulk doc generation)
**Impact:** LOW (incorrect docs are better than no docs for submission, and are easily corrected)
**Mitigation:** Focus doc comments on type-level purpose and contract, not implementation details. Review generated docs for accuracy on the 10 most critical types.

## Risk 3: TRUTH-002 archive moves break import paths or script references
**Likelihood:** MEDIUM (scripts may hardcode paths to docs being moved)
**Impact:** MEDIUM (gate failures)
**Mitigation:** Before moving files, grep the entire repo for references to each file being moved. Update any references found. Run full gate after archive operation.

## Risk 4: Time pressure causes incomplete Phase 2
**Likelihood:** MEDIUM (DOC-001 alone is a multi-hour task)
**Impact:** MEDIUM (submission proceeds with partial doc coverage)
**Mitigation:** Phase 1 is the hard gate. Phase 2 WIRE-001 is the second priority. DOC-001 can be scoped to the 5 most critical crates if time is short.

## Risk 5: CONV-001 HashMap→BTreeMap conversion changes behavior
**Likelihood:** LOW (BTreeMap is a drop-in with different ordering)
**Impact:** LOW (iteration order changes, but code should not depend on HashMap iteration order)
**Mitigation:** Run full test suite after each conversion. The HNSW HashMap is explicitly excluded from conversion.

---

## Forbidden Shortcuts

1. **Do not weaken gate scripts to pass.** If a gate fails, fix the underlying issue.
2. **Do not add `#[allow(dead_code)]` to suppress warnings.** Fix or remove dead code.
3. **Do not bulk-add `rename_all` without running `cargo test`.** Serde annotation changes can break deserialization of test fixtures.
4. **Do not move files to archive without checking for references.** A moved file that's still referenced by a script creates a new gate failure.
5. **Do not combine doc generation with code changes.** Doc-only commits are reviewable. Mixed commits hide code changes behind doc noise.
6. **Do not create new schema versions.** This pack closes gaps, it does not extend scope.
7. **Do not invent compatibility shims for the HashMap→BTreeMap conversion.** If code depends on HashMap iteration order, that's a bug — fix it.
8. **Do not edit CANONICAL_STACK_SPEC_V25 or V26.** These are constitutional documents. If they contain errors, file a separate issue.
