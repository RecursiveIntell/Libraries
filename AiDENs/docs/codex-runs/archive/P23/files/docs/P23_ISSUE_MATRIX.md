# P23 Issue Matrix

| ID | Priority | Phase | Issue | Acceptance |
|---|---|---:|---|---|
| P23-ZPY-001 | P0 | 01/03 | Package self-replay closure | Fresh extracted package passes included verifier or declares verifier excluded by role. |
| P23-ZPY-002 | P0 | 01 | Generic current-run logic | No P22-specific current-run allowlist remains; P23/P24 works by argument. |
| P23-ZPY-003 | P0 | 01/03 | Script ref strictness | Included scripts cannot reference excluded/missing local verifier dependencies. |
| P23-ZPY-004 | P0 | 01 | Legacy zip.py footgun removal | zip.py absent, archived, or hard-failing wrapper. |
| P23-ZPY-005 | P1 | 03 | Package role separation | release-context, next-codex-context/codex-run-full, audit-full roles tested or equivalent documented. |
| P23-HYG-001 | P0 | 02 | Pxx artifact classification registry | All P20/P21/P22 active artifacts classified or archived. |
| P23-CAP-001 | P0 | 04 | Receipt-bearing local agent run | Fixture-backed run produces run manifest and receipts. |
| P23-CAP-002 | P1 | 05 | Operator run inspect command | CLI or API inspects run directory and emits support-tier JSON. |
| P23-CAP-003 | P1 | 04/05 | Explicit degraded/unsupported provider behavior | Unsupported/cloud paths fail honestly with receipts/degradation fields. |
| P23-EVID-001 | P0 | 06 | Execution context in run reports | Run outputs include provider/tool route, budget, permits, replay command, support tier. |
| P23-CI-001 | P0 | 07 | P23 verifier and CI gates | scripts/p23_verify.sh passes with P23_REQUIRE_CARGO=1. |
