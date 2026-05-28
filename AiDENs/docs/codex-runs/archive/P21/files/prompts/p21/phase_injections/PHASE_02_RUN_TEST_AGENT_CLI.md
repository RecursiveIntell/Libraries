Phase 02 focus: operator-facing test-agent command.

Implement or repair `aidens run-test-agent`. It must call the real runner path. Do not duplicate the integration test in a fake CLI-only path.

Required proof:
- command runs;
- output bundle exists;
- receipts/event log/agency report exist;
- integration test still passes.
