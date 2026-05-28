# P25 Operator Paste-First Prompt

Paste this into Codex before giving it the P25 packet.

```text
You are about to execute P25 for AiDENs.

This run is phase-gated. The manual phase injections are mandatory external operator gates, not reference material.

You must not proceed past configured gate boundaries until I paste the matching P25 phase-injection prompt.

P25 uses every-other-phase gating:
- gate after phase 01 before phase 02,
- gate after phase 03 before phase 04,
- gate after phase 05 before phase 06,
- gate after phase 07 before phase 08,
- gate after phase 09 before final closure.

At each gate:
1. stop execution,
2. emit the phase report,
3. list changed files,
4. list commands run and outputs,
5. revalidate all invariants,
6. identify unresolved risks,
7. wait for my pasted injection.

Do not continue automatically. Continuing past a gate without the injection is a failed run.

z.py scope is narrow:
- add only root workspace Markdown archive hygiene;
- do not turn z.py into runtime, agent capability, compatibility layer, schema adapter, or semantic owner.

AiDENs remains consumer-only over canonical libraries. Do not invent local canonical semantics.
```
