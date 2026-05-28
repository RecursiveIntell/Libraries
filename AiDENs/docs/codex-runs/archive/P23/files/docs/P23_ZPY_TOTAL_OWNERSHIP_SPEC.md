# P23 z.py Total Ownership Spec

`z.py` is a certifier/packager and repo-state normalizer. It is not AiDENs' product, but it must stop contaminating all future work.

## Required final behavior

### Generic current-run support

`--codex-current-run P23` must derive current-run active paths generically. P24/P25 must work without code edits.

### Package roles

At minimum, the repo must distinguish these roles:

| Role | Purpose | Codex run control docs | Archived history | Verifier included |
|---|---|---:|---:|---:|
| `release-context` | clean source/operator release | no | no | optional, declared |
| `next-codex-context` | handoff for next coding pass | minimal | no | yes or explicitly excluded |
| `codex-run-full` | current run audit/control | current only | no | yes |
| `audit-full` | full historical audit | current + archive | yes | yes/optional declared |

If exact mode names are not implemented, aliases/equivalents must be documented and tested.

### Self-replay

A package that includes `scripts/p23_verify.sh` must include all files it references. No strict package may silently exclude verifier dependencies.

### Stale-run classification

Every Pxx/Pyy-marked active artifact must have a classification record. Exclusion is not enough; the system must know whether the artifact is an active regression fixture, active support matrix, archive evidence, deprecated template, or current instruction.

### zip.py law

Legacy `zip.py` must not remain an alternate packager. It must be removed, archived as evidence, or converted into a hard-failing wrapper that tells users to run `z.py`.

### Secret filename law

Intentional redaction fixtures may be included under safe filenames. Secret-content scanning remains strict. Do not make `--allow-secret-like-names` the normal path.
