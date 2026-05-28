# Digest Identity Source Of Truth

SOURCE BASIS: 2026-04-28

Canonical digest and content-addressing law is owned by `~/Coding/Libraries/stack-ids`.

AiDENs may use:

- `stack_ids::ContentDigest` and `stack_ids::DigestBuilder` for stack-owned digest computation;
- `stack_ids::ArtifactId` and sibling opaque ID types for typed identifiers;
- `DisplayDigestV1` only as a non-authoritative report/display wrapper.

AiDENs must not export local canonical digest or ID law. The removed local law names are:

- `stable_json_digest`
- `stable_text_digest`
- `deterministic_artifact_id`
- `canonical_json_string`
- `CanonicalDigestV1`

Current AiDENs display digest rule:

- digest bytes are computed by `stack_ids::ContentDigest`;
- serialized display strings use a `blake3:` prefix for operator readability;
- `DisplayDigestV1.non_authoritative` is always true for constructors;
- `DisplayDigestV1` is not an artifact identity type and must not be used as a canonical stack content address.

AiDENs-local idempotency IDs for queue, schedule, wake, and report wrappers may derive local `ArtifactId` strings from `stack_ids::ContentDigest`, but they do not define stack-wide artifact identity law.
