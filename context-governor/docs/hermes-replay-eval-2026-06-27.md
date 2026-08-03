# Hermes replay evaluation — 2026-06-27

Source: local Hermes SQLite session store.

Privacy boundary: this report intentionally stores aggregate metrics only. Raw transcript content and extracted probe strings are not written to this markdown file.

Method:

- Select the largest active Hermes sessions by message text volume.
- Convert messages into `CompactRequest` fixtures.
- Extract operational replay probes from active task, acceptance gates, decisions, errors, and file paths.
- Compare three prompt strategies:
  - `full`: original transcript
  - `head_tail`: naive first-message/last-message baseline
  - `context_governor`: compacted prompt plus exact fallback search

Claim boundary:

- This measures anchor survival and exact recoverability.
- It does not measure downstream LLM answer quality.
- It does not store private transcript text in this report.

## Aggregate

- Successful runs: 24
- Failures: 0
- Avg full tokens: 315660.5
- Avg context-governor tokens: 16003.9
- Avg token reduction vs full: 94.9%
- Avg head/tail recoverable rate: 2.7%
- Avg context-governor visible rate: 43.8%
- Avg context-governor recoverable rate: 98.8%
- Active task visible in context-governor: 24/24

## Per-session metrics

| session | messages | budget mode | target tokens | full tokens | head/tail tokens | governed tokens | governed recoverable | status/error |
|---|---:|---|---:|---:|---:|---:|---:|---|
| `20260619_235` | 729 | hard_cascade | 20000 | 353724 | 628 | 11623 | 100.0% | OK |
| `20260620_212` | 687 | hard_cascade | 20000 | 319591 | 71 | 11228 | 100.0% | OK |
| `20260618_124` | 470 | hard_cascade | 20000 | 317250 | 47 | 11209 | 100.0% | OK |
| `20260625_205` | 509 | hard_cascade | 20000 | 313423 | 348 | 7453 | 100.0% | OK |
| `20260619_231` | 560 | hard_cascade | 20000 | 311290 | 866 | 11276 | 100.0% | OK |
| `20260624_005` | 1051 | hard_cascade | 20000 | 310113 | 70 | 11857 | 93.8% | OK |
| `20260623_165` | 701 | hard_cascade | 20000 | 299643 | 129 | 11416 | 96.9% | OK |
| `20260621_001` | 1106 | hard_cascade | 20000 | 300250 | 448 | 12178 | 100.0% | OK |
| `20260619_235` | 729 | hard_cascade | 80000 | 353724 | 628 | 22949 | 100.0% | OK |
| `20260620_212` | 687 | hard_cascade | 80000 | 319591 | 71 | 25273 | 100.0% | OK |
| `20260618_124` | 470 | hard_cascade | 80000 | 317250 | 47 | 11639 | 100.0% | OK |
| `20260625_205` | 509 | hard_cascade | 80000 | 313423 | 348 | 7453 | 100.0% | OK |
| `20260619_231` | 560 | hard_cascade | 80000 | 311290 | 866 | 15946 | 100.0% | OK |
| `20260624_005` | 1051 | hard_cascade | 80000 | 310113 | 70 | 23015 | 93.8% | OK |
| `20260623_165` | 701 | hard_cascade | 80000 | 299643 | 129 | 12346 | 96.9% | OK |
| `20260621_001` | 1106 | hard_cascade | 80000 | 300250 | 448 | 29306 | 100.0% | OK |
| `20260619_235` | 729 | hard_cascade | 120000 | 353724 | 628 | 22949 | 100.0% | OK |
| `20260620_212` | 687 | hard_cascade | 120000 | 319591 | 71 | 25273 | 100.0% | OK |
| `20260618_124` | 470 | hard_cascade | 120000 | 317250 | 47 | 11639 | 100.0% | OK |
| `20260625_205` | 509 | hard_cascade | 120000 | 313423 | 348 | 7453 | 100.0% | OK |
| `20260619_231` | 560 | hard_cascade | 120000 | 311290 | 866 | 15946 | 100.0% | OK |
| `20260624_005` | 1051 | hard_cascade | 120000 | 310113 | 70 | 23015 | 93.8% | OK |
| `20260623_165` | 701 | hard_cascade | 120000 | 299643 | 129 | 12346 | 96.9% | OK |
| `20260621_001` | 1106 | hard_cascade | 120000 | 300250 | 448 | 29306 | 100.0% | OK |

Machine-readable report with probe text is local only:

`/home/sikmindz/Coding/Libraries/context-governor/target/context-governor-replay/hermes-replay-report.json`
