# Source basis and epistemic status

| Field | Value |
|---|---|
| Repository | `RecursiveIntell/Libraries` |
| Default branch observed | `p32-schema-compat` |
| Locator commit | `c65972dbdf0ee5a7b472019b12c905a9de77c5c9` |
| Pack generated | `2026-07-15` |
| Audit mode | GitHub connector-backed static inspection |
| Local build/test run by auditor | No |

## Facts carried into this plan

The audited snapshot contained a large root Rust workspace, a much smaller supported release lane,
independent workspaces under `AiDENs/`, `Primitives/`, and `poly-kv/`, older root agent instructions,
three P0 false-success paths, and multiple competing ID/digest/codec/evidence authorities.

## Mandatory re-baseline

If `HEAD` differs from the locator commit:

1. record actual branch, commit, tree, dirty state, lockfile digest, toolchain, OS/arch;
2. classify every issue locator as confirmed, moved, partially fixed, closed with evidence,
   not found, or superseded;
3. never close an issue because a string disappeared;
4. add newly discovered defects to the issue matrix and decision log.

## Authority order

1. Current checkout source, tests, schemas, logs, and reproducible commands.
2. Source-bound receipts produced in this run.
3. This pack's issue contracts and guardrails.
4. Repository status/readme/prose claims.
5. Model memory and prior summaries.

Generated or derivative files do not outrank their source merely because they are newer.
