# Review and merge protocol

## Task review gate

A task branch enters review only when its handoff JSON validates, every required command has a
receipt, and the branch contains no uncommitted changes. The reviewer checks the actual diff and
adjacent source, not only named files.

## Reviewer outcomes

- `approve`: all assigned gates and evidence are sufficient.
- `changes_required`: actionable code/test/evidence defects remain.
- `blocked`: review cannot complete because source, environment, or dependency evidence is missing.

Only `approve` permits integration.

## Integration

Hermes records task base/head, review verdict, and chosen integration operation. Cherry-pick is
preferred for isolated commits; merge commits are appropriate when preserving a multi-commit
migration narrative. Squashing after receipt generation is forbidden because it breaks source binding.

## Post-merge law

Task commands rerun on the integration tree. A green task branch with a red integration tree is not
closed. Shared manifest/lockfile changes are reviewed after all dependent task commits are present.

## Phase review

The integration reviewer examines API/schema/wire compatibility, feature combinations, migration
ordering, duplicate types/registries, rollback from the integrated state, and all required phase
commands. The phase receipt names exact input/output commits and tree.

## Rejection triggers

- source receipt does not match reviewed head/tree;
- required command skipped or blocked;
- compatibility path is unowned or untested;
- new default/empty/warning behavior erases an error;
- migration deletes source/history data;
- generated evidence changed during verify;
- benchmark/readiness claim exceeds its receipt.
