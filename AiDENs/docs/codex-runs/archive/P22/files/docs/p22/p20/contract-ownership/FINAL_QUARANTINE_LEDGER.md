# Final Quarantine Ledger

| Item | Status | Discovered in phase | Quarantine record | Required human decision | Next safe action |
|---|---|---|---|---|---|
| delegation-kit-attestation-settlement | needs human owner decision | 02 | `docs/contract-ownership/quarantine/delegation-kit-attestation-settlement.md` | Rebuild delegation/admission helper behavior against canonical `attestation-exchange`, `federated-settlement`, and `remote-oracle-admission`, or keep it removed from product surface. | Keep `aidens-delegation-kit` as a disabled quarantine/status surface until owner-approved canonical wiring exists. |
| phase05-schema-sketches | quarantined historical sketches | 05 | `docs/contract-ownership/quarantine/phase05-schema-sketches.md` | Decide whether historical schema sketches should be archived, rewritten as non-authoritative docs, or replaced by owner-generated schemas. | Do not treat `*.sketch.json` files as generated schemas or canonical schema authority. |
| phase06-wrapper-canonical-record-gaps | quarantined canonical record gaps | 06 | `docs/contract-ownership/quarantine/phase06-wrapper-canonical-record-gaps.md` | Decide owner-approved production/persistence for concrete repair, validation, region, and subtraction canonical records. | Treat AiDENs DTOs with empty canonical ID vectors as display/report wrappers only; do not use them as canonical truth. |
