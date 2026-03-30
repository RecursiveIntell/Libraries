# Risks and forbidden shortcuts for forge-pilot

## 1. The most likely bad move

The most likely bad move is to turn `forge-pilot` into a second kernel or a second truth plane.

Do not do that.

## 2. Forbidden shortcuts

1. **No new wire schema inside pilot.**
2. **No direct memory write-through.**
3. **No private table reach-in.**
4. **No pilot-local reimplementation of `compile_batch`, `schedule_execution`, or oracle logic.**
5. **No hallucinated joint evidence groups when the export does not carry them.**
6. **No patch execution without a concrete patch and workspace/fixture.**
7. **No hidden advisory-only actions counted as “real experiments.”**
8. **No LLM-selected targets.**
9. **No infinite loops or silent retry families.**
10. **No root-doc claims before tests are green.**

## 3. Current-code trap to avoid

Older research snapshots understate how much of the kernel lane is already present.

If you write `forge-pilot` like the repo still lacks kernel crates, you will duplicate code and reopen architecture that is already settled.

## 4. Safe fallback posture

When the repo cannot support a richer pilot action yet:

- degrade explicitly,
- emit a visible `ThinExport` / `MissingKernelPayload` reason,
- and prefer a blocked or advisory-only result over a fake closed loop.
