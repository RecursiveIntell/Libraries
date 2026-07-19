# Learning Agent Completion Ledger

**Projection status:** current-source projection only. This document is not a truth authority and does not mint, replace, or reinterpret owner receipts.

The snapshot below was generated from live repository state at the stated HEAD. Source digests are exact SHA-256 digests of the listed files. Historical receipts are explicitly not current execution evidence.

```json
{
  "schema": "learning-completion-ledger-v1",
  "projection_only": true,
  "branch": "feat/medusa",
  "head": "44a0a68deb15727c7b6b9886f8b21a9621be0539",
  "source_paths": [
    "AiDENs/Cargo.toml",
    "AiDENs/Cargo.lock",
    "AiDENs/crates/aidens-cli/src/lib.rs",
    "AiDENs/crates/aidens-runner/src/learning_controller.rs",
    "AiDENs/crates/aidens-runner/src/learning_experiment.rs",
    "AiDENs/crates/aidens-runner/src/learning_publication.rs",
    "AiDENs/crates/aidens-runner/src/learning_terminal.rs",
    "AiDENs/scripts/validate_learning_corpus_v2.py",
    "AiDENs/scripts/tests/test_learning_corpus_v2.py"
  ],
  "source_digests": {
    "AiDENs/Cargo.toml": "b052d48d75a6b149df40004dff80fc6f3375dc24c62990d67e77c249e6db621a",
    "AiDENs/Cargo.lock": "0aaf50bb91f6ad8274ea06e87e263cb6684fc43de92cc4d306e110ef7af02321",
    "AiDENs/crates/aidens-cli/src/lib.rs": "786f9874a2623b236f9881d1cab9c7e979d4ea1b408d4a204d54e5b4df8c45a2",
    "AiDENs/crates/aidens-runner/src/learning_controller.rs": "f324106800ff60fb2427b776c307e49b9a9b4fbb15bad5142306344959b63275",
    "AiDENs/crates/aidens-runner/src/learning_experiment.rs": "579ff629ff84f4f5d446a3813f6dbf49b9894a90478d0dc73c832916333f2547",
    "AiDENs/crates/aidens-runner/src/learning_publication.rs": "14f40bfdabb91d7d8d5ae4dbe194489287667f1ac2ddf2797c88d473c74af541",
    "AiDENs/crates/aidens-runner/src/learning_terminal.rs": "2d21540769383eb2c8e419dcdd5f726c48f78ea1b54e50401454a0ca8eae8105",
    "AiDENs/scripts/validate_learning_corpus_v2.py": "f4cc97a803f669a122254358f10330e3f6b58613593ef24054ed09c9de778f87",
    "AiDENs/scripts/tests/test_learning_corpus_v2.py": "bfb7c30805318ef792b6d6f8495ae72621432df7ea0a6d6c1be3da0755866eeb"
  },
  "tests": {
    "historical_passed": {
      "aidens-contracts": 102,
      "aidens-receipts": 20,
      "aidens-runner": 83,
      "aidens-cli": 69,
      "learning-corpus-v2-python": 14
    },
    "ignored_live_tests": [
      "aidens-runner: 2 ignored live tests",
      "aidens-cli: 1 ignored live test",
      "current-HEAD pinned-Podman closure/replay smoke: not run"
    ],
    "current_execution": "only the focused ledger contract test was run; no live learning execution was performed"
  },
  "receipts": {
    "historical": "Prior plan/source-reported receipts; not reproduced at this HEAD during this task",
    "current": "No current live closure receipt exists from this task"
  },
  "promotion_evidence": {
    "state": "absent",
    "qualification_corpus": "inspection/qualification evidence only; never promotion authority",
    "candidate": "inactive"
  },
  "claim_boundary": "AiDENs/docs/learning-agent/claim-boundary.json",
  "truth_authority": "canonical owner receipts and live execution"
}
```

## Interpretation

- The historical test counts and pinned-image evidence come from the saved plan and remain labelled historical.
- Ignored live tests and the absence of a current closure receipt prevent any current-HEAD live or production claim.
- Promotion evidence is absent; corpus qualification cannot promote a candidate.
- Rollback of this projection is deletion of these projection files only; owner evidence remains untouched.
