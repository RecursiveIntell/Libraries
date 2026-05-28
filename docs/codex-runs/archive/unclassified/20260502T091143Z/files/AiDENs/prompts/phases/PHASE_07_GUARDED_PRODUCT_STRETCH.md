# Phase 07 — Guarded Product Stretch

This phase is optional and must run only after Phases 00-06 are green.

## Allowed low-risk improvements

- clearer `doctor`, `status`, `provider-check`, `tools inspect`, and package example output;
- explicit support-tier reporting (`supported`, `partial`, `scaffold`, `deferred`, `failed`);
- better JSON output for packaging/reporting surfaces;
- stronger receipt redaction and summary views;
- package example classification tests;
- operator instructions for normal vs audit-full packaging.

## Forbidden stretch work

- promoting OpenAI/OpenRouter/Anthropic/cloud execution support without full tests;
- native tool loops unless already fully executable and tested;
- full daemon UX/socket/timer loop;
- multi-agent fanout;
- federation/mechanism/research workbench product flows;
- local replacement of canonical library semantics.

## Acceptance Gate

All pre-stretch gates still pass. Any new feature must have tests and truthful support-tier docs. If not, classify as partial/deferred or remove it.
