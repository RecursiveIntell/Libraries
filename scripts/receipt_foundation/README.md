# Receipt observation foundation 0.2

Offline, create-only historical evidence tooling. The archive is the source;
SQLite is a disposable observation projection. No native receipt, signature,
authority, fact, or successful task outcome is manufactured by this tool.

## Recommended operator path

Use Linux, Python 3.11 or newer, SQLite 3.38 or newer, the `zstd` executable,
and the validation dependencies in `requirements-validation.txt`. The tested
versions are recorded by `doctor` and the certification receipt. Package
installation is never performed by this utility. Use a private, operator-owned
workspace and an isolated environment when installing dependencies.

```bash
cd scripts/receipt_foundation
# ARCHIVE, PRIVATE_PARENT and ARCHIVE_SHA256 are explicit operator-selected inputs.
python -m receipt_foundation doctor "$ARCHIVE" \
  --output-parent "$PRIVATE_PARENT" --expect-sha256 "$ARCHIVE_SHA256"
python -m unittest discover -s tests -v
python certify_local.py "$ARCHIVE" \
  --work-dir "$PRIVATE_PARENT/new-certification" --expect-sha256 "$ARCHIVE_SHA256"
```

`certify_local.py` creates a NEW private directory, runs preflight and tests,
publishes a complete bundle, retains its operational witnesses, deletes only
that newly created derived bundle, rebuilds in another process with a different
hash seed, compares every logical table, validates contracts, and exports the
private corpus audit. A nonzero exit is failure; read `LOCAL_CERTIFICATION.json`.
A successful local certification is not native adoption or production activation.
The audit step targets the supplied Ares collection family, not every possible
foreign archive; absence of its required witnesses fails the step.

## Bundle publication and consumption

```bash
python -m receipt_foundation bundle-build "$ARCHIVE" \
  --destination "$PRIVATE_PARENT/new-bundle" \
  --expect-sha256 "$ARCHIVE_SHA256" --recover-json-streams
python -m receipt_foundation bundle-verify "$PRIVATE_PARENT/new-bundle" \
  --expect-sha256 "$ARCHIVE_SHA256"
python -m receipt_foundation bundle-query "$PRIVATE_PARENT/new-bundle" \
  --expect-sha256 "$ARCHIVE_SHA256" --kind family --value AresProfilePanelReceiptV2
```

A bundle contains exactly `projection.sqlite`, `build.json`, `contracts.json`,
`query-proof.json`, and `bundle.json`. All files are private. Publication uses
Linux `renameat2(RENAME_NOREPLACE)` and fsync of staged files/directories and the
parent. There is no overwrite-capable fallback. Unsupported kernels/filesystems
fail explicitly. The final directory name does not expose a partially populated
bundle. A post-rename fsync error is **durability unknown**, not success and not
permission to overwrite the existing bundle.

Every bundle query checks the exact file set, hashes, permissions, implementation
binding, expected archive identity, SQLite schema/integrity, logical manifest,
summary and query witnesses. It uses the same descriptor-pinned database that
was checked. It does not treat a manifest as a signature or rehash the original
archive during each query. Rebuild/certification is the proof against original
source bytes. A user controlling the same operating-system account can forge an
unsigned manifest; this is not an authorization or anti-root security boundary.

Failures may leave only a uniquely named `.receipt-bundle-staging-*` directory
with a safe failure record. It is never an admitted bundle. Retain or explicitly
remove only the identified derived stage after inspection; no automatic cleanup
of unrelated or historical state occurs.

## Contract change from 0.1

Projection `user_version` is **2**, rules are `receipt-observation-rules/2`.
Envelope and edge schemas are V2. Old projections are rejected, not migrated in
place or parsed through a compatibility shim. Rebuild from original evidence.
Blob, occurrence, anomaly, and ingestion-request contracts retain V1 where their
shape is unchanged. The generated JSON schemas come only from `contracts.py`.

Session versus episode, task versus goal, actor versus agent ID, tool versus
capability, and effect value versus effect ID are separate fields. Artifact kind
is not a schema alias. Conflicting native schema/version observations remain
ambiguous, with typed anomalies. Invalid timezone offsets are not normalized
into valid ones. RFC3339 `-00:00` keeps its distinct unknown-local-offset label;
its known UTC instant is still usable for ordering. Unknown source versions,
times, ownership, verification and authority remain unknown.

Query kinds include `records`, `digest`, `occurrence`, `occurrences`, `family`,
`revision`, `run`, `trace`, `session`, `episode`, `task`, `goal`, `verification`,
`artifacts`, `quarantine`, `duplicates`, `provenance`, `schemas`, `times`, and
`verification-coverage`. `--private` opts into selected private path metadata.
No query permits public export or returns full source bodies.

## Low-level diagnostics

The older `build`, `build-request`, `query`, `summary`, `envelope`, `source`,
`validate-contracts`, `logical-manifest`, `rebuild-check`, and `schemas` commands
remain explicit diagnostic/library surfaces. A naked database and detached
receipt are not an adopted installation. New consumers must use verified bundles.
`source --ack-private-content` verifies the original archive and exact blob,
then writes original JSON value bytes to a new private file. A nested projection
returns its containing original value plus a JSON pointer, not reconstructed
child bytes. Do not expose this diagnostic command to untrusted agent inputs.

## Scope and acceptance

No semantic-memory import, native vault admission, signatures, learned routing,
scheduler, daemon, service configuration, online artifact fetch, public export,
source deletion, or live-store migration is implemented. Landing under Libraries
scripts is an additive proposal subject to current owner/instruction review.
Run target-repository and native-family checks separately before claiming those
gates. Tests use synthetic fixtures. Process-interruption tests are not a claim
of hardware power-loss or cross-platform certification.
