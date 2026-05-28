# Decision Receipts

Every evaluated decision emits `ControlDecisionReceiptV1`.

Required receipt content:

- input hash;
- canonical policy hash;
- evaluator algorithm ID and hash;
- hard rules checked and triggered;
- minimum action floors applied;
- axis scores;
- derived pressures;
- chosen action;
- rejected actions;
- reason codes;
- authority basis;
- evidence basis;
- valid-time basis;
- recorded-time;
- optional supersession reference.

The receipt is scoped to SCR-P0A evaluation. It is not a replacement for
upstream domain receipts.
