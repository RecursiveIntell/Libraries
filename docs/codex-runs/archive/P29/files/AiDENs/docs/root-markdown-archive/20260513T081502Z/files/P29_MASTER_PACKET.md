# P29 Master Packet

## 1. Run

`P29 — AiDENs Evidence Repair + v11A Local Release Candidate + v11B Executable Seed`

## 2. Strategic purpose

P29 is a repair-plus-advance mega-pass.

It must:

1. Repair P28 evidence and package truth.
2. Close high-priority bugs from the Claude AiDENs audit.
3. Finish v11A local release-candidate gates for the declared supported-local agent path.
4. Seed v11B executable graph/region/subtraction surfaces.
5. Keep v11C reserved-only.
6. Produce a package that can self-replay from an extracted zip.

## 3. Source basis

Primary source basis:

- latest AiDENs package from 2026-05-06;
- P28 status/evidence/package sidecars;
- uploaded Claude hard audit report with 200 confirmed bugs;
- v11A/v11B/v11C specs already present in AiDENs package/spec docs.

## 4. Why P29 must not be lean

The P28 failure was not primarily implementation failure. It was evidence/package boundary failure.

The Claude audit also exposes deep runtime risks:

- HNSW TOCTOU and deadlock issues;
- SQLite migration atomicity issues;
- search/ranking/fusion correctness issues;
- quantization and vector disclosure issues;
- stack ID and tracing correctness issues;
- AiDENs v11A execution/receipt issues;
- high-risk unaudited effect/verification/forge/federation layers.

A thin packet would let Codex skip or compress the wrong pieces.

## 5. Correct final support labels

Allowed:

```text
p29-package-repaired
p29-supported-local-plus
v11A-local-release-candidate
v11B-executable-seed
v11C-reserved-only
```

Forbidden:

```text
v11B-complete
v11C-complete
production-cloud-ready
broad-autonomy-ready
canonical memory truth owner
canonical governance truth owner
canonical kernel truth owner
canonical provider/tool contract owner
canonical schema-generation owner
```

## 6. Execution model

P29 has 22 phases.

Phases 00–04 repair evidence, package, run identity, and audit triage.
Phases 05–11 close high-priority runtime and contract bugs.
Phases 12–16 finish v11A local release-candidate gates.
Phases 17–19 seed v11B executable territory.
Phases 20–21 converge docs/status and perform final hostile package replay.

## 7. Phase evidence rule

Every phase must produce:

```text
handoffs/p29/PHASE_XX_REPORT.md
```

Each report must include:

- files changed;
- tests/checks run;
- issue IDs addressed;
- evidence artifacts;
- unresolved risks;
- pass/fail gate status.

## 8. Release law

No package, no claim.

No extracted-package verifier, no release.

No manifest path resolution, no release.

No current-run identity agreement, no release.

No v11A local material-operation evidence, no v11A local release-candidate label.
