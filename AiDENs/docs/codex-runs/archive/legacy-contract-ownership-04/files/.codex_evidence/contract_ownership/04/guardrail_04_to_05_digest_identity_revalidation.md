# Guardrail 04 To 05 - Digest And Identity Law Revalidation

Date: 2026-04-29
Working directory: `/home/sikmindz/Coding/Libraries/AiDENs`

## 1. Removed AiDENs Canonical Digest Exports

Command:

```bash
rg -n "pub fn stable_json_digest|pub fn stable_text_digest|pub fn deterministic_artifact_id|pub fn canonical_json_string|pub struct CanonicalDigestV1|pub enum CanonicalDigestV1|pub type CanonicalDigestV1" crates/aidens-contracts/src/lib.rs
```

Result: no matches, exit status 1 from `rg` because the requested exported definitions are absent.

## 2. Canonical Digest And ID Owner

Canonical owner evidence:

- `../stack-ids/src/digest.rs` defines `ContentDigest`, `DigestBuilder`, `ContentDigest::compute_str`, and `ContentDigest::compute_json`.
- `crates/aidens-contracts/src/lib.rs` imports/re-exports stack-owned ID and digest types from `stack_ids`.
- `crates/aidens-contracts/src/lib.rs` delegates JSON digesting through `canonical_stack::digest_json`, which calls `ContentDigest::compute_json`.
- `crates/aidens-contracts/src/lib.rs` delegates text digesting through `StackContentDigest::compute_str`.
- `docs/contract-ownership/DIGEST_IDENTITY_SOURCE_OF_TRUTH.md` records `~/Coding/Libraries/stack-ids` as the canonical digest/content-addressing owner.

## 3. Display-Only Digest Surfaces

Display-only evidence:

- `CanonicalDigestV1` has been replaced by `DisplayDigestV1`.
- Public helpers are named `non_authoritative_json_display_digest` and `non_authoritative_text_display_digest`.
- `DisplayDigestV1` has `non_authoritative: true`.
- Constructors add `display-only-not-artifact-identity`.
- `docs/contract-ownership/DIGEST_IDENTITY_SOURCE_OF_TRUTH.md` states `DisplayDigestV1` is not an artifact identity type and must not be used as a canonical stack content address.

## 4. Artifact Identity Path Check

Identity path scan:

```bash
rg -n "local_artifact_id_from_stack_digest\\(|generated_artifact_id\\(|ArtifactId::new\\(|StackContentDigest::compute_str\\(" crates/aidens-contracts/src/lib.rs
```

Observed identity constructors:

- `generated_artifact_id(prefix)` creates stack-owned `ArtifactId` values with `Uuid::new_v4()` for local app/report IDs.
- `local_artifact_id_from_stack_digest(prefix, material)` is private and derives local wrapper/idempotency IDs through `StackContentDigest::compute_str(material)`, not through AiDENs-local digest law.
- No path references `stable_json_digest`, `stable_text_digest`, `deterministic_artifact_id`, `canonical_json_string`, `CanonicalDigestV1`, `sha2`, or `Sha256` in `crates/aidens-contracts`.

Conclusion: no artifact identity path uses AiDENs-local digest law.

## 5. Digest Law Gate

Command:

```bash
bash scripts/assert_no_local_canonical_digest_law.sh
```

Output:

```text
PASS: no exported local canonical digest law detected.
```

## Halt Decision

AiDENs does not currently own canonical digest/identity semantics. Phase 05 may start only after the human guardrail is accepted.
