# Canonical Owner Proof Tests

This bundle installs script-level gates. Codex may also convert them into Rust tests if that fits the repo.

Required test behavior:

1. Construct a temporary duplicate public type in `aidens-contracts` and prove the duplicate gate fails.
2. Remove the duplicate and prove the gate passes.
3. Add a fake canonical schema family to AiDENs schema generation and prove schema scope gate fails.
4. Add exported local digest law and prove digest gate fails.

Do not leave negative fixtures in source files after testing. Use temp copies or test fixtures.
