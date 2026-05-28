# P20 Reference Interpreter and Conformance Plan

## Hard rule

No hard semantic seam may be marked complete if its reference behavior is still deferred.

## Minimum seams

- provider capability truth;
- permit/tool exposure enforcement;
- strict JSON and repair semantics;
- temporal query/as-of semantics;
- bridge atomicity/digest/backpointer preservation;
- runtime widening/degradation disclosure;
- repair-record invariants;
- agency policy decision semantics.

## Required action

For each seam:

1. Implement reference interpreter/test if feature is supported.
2. Otherwise demote feature to `partial` or `deferred` in docs/status.

## Required report

Create:

```text
docs/p20/REFERENCE_INTERPRETER_CLOSEOUT.md
```

Include pass/fail status per seam.
