# P26 CodingAgentV1 Spec

## Goal

Generalize the P25 flagship demo into a reusable supported-local coding-agent profile.

## Supported-local actions

- `repo.read`
- `repo.list`
- `repo.search`
- `patch.propose`
- `patch.apply` with permit
- `checks.run` with permit
- `run.inspect`
- `run.replay`

## Required safety

- All sandbox paths must remain under declared sandbox root.
- Write actions require scoped permit.
- Shell/check commands require scoped permit and bounded command allow/deny policy.
- Patch apply must dry-run or validate before modifying files where possible.
- Failed patch/check emits evidence and does not report success.

## Required examples

- `examples/agents/local-coding-agent/agent.json`
- `examples/agents/local-coding-agent/task.md`
- `fixtures/p26/coding-agent-repo/`
- expected success run bundle
- expected missing-permit abstention bundle
- expected failed-check repair bundle
