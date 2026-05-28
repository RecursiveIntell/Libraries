# P31 Fixture and Test Matrix

| Test | Input fixture | Profile setup | Expected decision | Required receipts |
|---|---|---|---|---|
| valid_minimal_json_is_accepted_and_gets_canonical_digest | `fixtures/valid_minimal.json` | allowed fields: `id`, `kind`, `amount`, `treatment` | Accept | Parse receipt with raw + canonical digest |
| malformed_json_is_rejected_with_parse_receipt | `fixtures/malformed.json` | default strict | Reject | Parse receipt with error |
| duplicate_key_is_rejected_or_quarantined | `fixtures/duplicate_key.json` | duplicate policy Reject or Quarantine | Reject/Quarantine | Parse receipt with ambiguity |
| duplicate_key_is_not_silently_last_write_wins | `fixtures/duplicate_key.json` | duplicate policy Reject | Reject | Parse receipt; result value absent |
| unknown_field_policy_rejects_surprise_structure | `fixtures/unknown_field.json` | allowed fields exclude `surprise` | Reject | Parse receipt + error |
| string_number_coercion_is_rejected_by_default | `fixtures/coercion_string_number.json` | expected `amount: number`; coercion disabled | Reject | Parse receipt + coercion/type error |
| resource_ceiling_rejects_large_input | generated or `fixtures/large_input.json` | max bytes below input length | Reject | Parse receipt with ceiling |
| resource_ceiling_rejects_deep_input | `fixtures/deep_input.json` | max depth below input depth | Reject | Parse receipt with ceiling |
| treatment_critical_missing_path_requires_integrity_receipt | `fixtures/treatment_missing.json` | treatment-critical `/treatment/id` | Reject/Quarantine or Accept with receipt depending policy | Treatment integrity receipt |
| no_repair_policy_never_emits_fake_repair_accept | malformed or duplicate input | repair policy NoRepair | not RepairedAccept | no RepairReceiptV1 |
| canonical_digest_is_stable_for_equivalent_object_ordering | `fixtures/order_a.json`, `fixtures/order_b.json` | same accepted profile | Accept both, same digest | Parse receipts both |
| accepted_and_rejected_results_both_have_receipts | valid + malformed | default strict | Accept + Reject | parse receipts both |

## Recommended fixture contents

Fixtures in this pack are deliberately small. Codex may copy them into the crate tests or keep test literals inline.
