# Claude Code Prompt — Architecture Finish V3

You are working from the latest full-stack snapshot and the attached handoff docs:

- `MASTER_ISSUE_MATRIX_ARCH_FINISH_V3.md`
- `MASTER_ISSUE_MATRIX_ARCH_FINISH_V3.csv`
- `FILE_AUDIT_INVENTORY_ARCH_FINISH_V3.md`
- `ARCHITECTURE_FINISH_EXECUTION_MAP_V3.md`
- `ARCHITECTURE_FINISH_ACCEPTANCE_CHECKLIST_V3.md`
- `CANONICAL_STACK_SPEC_V5.md`
- `LATEST5.md`
- `LATEST6.md`

## Mission

Finish as much of the **remaining architecture closure work** as is realistically possible in one strong pass.

This is **not** a brainstorming pass.
This is **not** a rename/comments-only pass.
This is **not** a “make the docs sound more canonical” pass.

Your job is to move the live code closer to the target architecture by closing the highest-pressure issues in the matrix.

## Primary goals

1. Harden `semantic-memory` canonical import semantics.
2. Close the most important `knowledge-runtime` execution gaps.
3. Burn down the worst fossilizable compat debt in the control-flow crates.
4. Tighten the bridge / forge seam where architecture truth still leaks.
5. Add or update tests so the closed issues are actually proven.

## Constraints

- Do **not** reopen the authority model.
- Do **not** add a runtime shadow database.
- Do **not** make the bridge own runtime/query policy.
- Do **not** join raw Forge truth directly into runtime answers.
- Do **not** invent fake lineage values.
- Do **not** create the wrong dependency direction to “fix” importer typing.
- Do **not** add new compatibility surfaces.
- Do **not** quietly preserve old behavior by just moving the same defaults around.

## Required working style

- Start by reading the issue matrix and execution map.
- Work in the execution order from `ARCHITECTURE_FINISH_EXECUTION_MAP_V3.md`.
- Prefer real behavior fixes over doc polish.
- Update or add tests for every closed issue.
- Keep phase-accepted compatibility debt frozen unless you are actively shrinking it.

## Minimum issues to target in this pass

Prioritize these first:

- I006
- I007
- I011
- I015
- I016
- I022
- I024
- I026
- I003
- I033

Then advance as many of these as realistically possible:

- I008
- I017
- I018
- I019
- I023
- I027
- I029
- I030

## Deliverables

At the end of the pass, provide:

1. A concise summary of what changed.
2. A list of issue IDs closed.
3. A list of issue IDs partially advanced.
4. A list of issue IDs intentionally left open.
5. Notes on any issue where the matrix was wrong or the code proved something different.
6. The exact tests added or changed.

## Quality bar

A good result means:
- malformed canonical imports fail loudly,
- relation truth-order invariants are enforced correctly,
- runtime is less downgrade-first and more genuinely executable,
- control-flow crates are more canonical-first,
- and the forge/bridge/memory seam is better proven.

A bad result means:
- mostly comments,
- mostly renames,
- more compat labels,
- or “future-proofing” that doesn’t close real behavior gaps.
