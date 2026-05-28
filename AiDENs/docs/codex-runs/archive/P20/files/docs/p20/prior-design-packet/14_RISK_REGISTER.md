# 14 — Risk Register

| Risk | Severity | Likelihood | Mitigation |
|---|---:|---:|---|
| AiDENs becomes a monolithic `RecallSession` clone | High | High | Strict crate boundary/dependency law; runner owns one run only |
| App profiles silently enable dangerous capabilities | High | High | Profiles expand into visible `AppPlanV1`; dangerous overrides explicit |
| Parser fallback is treated like native tool execution | High | Medium | Exact provider route labels and degraded receipts |
| UI becomes approval authority | High | Medium | `permit-kit` owns grants; UI only displays/returns user input |
| Queue retry creates duplicate jobs | High | Medium | attempt family + lease + queue hop receipts |
| Host wake becomes scheduler truth | High | Medium | split schedule/wake; host wake is projection only |
| Runtime becomes memory DB | High | Medium | memory-kit adapters only; no durable truth in runner |
| Contract crate becomes dumping ground | Medium | High | semantic crate owns artifact family; contracts owns shared primitives only |
| Schema generation exists without compatibility governance | Medium | High | meta-validation + historical schema compatibility tests |
| Disabled tools remain invocable | High | Medium | disabled tools absent at registration; invocation denial defense in depth |
| Provider native mode mislabeled | High | Medium | unknown native provider rejected; provider truth doctor check |
| Memory scope leaks personal data into coding app | High | Medium | profile scope policy; personal memory opt-in |
| Reranker violates temporal/scope filters | High | Medium | filter before rerank; widening receipt required |
| Config apply changes active run | Medium | Medium | run pins config generation; hot-swap explicit only |
| App setup still takes too long | High | Medium | CLI templates + generated tests + doctor checks are v0.1 requirements |
| Crate split becomes crate confetti | Medium | Medium | normal users depend only on `aidens` or `aidens-app-kit` |
| Existing Recall bugs leak into AiDENs | High | Medium | extract semantics, not files; testkit validates every footgun class |
| Advanced kernel crates delay v0.1 | Medium | High | defer kernel/plan/repair to later milestone |
| Daemon mode creates split-brain with local CLI | High | Medium | daemon authority mode; local fallback requires explicit override |
| Auto approval enables local damage | High | Medium | explicit unlock + hard denylist + rate limit + sandbox + receipts |

## Highest priority risks

1. Monolith recurrence.
2. Receipt-less execution.
3. Provider/tool mode dishonesty.
4. UI/daemon authority confusion.
5. Dangerous profile defaults.

## Hard stop conditions

Do not ship if any are true:

```text
write tool can run without permit or explicit safe profile
parser fallback path emits no receipt
unknown native provider maps to a native mode
runtime status says ready while provider is unavailable
queue retry lacks attempt family identity
profile enables shell/web/write without visible risk summary
```
