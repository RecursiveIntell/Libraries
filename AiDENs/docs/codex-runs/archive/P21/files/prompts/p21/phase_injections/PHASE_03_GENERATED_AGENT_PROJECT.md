Phase 03 focus: generated agent project.

Make `aidens new coding-agent` create a project/config that actually runs safely. Defaults must be mock/disabled/safe, not dangerous.

Required proof:
- generated project contains operator docs/config/tests;
- `aidens run --config target/demo-agent/aidens.toml "read README"` works;
- receipts are emitted.
