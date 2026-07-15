# Phase 2 — Digest, scope, ledger

Issues: `DIG-001`, `SCP-001`, `LED-001`.

Order: freeze framed digest V2; implement strict scope conversion; migrate ledger IDs/hash; add
anchored head; run compatibility/corruption corpus.

Corpus includes embedded NUL/newline, empty fields, Unicode, reordered maps, malformed JSONL middle/
tail, duplicated/skipped/reordered sequence, valid suffix truncation, and cross-scope collisions.

Exit: no material parse error is dropped; no separator-only structured digest; loss requires policy/
receipt; V1 remains readable.
