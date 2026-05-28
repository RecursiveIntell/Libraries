# Claude implementation / review prompt

Review the repo against the 2026-03-22 hardening receipt, the restored root control-plane pack, and the active master issue matrix.

Your job is not to speculate about new architecture. Your job is to prove or falsify the remaining finish-line claims:

- Is the restored root pack internally consistent?
- Does the support claim stay scoped to the 17-crate lane?
- Does the demo emit typed artifacts through v21 -> v22 -> v23?
- Does the benchmark package actually measure replayability and temporal correctness?
- Is physical root reduction really closed?

Reject any diff that hides missing evidence behind prose.
