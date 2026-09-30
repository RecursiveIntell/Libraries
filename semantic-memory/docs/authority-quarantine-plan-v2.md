# Authority quarantine proposal V2: read-view evidence

This unsigned proposal remains advisory. `application_authorization_required`
remains true, even for an empty result. There is no apply, restore, delete,
enrollment, transport, or permission-granting API in the planner.

## Why V1 is replaced before merge

V1 independently reopened the configured pathname to hash before and after
inspecting an already-open pooled SQLite connection. Both file hashes could
agree while the retained connection read a different file after pathname
replacement, or after changing cwd with a relative storage path. The result
could contain rows from A with `database_sha256` of B.

V2 intentionally removes `database_sha256` and replaces
`AuthorityRelationQuarantinePlanV1` with `AuthorityRelationQuarantinePlanV2`.
There is no compatibility alias and no claim that a V2 digest is an original
file hash. Discard V1 proposals whose image identity was not independently
proved and regenerate them. No live data migration is involved.

## Exact evidence contract

- `schema_version`: `authority_relation_quarantine_plan_v2`
- `read_view_format`: `sqlite3_backup_then_serialize_v1`
- `read_view_sha256`: SHA-256 of the exact SQLite serialization consumed by the
  planner's private read-only deserialized connection
- `read_view_size_bytes`: length of those serialized bytes
- Schema, epochs, violations, lossless cells, rows and their digests all come
  from that same private image; the final plan hash also covers its evidence
  fields

The canonical `MemoryStore` reader pool remains the sole source owner. A real
schema query pins one acquired connection's read transaction. Source DELETE
journal admission and checked page-count × page-size budgeting occur in that
transaction. The canonical integrity snapshot backup helper copies that view
into a disposable in-memory image while retaining the source transaction.
Busy/locked, incomplete, deadline and SQLite backup errors fail closed. The
planner never opens the configured pathname for hashing.

The private image is serialized in memory and those exact owned bytes are
transferred into a read-only SQLite deserialization for inspection. This
avoids relying on direct disk serialization: the bundled SQLite serializer
can zero-fill a page when a disk page read fails, whereas the backup API
propagates page-read failures. There is no external staging file, publication,
second truth store or retained image; the copy is disposable evidence owned
by `semantic-memory`.

The source guard is DELETE journaling; the private in-memory connection may
report memory journaling. These are deliberately separate. Exact schema 39
including DDL, supported orphan topology, and all original row/cell/output
checks remain mandatory. The source bytes, journal mode, schema, epochs and
vector sidecars are never changed by planning.

## Budgets and limits

The source view's checked `page_count * page_size` must be positive and at most
256 MiB before backup allocates its private image. Overflow and invalid or
oversized sizes return `ReadViewLimitExceeded`. The copied image is measured
and checked again before the serializer's second full-image allocation, and
the actual serialization length must match. A size or ownership mismatch
returns `InvalidReadView`; SQLite allocation/deserialize failures return a
typed SQLite error. Peak memory can include both the backup and serialized
images plus SQLite overhead; 256 MiB is an image-size cap, not a process-RSS cap.

Independent existing caps remain 1–10,000 violations/rows and 1–16 MiB encoded
proposal JSON, with caller-specified lower budgets. The canonical backup uses
256-page steps, a source-page-derived finite step bound and a cooperative
30-second deadline. A blocked OS call is not forcibly interrupted. There is
no retry, allocation fallback or unsupported-schema fallback.

A V2 digest binds an inspected derived image, not a file pathname, current
filesystem occupant, device/inode, original byte-for-byte disk file, WAL or
sidecar set. Backup/serialization can change SQLite header details, so digest
identity is not promised across SQLite versions or arbitrary recaptures. The
planner does not certify general corruption freedom, repairability, or
applicability to a later live image. It relies on ordinary SQLite read/backup
semantics and does not defend against hostile same-UID memory/filesystem
corruption outside SQLite locking. Any future application facility requires
its own owner/authority contract and current-state revalidation.

## Gates and rollback

`integrity_read_view_binding` uses the actual public MemoryStore constructors
for pathname replacement and isolated-process cwd drift. Whole-plan equality
assertions are behavioral red on V1 and green on V2, with both source images
checked unchanged. Original planner refusal/lossless/repeatability tests,
nondefault page-size and image-budget tests, integrity snapshot pin/failure
coverage, diagnostics, and admin-delete guard tests must remain green.

Keep the PR draft and unmerged until exact-head CI and independent post-review
pass. Rollback is to discard this unmerged delta or revert its exact source
change; no live store recovery is needed. Do not resume using unverified V1
proposals on rollback.
