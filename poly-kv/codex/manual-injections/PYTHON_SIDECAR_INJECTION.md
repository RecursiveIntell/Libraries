# Python Sidecar Manual Injection

Paste before implementing PyO3 bindings.

```text
Before writing PyO3 code, prove the Rust core API is stable enough to wrap. The Python sidecar must be a wrapper over canonical Rust semantics, not a semantic fork.

Rules:
- no daemon mode;
- no abi3 first unless the buffer protocol plan is explicitly revised;
- native extension is poly_kv._native;
- handwritten _native.pyi and py.typed are required;
- errors map to custom Python exceptions;
- expensive Rust operations detach from Python;
- every copy/zero-copy claim is represented in a receipt;
- no all-HF-model compatibility claim.
```
