# agent-guard

Linux control plane for AI agent security — defines the security trait surface and receipt types for agent sandboxing via BPF LSM, cgroup v2, Landlock, and seccomp.

## Scaffold notice

This crate is a **scaffold**: it defines the security trait surface and receipt types, but does not implement any OS-level enforcement yet. The `initialize()` method only sets an internal boolean. No actual sandboxing is applied.

Do not rely on this crate for production agent containment. Until real enforcement is implemented, treat all security decisions as advisory.

## What it gives you

- `ControlPlane` — unified interface to Linux security mechanisms (BPF LSM, cgroup v2, Landlock, seccomp)
- `Subject` — identity for an agent process requesting an action
- `Action` / `ActionType` — typed action request with target paths and parameters
- `SecurityDecision` — allow/deny/audit decision with reason and mechanism
- `SecurityMechanism` — enum of Linux security mechanisms
- Receipt types for audit trails of security decisions

## Linux only

This crate is only available on Linux. On other platforms, all methods return errors indicating the platform is unsupported.

## Quick start

```rust
use agent_guard::{AgentGuard, Subject, Action, ActionType};

let mut guard = AgentGuard::new();
guard.initialize()?;

let subject = Subject::new("agent-1");
let action = Action::new(ActionType::FileRead, "/tmp/sandbox");
let decision = guard.evaluate(&subject, &action)?;
```

## Roadmap

- [ ] BPF LSM enforcement
- [ ] cgroup v2 resource limits
- [ ] Landlock filesystem sandboxing
- [ ] seccomp syscall filtering
- [ ] Real `initialize()` with kernel-level setup

## License

Apache-2.0