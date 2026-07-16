# Evidence requirements

## Command receipt minimum

- stable receipt and command name;
- full argv and cwd;
- start/end/duration/exit code;
- branch, commit, tree, dirty-state digest, Cargo.lock digest;
- rustc/cargo, OS, architecture, Python/runtime versions;
- stdout/stderr paths, byte counts, and SHA-256;
- issue/task/stage association;
- whether source state changed during the command.

## Task handoff minimum

- base/head and changed files;
- semantic contract change;
- one row per acceptance gate with evidence;
- pass/fail/skipped/blocked for every command;
- residual risks, scope deviations, rollback, reviewer focus.

## Migration evidence

- old/new schema or wire version;
- input artifact/database digest and snapshot;
- reader/writer inventory;
- dry-run/preflight result;
- migrated/skipped/failed counts;
- idempotent replay result;
- postconditions and reverse command;
- preserved original data location.

## Phase receipt

- input/output commit and tree;
- included task heads and review verdicts;
- post-merge command receipts;
- issue states and open blockers;
- rollback tag/ref;
- pass/fail/blocked verdict.

## Final receipt

- source and environment binding;
- workspace inventory digest;
- complete required validation matrix;
- claims manifest digest;
- migration/rollback index;
- independent auditor verdict;
- clean-tree proof.

No receipt proves more than its bound command, source, environment, and input data.
