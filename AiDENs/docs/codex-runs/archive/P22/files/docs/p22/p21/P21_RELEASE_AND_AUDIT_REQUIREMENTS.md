# P21 Release and Audit Requirements

## Final source-controlled handoff

P21 must write:

```text
handoffs/p21/PHASE_00_REPORT.md
handoffs/p21/PHASE_01_BUILD_CERTIFICATION.md
handoffs/p21/PHASE_02_TEST_AGENT_CLI.md
handoffs/p21/PHASE_03_GENERATED_AGENT_PROJECT.md
handoffs/p21/PHASE_04_PROFILE_PLAN.md
handoffs/p21/PHASE_05_PROVIDER_TOOL_CERTIFICATION.md
handoffs/p21/PHASE_06_AGENCY_V02.md
handoffs/p21/PHASE_07_RECALL_EXTRACTION.md
handoffs/p21/PHASE_08_ARCHIVE_REPLAY.md
handoffs/p21/PHASE_09_STRETCH_REPORT.md
handoffs/p21/FINAL_AUDIT_REPORT.md
handoffs/p21/KNOWN_LIMITATIONS.md
```

## Target logs

P21 must keep full command logs under `target/p21/` and summarize them in handoff docs.

## Release archive

P21 must create a release candidate archive and verify by unpacking and re-running package scanners.
