# P28 Source Basis — AiDENs v11A Super Pass

**Run id:** P28  
**Created UTC:** 2026-05-06T17:45:40Z  
**Target:** v11A constitutional material-operation kernel for AiDENs, with v11B/v11C reserved-only containment.

## Inputs used

1. `AiDENs-aidens-next-codex-context-20260505.zip` and sidecars.
2. `AiDENs-aidens-next-codex-context-20260505.report.md`.
3. `AiDENs-aidens-next-codex-context-20260505.manifest.json`.
4. `AiDENs-aidens-next-codex-context-20260505.findings.json`.
5. `AiDENs-aidens-next-codex-context-20260505.excluded.json`.
6. `AiDENs-aidens-next-codex-context-20260505.codex-archive.json`.
7. `P27_STATUS_EVIDENCE_MANIFEST.json`.
8. Claude P27 hard audit from `Pasted text.txt`.
9. v11 spec corpus:
   - `CANONICAL_STACK_SPEC_V11A_CONSTITUTIONAL_ARTIFACT_RUNTIME_CORE.md`
   - `CANONICAL_STACK_SPEC_V11B_RECURSIVE_SUBTRACTIVE_REGIONAL_RUNTIME.md`
   - `CANONICAL_STACK_SPEC_V11C_SELF_HOSTING_FEDERATED_MECHANISM_AND_AGENCY_RUNTIME.md`
   - `V11_PLUS_ARTIFACT_FAMILY_INDEX.md`
   - `V11_PLUS_CONFORMANCE_AND_RELEASE_BAR.md`
10. Supporting research corpus in `Full Provenance+ Research 4⁄26⁄26.zip` and the extracted research markdowns.

## P27 factual baseline

P27 is a credible **supported-local** verification pass, not a v11 release. P27 evidence reports strict packaging and no zip validation findings. The P27 evidence manifest records green upstream `cargo fmt`, `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo doc`, strict verifier, strict package validation, and full package self-replay. P27 also explicitly preserves the non-claim that AiDENs is not production-cloud-ready, not broad-autonomy-ready, and not V10/V11/V12-complete.

## P28 posture

P28 MUST NOT attempt a broad feature sprint. P28 is a constitutional hardening pass:

```text
Artifact(s)
+ OperatorContract
+ ExecutionContext
+ Permit/Policy
+ Budget
→ Artifact(s)
+ Receipt(s)
+ Proof/Refutation/Degradation state
```

The target is **v11A-conformant-core on declared local production paths**, plus **v11C-reserved** compatibility hooks. v11B region/subtraction surfaces remain reserved/draft unless the v11A gates are complete.
