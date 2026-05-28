# Phase 5 — Search Disclosure and Explained Result Path

## Goal

Make approximate scoring visible and controlled.

## Required changes

1. Add explained vector scoring metadata:
   - codec family;
   - profile digest;
   - approximate score;
   - exact/f32 rerank score if available;
   - approximation class;
   - rerank status;
   - degradation flags.

2. Add controlled approximate search path:
   - only enabled when `allow_approximate_results = true`;
   - otherwise TurboQuant may evaluate in shadow but not affect returned ordering.

3. Add optional f32 rerank:
   - candidate scores from TurboQuant can be reranked from raw f32 when raw vectors exist;
   - result must state reranked or not.

4. Tests:
   - approximate disabled means no ranking effect;
   - approximate enabled returns metadata;
   - f32 rerank status visible;
   - degradation flags visible on fallback/missing raw vector;
   - existing hybrid/vector-only tests remain green.

## Non-goal

Do not replace HNSW candidate generation in this phase unless trivial and safe.
