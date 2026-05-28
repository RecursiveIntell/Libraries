# P31 Boundary Compiler Microkernel Report

**Run:** P31 — v11A Boundary Compiler Microkernel + Receipt Fixtures  
**Date:** `<fill>`  
**Repository root:** `<fill>`  
**Codex/session id:** `<optional>`

## 1. Chosen crate/module

- Chosen location:
- Existing or new:
- Reason for selection:
- Workspace integration decision:

## 2. Files changed

```text
<fill>
```

## 3. Artifact/contract types implemented

- [ ] `BoundaryCompilerProfileV1`
- [ ] `ParseReceiptV1`
- [ ] `RepairReceiptV1`
- [ ] `TreatmentIntegrityReceiptV1`
- [ ] `BoundaryDecisionV1`
- [ ] `BoundaryCompileResultV1`

Notes:

```text
<fill>
```

## 4. Strict JSON behavior

Duplicate-key detection method:

```text
<fill>
```

Malformed input behavior:

```text
<fill>
```

Unknown-field behavior:

```text
<fill>
```

Coercion behavior:

```text
<fill>
```

Resource ceiling behavior:

```text
<fill>
```

## 5. Canonicalization

Canonicalization profile:

```text
<fill>
```

Digest algorithm:

```text
<fill>
```

Limitations:

```text
<fill>
```

## 6. Receipts

Parse receipts emitted for:

- [ ] accepted inputs
- [ ] malformed inputs
- [ ] duplicate-key inputs
- [ ] unknown-field inputs
- [ ] resource-ceiling failures

Repair receipt behavior:

```text
<fill>
```

Treatment integrity receipt behavior:

```text
<fill>
```

## 7. Commands run

```bash
<fill>
```

## 8. Test results

```text
<fill>
```

Required fixture coverage:

- [ ] valid minimal accepted + canonical digest
- [ ] malformed rejected + receipt
- [ ] duplicate key rejected/quarantined
- [ ] duplicate key not last-write-wins
- [ ] unknown field rejected/quarantined
- [ ] string/number coercion rejected by default
- [ ] max bytes ceiling
- [ ] max nesting depth ceiling
- [ ] treatment-critical missing path receipt
- [ ] NoRepair no fake RepairedAccept
- [ ] canonical digest stable across object ordering
- [ ] accepted and rejected results both have receipts

## 9. Blockers / dependency issues

```text
<fill>
```

## 10. Confirmation of scope boundary

- [ ] No v11B graph compiler work added.
- [ ] No region runtime work added.
- [ ] No recursive kernel work added.
- [ ] No lawful subtraction work added.
- [ ] No broad repo cleanup attempted beyond selected crate/module.

## 11. P32 follow-up

Recommended next pass:

```text
P32 — Schema Compatibility + Reference Boundary Fixtures
```

Remaining work:

```text
<fill>
```
