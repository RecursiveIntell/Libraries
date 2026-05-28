# AiDENs Daemon-Safe Smoke

This example is a bounded Phase 09 stretch proof for the partial `autonomous-daemon` surface.

It exercises only the safe queue/schedule/wake controller path:

1. create an owner-scoped namespace;
2. enqueue one read-only schedule occurrence;
3. submit the same logical occurrence again and verify duplicate suppression;
4. acquire a lease;
5. enable safe mode;
6. verify a risky wake is blocked with a safe-mode receipt;
7. verify a read-only wake remains admissible;
8. drain queued work explicitly.

Run:

```bash
bash scripts/p21_daemon_smoke.sh target/p21/phase09/daemon-smoke
```

The script writes JSON command outputs and `daemon_smoke_report.json` under the output directory.

This does not claim a full desktop daemon, timer loop, socket server, UI bridge, or Recall-compatible scheduler. It is an operator smoke for AiDENs-owned orchestration over the queue/schedule/wake facade.
