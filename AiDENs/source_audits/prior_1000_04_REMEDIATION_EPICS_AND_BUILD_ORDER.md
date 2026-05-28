# 04 — Remediation Epics and Build Order

## E01 — Receipt/log durability and no done without receipts

**Affected findings:** about 170

**Purpose:** Make durable receipt emission mandatory before visible completion; add hash-chain verification and corruption quarantine.

**Acceptance:**
- Introduce file locking/single-writer logs, atomic writes, fsync, hash-chain verification, corruption quarantine, and no done without receipts.
- Enforce OperatorContract effect sets and artifact lifecycle transition receipts for all material paths.
- Make docs/support labels/matrix statuses gate-derived and classify active/seeded/reserved surfaces precisely.

## E02 — Transactional patch and command execution hardening

**Affected findings:** about 185

**Purpose:** Replace patch/command footguns with atomic treatment receipts, structured argv, process-group kill, and replay fingerprints.

**Acceptance:**
- Replace or constrain patch_apply; require transaction receipt, before/after digests, rollback/quarantine, and post-write verification.
- Require structured argv, process-group kill, output caps, environment/toolchain fingerprints, and replay mismatch taxonomy.
- Add hostile sandbox tests and deny/quarantine receipts for .git/.env/symlink/hardlink/unicode/TOCTOU cases.

## E03 — Sandbox and security hostile corpus

**Affected findings:** about 105

**Purpose:** Add secret-path, symlink, hardlink, unicode, case-folding, TOCTOU, hidden metadata, and redaction tests.

**Acceptance:**
- Add hostile sandbox tests and deny/quarantine receipts for .git/.env/symlink/hardlink/unicode/TOCTOU cases.
- Add config receipt, redaction, environment fingerprint, default-policy, and migration fixtures.

## E04 — Provider honesty and route discipline

**Affected findings:** about 100

**Purpose:** Separate mock/local/Ollama/live routes and block fallback without degradation/support-label changes.

**Acceptance:**
- Make local/mock/Ollama/provider routes honest; prevent local=>mock fallback without explicit degradation and support-label change.
- Add tool exposure parity harness: declared vs registered vs executable vs exposed vs provider schema vs permit-used.

## E05 — Boundary compiler and schema governance

**Affected findings:** about 130

**Purpose:** Strict compiler profiles, full schema validation or unsupported-key rejection, compatibility gates, canonical digest identity.

**Acceptance:**
- Adopt strict boundary compiler profiles, full schema support or unsupported-key rejection, durable repair receipts, and fuzz/property tests.
- Gate generated schema changes with meta-validation, compatibility diffing, canonical digest identity, and migration receipts.

## E06 — Queue/daemon concurrency

**Affected findings:** about 145

**Purpose:** Lock/single-writer queue append, lease ownership, idempotency, safe-mode quarantine, hop receipts.

**Acceptance:**
- Make queue append/lease/complete atomic with leases, locks, hop receipts, idempotency proofs, and safe-mode quarantine.
- Introduce file locking/single-writer logs, atomic writes, fsync, hash-chain verification, corruption quarantine, and no done without receipts.

## E07 — Bitemporal/proof/view reference corpus

**Affected findings:** about 145

**Purpose:** Build pure reference fixtures and differential production-path tests for temporal/proof/view semantics.

**Acceptance:**
- Build bitemporal/proof/view reference fixture corpus and production-path differential tests.
- Replace marker-only assertions with semantic, negative, extracted-package, concurrency, and fuzz fixtures.

## E08 — Minimal v11B regional runtime slice

**Affected findings:** about 230

**Purpose:** One tiny right-graph/region/convergence/syndrome/repair/support-core/oracle-diff golden slice.

**Acceptance:**
- Implement one minimal v11B region with right-graph misuse tests, convergence failure, syndrome, repair, support-core, oracle diff.
- Build bitemporal/proof/view reference fixture corpus and production-path differential tests.
- Enforce OperatorContract effect sets and artifact lifecycle transition receipts for all material paths.

## E09 — Modularity/source-of-truth cleanup

**Affected findings:** about 150

**Purpose:** Split gravity wells and prevent facades/contracts from becoming hidden semantic owners.

**Acceptance:**
- Split gravity-well modules by source-of-truth boundary and forbid new semantics in facades.
- Enforce OperatorContract effect sets and artifact lifecycle transition receipts for all material paths.
- Add tool exposure parity harness: declared vs registered vs executable vs exposed vs provider schema vs permit-used.

## E10 — Docs/matrix/support evidence closure

**Affected findings:** about 100

**Purpose:** Gate support labels from executable evidence, convert issue statuses, and clarify active/seeded/reserved surfaces.

**Acceptance:**
- Make docs/support labels/matrix statuses gate-derived and classify active/seeded/reserved surfaces precisely.
- Replace marker-only assertions with semantic, negative, extracted-package, concurrency, and fuzz fixtures.

