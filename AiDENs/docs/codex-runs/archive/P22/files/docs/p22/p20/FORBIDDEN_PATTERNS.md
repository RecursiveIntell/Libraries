# Forbidden Patterns

The next run is not complete if these remain in runtime code:

```text
AiDENs placeholder response
placeholder runner output
wire provider implementation next
fake success
TODO runtime
not implemented but healthy
skeletal advanced crates are healthy
```

Additional forbidden behavior:

- disabled provider returns text;
- mock provider is enabled implicitly;
- doctor says provider healthy when API key/config missing;
- raw LLM JSON reaches tools without boundary validation;
- dangerous shell/write/network tools are registered by default;
- generated apps manually wire internal crates;
- CLI/daemon/UI owns runtime truth;
- provider kind is mislabeled as native when using parser fallback;
- full tool registry exposed by default;
- queue/scheduler/daemon advanced sections claim healthy if not implemented.
