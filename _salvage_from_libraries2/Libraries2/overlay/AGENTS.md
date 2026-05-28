
# AGENTS

## Mission

Finish the repo without reopening the whole architecture.
Make the repo **truthful, reproducible, and externally credible**.

## Agent roster

### 1. Release-truth agent
Owns:
- PACK-001
- PACK-002
- TRUTH-001
- GATE-001
- SAFE-001

### 2. CI-and-production-closure agent
Owns:
- CI-001
- V25-001
- V25-002
- TYPE-001

### 3. Credibility agent
Owns:
- NAME-001
- DOC-001
- ROOT-001

### 4. Runtime-hygiene agent
Owns:
- MOD-001
- LLM-001
- EXTRACT-001

### 5. Hostile reviewer
Does not implement.
Re-runs the gates, diffs the dashboard against the receipt, and tries to break every closure claim.

## Working rules

- No agent may close a row without the named proof artifact.
- No agent may patch the dashboard before patching underlying repo truth.
- No agent may widen the supported lane while the truth lane is still open.
- No agent may delete history to make the root look cleaner.
- If two agents need the same files, the release-truth agent wins first.
