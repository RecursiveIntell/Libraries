# PHASE 04 — Digest and Artifact Identity Law Collapse

## Objective

Remove AiDENs-local canonical digest/content-addressing law.

## Required actions

1. Find all exports/usages of:
   - `stable_json_digest`
   - `stable_text_digest`
   - `deterministic_artifact_id`
   - `canonical_json_string`
   - `CanonicalDigestV1`
2. Determine whether each is:
   - canonical artifact identity/digest semantics: must move to `stack-ids`;
   - display-only/report digest: must be renamed and documented as non-authoritative;
   - test helper: must be private/test-only and not exported.
3. Replace canonical uses with `stack-ids` primitives.
4. Ensure no display digest can be used as artifact identity.

## Required gate

```bash
bash scripts/assert_no_local_canonical_digest_law.sh
```

## Acceptance

- AiDENs does not export local canonical digest/ID law.
- Canonical digest/ID semantics come from `stack-ids`.
- Display-only digests are visibly non-authoritative.
- Any ambiguous digest is quarantined.

## Stop

Stop after this phase and wait for `GUARDRAIL_04_TO_05`.
