# Compatibility Note For Historical Label `CANONICAL_STACK_SPEC_V6`

## Status

This file is a repo-local compatibility note for the historical `CANONICAL_STACK_SPEC_V6` label.
It is not the full canonical published stack specification.

## Purpose

The root hardening surface still needs one place that records the current implementation-truth dependency facts historically associated with the V6 label.
That purpose is compatibility and doc-truth, not spec publication theater.

## Implementation Truth Preserved Here

`semantic-memory` currently depends directly on forge-memory-bridge for canonical ProjectionImportBatchV3 importer wire types.

`knowledge-runtime` currently depends directly on constraint-compiler, kernel-execution, kernel-oracles, and recursive-kernel-core for bounded advisory inference.

## Scope Limit

Use this file to preserve truthful historical label mapping.
Do not cite it as the full canonical law for the stack.
