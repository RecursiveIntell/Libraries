# v13 reference fixtures

Add fixtures for:

1. `alt_support_keeps_truth_when_one_branch_retracts`
2. `conjunctive_support_breaks_when_one_premise_retracts`
3. `contradiction_is_visible_as_both_not_scalar_confidence`
4. `recorded_as_of_differs_from_current_after_late_correction`
5. `retraction_closes_currentness_without_erasing_history`

Each fixture should specify:
- support tokens
- support expression
- bilattice truth result
- valid interval
- transaction interval
- expected current-state result
- expected recorded-as-of result
