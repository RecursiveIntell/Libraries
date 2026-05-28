# Auto Injection Gate — Phase 04

This gate is loaded automatically by .codex/tools/auto_phase_runner.py. The operator must not paste it manually.

Before advancing from Phase 04:

1. Re-read AGENTS.md and AGENTS_COMPLETION_APPEND.md if present.
2. Confirm the current phase prompt was followed.
3. Run the phase's declared required commands from .codex/prompt_manifest.json where applicable.
4. Record command outputs or blockers in the phase report.
5. Do not weaken tests, validators, or release gates to pass.
6. Do not claim completion without receipts.
7. If a blocker remains, stop and write a repair note instead of continuing.

Required invariant: phase injections are automatic artifacts, not manual operator paste steps.
