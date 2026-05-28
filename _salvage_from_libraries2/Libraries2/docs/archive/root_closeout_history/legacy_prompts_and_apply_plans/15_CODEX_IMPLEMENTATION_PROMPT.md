# Codex implementation prompt

Work against the uploaded repo snapshot using this pack as the active command set.

## Mission

Take the repo as far as possible in the next pass **without reopening the architecture**.

## Non-negotiables

- do not invent a new authority model
- do not let bridge/runtime/control mutate truth directly
- do not make semantic edits inside split-only PRs
- do not start v14 or v15 implementation before v13 substrate work is real
- do not claim completion without proof artifacts

## Work order

1. `CI-001`
2. split issues
3. `V13-IDS-001`
4. `V13-FORGE-001`
5. `V13-BRIDGE-001`
6. `V13-SMEM-001`
7. `V13-VCTRL-001`
8. `REF-001`
9. second-order cleanup

## Output style

For each issue:
- list the crate/file diff
- list the acceptance criteria hit
- list the proofs produced
- list anything still open honestly

## Extra warning

Do not take the fun wrong path by pushing claim algebra into runtime heuristics before Forge and memory surfaces exist.
