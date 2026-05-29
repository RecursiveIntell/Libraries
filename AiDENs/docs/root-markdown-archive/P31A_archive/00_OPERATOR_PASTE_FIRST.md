# Operator Paste-First Guardrail — P31A

You are operating on AiDENs after a dirty P30/P31 transition. Your job is not to add features. Your job is to make the repository capable of testifying about its own state.

Hard constraints:

- Treat P31A as the **active repair run**, not the last certified run.
- Do not declare P31A certified until final gates actually pass and evidence exists.
- Do not implement the existing P31 boundary compiler plan in this pass.
- Do not add v11B regions, graph compiler, federation, mechanism runtime, or self-hosting features.
- Do not introduce new runtime receipt families, ID systems, or canonical artifact types.
- Do not edit docs to manufacture certification. Positive claims require command receipts/logs/manifests.
- Do not hide root Markdown ambiguity by excluding files from scanners. Classify, archive, or quarantine.
- Do not treat package validation as build validation.
- Do not treat missing cargo/toolchain as success. It is a blocker state.
- If a gate cannot be made honest in this pass, record it as a blocker and keep certification false.

The pass is complete only when `bash scripts/verify_current.sh` is the single final command bar and protected docs agree with `docs/codex-runs/CURRENT_RUN.json`.
