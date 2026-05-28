# P27 Master Issue Matrix

| ID | Sev | Source | Title | Phase(s) | Acceptance proof | Forbidden fix |
|---|---:|---|---|---|---|---|
| P27-001 | P0 | GPT audit + package inspection | Verifier wrappers point to missing scripts | 01 | `scripts/verify_current.sh`, `scripts/verify.sh`, CI all target existing verifier; script-ref assertion passes | deleting wrappers or claiming prior gate |
| P27-002 | P0 | GPT audit + Claude A-001 | Package self-replay unverified / false-green risk | 03,19 | replay command output captured; green or honest environment classification | `pass by prior gate` without shipped evidence |
| P27-003 | P0 | GPT audit | Active run truth split P22/P23/P26/P27 | 02 | active-run assertion passes across docs | leaving historical docs unlabeled |
| P27-004 | P1 | Claude A-003 | Ownership scanner false confidence | 04 | absent-baseline fixture fails closed | reporting zero duplicates from empty baseline |
| P27-005 | P1 | Claude A-004 | Reproducibility requires sibling monorepo | 07 | source-basis and prereq checker document exact layout | pretending standalone clone builds |
| P27-006 | P1 | GPT + Claude A-002 | Megafile concentration risk | 14,15 | contracts/CLI module split or accepted containment report | semantic rewrites while splitting |
| P27-007 | P2 | Claude A-005 | Root Markdown sprawl / drift | 05 | archive/label report; no active stale run docs | deleting evidence without archive |
| P27-008 | P2 | Claude A-006 | Scaffold-only profile crate inflation | 06 | crates removed/fenced; support profile updated | claiming scaffold as supported |
| P27-009 | P2 | Claude A-007 | No live provider path beyond mock/Ollama | 09 | mock E2E test; optional Ollama smoke skip | requiring cloud keys in verifier |
| P27-010 | P1 | GPT audit | Patch engine too simple for serious coding autonomy | 10,11 | invalid/ambiguous patch tests fail closed; receipts emitted | broad unsafe patch parsing |
| P27-011 | P1 | GPT audit | Memory grounding not durable canonical seam | 12 | backpointer/degradation receipt tests; no local memory truth | local memory DB substitute |
| P27-012 | P2 | 11A/spec | Evidence outputs lack exactness/proof/degradation semantics | 17 | exact/approx/support labels visible and tested | hidden semantic widening |
| P27-013 | P2 | Claude A-008 | `too_many_arguments` suppressions | 14,15,16 | configs/builders for high-argument APIs where touched | broad clippy allow sweep |
| P27-014 | P3 | Claude A-009 | Stale AGENTS.md | 00/overlay | P27 AGENTS.md installed | keeping P23 doctrine active |
| P27-015 | P2 | Research corpus | Strict structured-output boundary | 13 | duplicate-key/invalid JSON refusal for evidence-bearing inputs | lenient repair with no record |
| P27-016 | P2 | End-state spec | Support claims outrun evidence | 18,19 | support profile traceability table | marketing text |
| P27-017 | P2 | V10 risk register | V10 geometry contaminates finish line | all | V10 work remains stretch/design-only | implementing regions/hypergraphs prematurely |
| P27-018 | P2 | Execution evidence research | Execution context not durable enough | 08,17 | receipt store + replay fields | side-channel logs only |
| P27-019 | P2 | Coding-agent audit | CLI/operator UX for run evidence insufficient | 08,11,18 | inspect-run can load durable receipts | target-only ephemeral output |
| P27-020 | P2 | Package sidecars | stale codex-run archive active/current mismatch | 02,05,20 | codex archive report aligned with active run policy | moving files without manifest |
