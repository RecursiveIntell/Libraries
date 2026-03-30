# Compatibility Note For Historical Label `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL`

## Status

This file is a repo-local compatibility note for the historical `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL` label.
It is not the full canonical published recursive inference kernel specification.

## Purpose

The active hardening surface needs a truthful bridge between the historical V7 filename and the current implementation facts that doc-truth checks rely on.
That is the only purpose of this file.

## Implementation Truth Preserved Here

`semantic-memory` currently depends directly on `forge-memory-bridge` for canonical `ProjectionImportBatchV3` importer wire types.

`knowledge-runtime` currently depends directly on `constraint-compiler`, `kernel-execution`, `kernel-oracles`, and `recursive-kernel-core` for bounded advisory inference.

## Scope Limit

Use this file as a compatibility note tied to the historical V7 label.
Do not present it as the full canonical publication for the stack.
