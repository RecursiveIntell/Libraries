# Phase 02 — Automated Phase Runner

Goal: implement the automatic injection mechanism.

Required files:

```text
.codex/tools/phase_prompt_builder.py
.codex/tools/auto_phase_runner.py
```

Required behavior:
- read `.codex/prompt_manifest.json`;
- load master prompt, phase prompt, and auto-injection for each phase;
- assemble phase prompt text deterministically;
- support `--dry-run`;
- support `--print-prompts`;
- support `--phase <id>`;
- support `--from-phase <id>` and `--to-phase <id>`;
- support `--receipt <path>`;
- emit JSON receipt with phases, prompt paths, commands, status, and timestamps;
- if `--execute` is used, call `codex exec` per phase only if Codex CLI exists;
- never require operator-pasted manual injection.

Required command:

```bash
python .codex/tools/auto_phase_runner.py --dry-run --print-prompts --receipt .codex/runs/P0-completion/auto_phase_dry_run.json
```
