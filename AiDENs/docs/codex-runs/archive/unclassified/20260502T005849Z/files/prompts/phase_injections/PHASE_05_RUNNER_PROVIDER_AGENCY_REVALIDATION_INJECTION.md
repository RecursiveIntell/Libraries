# Phase 05 injection — runner/provider/agency revalidation

Revalidate actual behavior.

Required:

- mock runner vertical slice test passes;
- provider matrix says mock executable, Ollama chat-only/partial, cloud/native tool loops unavailable unless tested;
- agency eval cases produce expected policy outcomes and required receipts;
- boundary repair still emits receipts and preserves treatment-integrity warnings.

Forbidden:

- native tool-loop claims without tests;
- agency behavior only in prompt text;
- silent fallback from provider/tool failures.
