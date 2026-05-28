# remote-oracle-admission scaffold crate

Proposed responsibility:
- own lease/request/result/replay-ticket helpers,
- own typed admission/dispute/re-admission workflows adjacent to remote-oracle use,
- preserve disclosure and replay semantics.

Non-goals:
- hidden local policy overrides,
- generic remote execution abstraction with disappearing provenance,
- score-only promotion paths.
