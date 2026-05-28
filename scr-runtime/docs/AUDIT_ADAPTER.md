# Audit Adapter

The audit adapter maps local fixture cases into `ControlEvaluationInputV1`.

Fixture signals become opaque external refs with `ref_kind = "signal"`. These
signals are deterministic fixture inputs, not source-of-truth claims. The
adapter does not fetch evidence, inspect repositories, call tools, or mutate
files.

The adapter exists to test the evaluator against hostile and ordinary audit
scenarios using stable local JSON fixtures.
