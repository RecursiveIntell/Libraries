# Kernel and Region Runtime Plan

## Goal
Shape the v10 horizon without contaminating the v9 finish line.

## What belongs in the horizon
- right-graph law
- typed region protocols
- convergence/damping/oscillation artifacts
- nuisance-state discipline
- residual/syndrome-first local repair routing
- delta-aware incremental recomputation
- exact-on-small-slice oracle escalation

## What does not belong in the finish line
- shipping a giant omniscient graph
- introducing runtime geometry changes before the artifact constitution is frozen
- smuggling new semantics into old APIs

## Regional execution principles
- regions are the default execution unit
- regions exchange typed artifacts, not hidden mutable state
- local repair before global rebuild
- approximation is challengeable and budgeted
- witnesses/certificates outrank opaque scores
