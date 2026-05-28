# Compatibility Ledger

No compatibility surfaces are retained. This table is intentionally empty.

| Shim name | File path | Reason | Canonical replacement | Allowed lifetime | Removal criterion | Tests proving compatibility | Non-authoritative? |
|---|---|---|---|---|---|---|---|

`scripts/assert_compat_is_finite.sh` still enforces the ledger schema. Any future
compatibility surface must be rejected or removed rather than added here.
