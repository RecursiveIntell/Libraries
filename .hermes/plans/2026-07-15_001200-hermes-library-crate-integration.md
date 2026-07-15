# Hermes ← Libraries Crate Integration Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Wire three high-ROI Libraries crates (`cea-bridge`, `knowledge-router`, `llm-output-parser`) into Hermes as hooks and/or plugins, plus fix the `knowledge-runtime` test failures that block one of them.

**Architecture:** All three crates are Rust CLIs that speak JSON on stdin/stdout. They integrate as Hermes hooks (Python scripts that shell out to the Rust binaries) or as a compiled Python extension. No changes to Hermes core. No new model tools. All wiring is via `config.yaml` hooks and/or `~/.hermes/agent-hooks/` scripts — exactly how semantic-memory and context-governor are already wired.

**Tech Stack:** Rust CLIs (already installed in `~/.cargo/bin/`), Python hook scripts (shell out to CLIs), Hermes `config.yaml` hooks section.

---

## Current State (Evidence)

- **Repo:** `/home/sikmindz/Coding/Libraries`, HEAD `1fa0acf`
- **64 crates** in workspace, 63 compile clean, 1 fails (`hnsw-bench` — pre-existing wildcard deps)
- **Already wired into Hermes:** `semantic-memory` (MCP + 5 hooks), `agent-graph` (MCP), `context-governor` (context engine adapter)
- **Installed but not wired:** `cea-bridge`, `knowledge-router`, `llm-output-parser` (no bin yet)
- **Hermes hooks config:** `~/.hermes/config.yaml` has `on_session_start`, `pre_llm_call`, `post_llm_call`, `pre_tool_call`, `post_tool_call` (empty)
- **Existing hook scripts:** `~/.hermes/agent-hooks/` (sm-*.py family), `/home/sikmindz/Coding/agent-memory-kits/hermes/hooks/` (sm-primer, sm-recall, common.py)
- **knowledge-runtime tests:** 40 pass, 17 fail — all panicking at `stack-ids/src/ids.rs:209` (`define_id!` `new()` panics on cross-family or non-canonical IDs)

### Installed Binaries

```
~/.cargo/bin/cea-bridge         # cea-bridge 0.1.0 — record-telemetry, score-relevance, query-provenance, graph-stats
~/.cargo/bin/knowledge-router   # knowledge-router — classify, route (JSON stdin/stdout)
~/.cargo/bin/context-governor   # already active
~/.cargo/bin/agent-graph-mcp    # already active
```

### Key API Shapes

**cea-bridge `record-telemetry`:** stdin JSON `{tool_name, outcome, context_summary?, file_path?, session_id?}` → stdout `{status, inserted, evidence_kind, causal_claim: false}`

**cea-bridge `score-relevance`:** stdin JSON `{messages: [{index, tool_name?}], focus?}` → stdout `[{index, relevance_score, reason, evidence_kind, causal_claim: false}]`

**knowledge-router `classify`:** stdin JSON `{query: "..."}` → stdout `{mode, confidence, reason?}`

**knowledge-router `route`:** stdin JSON `{query, namespace?, domain?, workspace_id?, repo_id?, default_limit?}` → stdout `{classify: {...}, route: {legs: [{strategy, limit, filter?}]}}`

**llm-output-parser:** Rust library only (no bin). Public API: `parse_json(raw) -> Result<T>`, `parse_json_value(raw) -> Result<Value>`, `strip_think_tags(raw) -> String`, `parse_text(raw) -> Result<String>`, `parse_string_list(raw) -> Result<Vec<String>>`. 141 tests, all passing.

---

## Phase 0: Preserve Receipts

### Task 0.1: Snapshot current state

**Objective:** Capture pre-work state as receipts.

**Files:**
- None (read-only)

**Step 1:** Capture git status and test baseline.

```bash
cd /home/sikmindz/Coding/Libraries
git status --short | wc -l > /tmp/hermes-integration-baseline-status.txt
git diff --stat >> /tmp/hermes-integration-baseline-status.txt
git log -1 --oneline >> /tmp/hermes-integration-baseline-status.txt
cargo test -p knowledge-runtime --lib 2>&1 | tail -3 >> /tmp/hermes-integration-baseline-status.txt
cargo test -p llm-output-parser --lib 2>&1 | tail -3 >> /tmp/hermes-integration-baseline-status.txt
cargo test -p cea-bridge 2>&1 | tail -3 >> /tmp/hermes-integration-baseline-status.txt
```

---

## Phase 1: Wire `cea-bridge` as `post_tool_call` hook

**Rationale:** Every `patch` and `write_file` call should record tool telemetry. CEA-bridge is already installed, has no test failures, depends only on blake3 + rusqlite, and speaks JSON stdin/stdout. The hook is fail-open (telemetry is advisory, not causal proof).

### Task 1.1: Create the cea-bridge hook script

**Objective:** Python hook script that receives post-tool-call payload from Hermes and forwards to cea-bridge.

**Files:**
- Create: `~/.hermes/agent-hooks/cea-telemetry.py`

**Step 1:** Write the hook script.

```python
#!/usr/bin/env python3
"""
cea-bridge post_tool_call hook for Hermes.

Records file-edit tool telemetry to the CEA telemetry database.
Fail-open: any error is logged to stderr and swallowed — never blocks the agent.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

CEA_BINARY = os.environ.get("CEA_BRIDGE_BIN", str(Path.home() / ".cargo/bin/cea-bridge"))
CEA_DB = os.environ.get("CEA_TELEMETRY_DB", str(Path.home() / ".hermes/cea-telemetry-v2.db"))
TIMEOUT_SEC = 10

# Only record these tools — the ones that mutate files
RECORDED_TOOLS = {"patch", "write_file", "browser_click", "browser_type"}


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        # Not valid JSON from Hermes — nothing to do
        return

    tool_name = payload.get("tool_name", "")
    if tool_name not in RECORDED_TOOLS:
        return

    # Build the telemetry request for cea-bridge
    outcome = "success"
    if payload.get("error"):
        outcome = "error"

    # Extract file path if available from tool arguments
    file_path = None
    args = payload.get("tool_args", {})
    if isinstance(args, dict):
        file_path = args.get("path") or args.get("file_path")

    request = {
        "tool_name": tool_name,
        "outcome": outcome,
        "context_summary": payload.get("context_summary", ""),
        "file_path": file_path,
        "session_id": payload.get("session_id", ""),
    }

    try:
        proc = subprocess.run(
            [CEA_BINARY, "record-telemetry", "--db", CEA_DB],
            input=json.dumps(request),
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SEC,
        )
        if proc.returncode != 0:
            print(f"cea-bridge warning: {proc.stderr.strip()}", file=sys.stderr)
    except subprocess.TimeoutExpired:
        print("cea-bridge timeout (10s) — telemetry skipped", file=sys.stderr)
    except FileNotFoundError:
        print(f"cea-bridge binary not found at {CEA_BINARY} — telemetry skipped", file=sys.stderr)
    except Exception as e:
        print(f"cea-bridge error: {e}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

**Step 2:** Make it executable.

```bash
chmod +x ~/.hermes/agent-hooks/cea-telemetry.py
```

### Task 1.2: Test the hook script standalone

**Objective:** Verify the hook works with simulated Hermes payload.

**Step 1:** Test with a simulated payload.

```bash
echo '{"tool_name": "patch", "tool_args": {"path": "/tmp/test.txt"}, "error": null, "session_id": "test-1"}' | python3 ~/.hermes/agent-hooks/cea-telemetry.py
echo "exit=$?"
```

Expected: no output on stdout, exit 0. Check the DB:

```bash
~/.cargo/bin/cea-bridge graph-stats --db ~/.hermes/cea-telemetry-v2.db
```

Expected: JSON output with `total_telemetry >= 1`.

**Step 2:** Test fail-open with missing binary.

```bash
CEA_BRIDGE_BIN=/nonexistent echo '{"tool_name": "patch", "tool_args": {"path": "/tmp/test.txt"}, "error": null}' | python3 ~/.hermes/agent-hooks/cea-telemetry.py 2>&1
```

Expected: stderr warning, exit 0 (never non-zero).

**Step 3:** Test with non-recorded tool (should be silent no-op).

```bash
echo '{"tool_name": "web_search", "tool_args": {}, "error": null}' | python3 ~/.hermes/agent-hooks/cea-telemetry.py
```

Expected: no output, exit 0.

### Task 1.3: Wire the hook into config.yaml

**Objective:** Add cea-telemetry.py as a post_tool_call hook in Hermes config.

**Files:**
- Modify: `~/.hermes/config.yaml`

**Step 1:** Replace the empty `post_tool_call: []` with the hook entry.

Current:
```yaml
  post_tool_call: []
```

New:
```yaml
  post_tool_call:
    - command: python3 /home/sikmindz/.hermes/agent-hooks/cea-telemetry.py
      timeout: 15
```

Note: No `matcher` field — we want all tool calls (the script itself filters to recorded tools).

**Step 2:** Verify config is valid.

```bash
hermes config validate 2>/dev/null || python3 -c "import yaml; yaml.safe_load(open('$HOME/.hermes/config.yaml'))" && echo "config valid"
```

### Task 1.4: End-to-end verification

**Objective:** Confirm a real tool call in a new Hermes session generates telemetry.

**Step 1:** Start a new Hermes session (or use current), make any file edit.

**Step 2:** Check the telemetry DB:

```bash
~/.cargo/bin/cea-bridge graph-stats --db ~/.hermes/cea-telemetry-v2.db
```

Expected: `total_telemetry >= 1` with tool_name entries for `patch` or `write_file`.

**Step 3:** Score relevance to confirm the telemetry is queryable:

```bash
echo '{"messages": [{"index": 0, "tool_name": "patch"}, {"index": 1, "tool_name": "write_file"}], "focus": "file edit"}' | ~/.cargo/bin/cea-bridge score-relevance --db ~/.hermes/cea-telemetry-v2.db
```

Expected: JSON array with relevance scores.

---

## Phase 2: Fix `knowledge-runtime` test failures

**Rationale:** 17 of 57 tests fail because `knowledge-runtime` uses `stack-ids` ID constructors (`EntityId::new(...)`, `CodeEntityId::new(...)`) that now panic when given unprefixed strings. The `define_id!` macro's `new()` method panics on values that don't start with the family prefix (e.g. `EntityId::new("foo")` panics because it expects `entity:foo`). The fix is to use `try_new()` in test/code paths, or construct IDs with the proper family-qualified strings.

### Task 2.1: Identify the failing test patterns

**Objective:** Catalogue every failing test and the exact panic site.

**Files:**
- Inspect: `knowledge-runtime/src/entity/code_ids.rs`
- Inspect: `knowledge-runtime/src/entity/registry.rs`
- Inspect: `stack-ids/src/ids.rs` (lines 195-230 — the `define_id!` macro)

**Step 1:** Run the tests and capture output.

```bash
cd /home/sikmindz/Coding/Libraries
cargo test -p knowledge-runtime --lib 2>&1 | tee /tmp/knowledge-runtime-failures.log
```

**Step 2:** Extract failing test names and panic sites.

```bash
grep -B1 "panicked at" /tmp/knowledge-runtime-failures.log | grep "test " | sed 's/test //' | sed 's/ \.\.\. FAILED//' > /tmp/kr-failing-tests.txt
```

All 17 failures panic at `stack-ids/src/ids.rs:209` — the `define_id!` macro's `new()` method. The tests construct `EntityId` or `CodeEntityId` with bare strings like `"foo"` instead of `"entity:foo"`.

### Task 2.2: Fix `entity::code_ids` tests (8 failing)

**Objective:** Fix the 8 failing tests in `entity::code_ids` by using family-qualified strings or `try_new()`.

**Files:**
- Modify: `knowledge-runtime/src/entity/code_ids.rs` (the `tests` module at the bottom)

**Step 1:** Read the current test code.

```bash
cd /home/sikmindz/Coding/Libraries
sed -n '/mod tests/,/^}/p' knowledge-runtime/src/entity/code_ids.rs | head -100
```

**Step 2:** For each test that constructs an `EntityId`, `CodeEntityId`, or similar ID with `::new(...)`, change the argument to include the family prefix.

Example fix pattern:
```rust
// Before (panics):
let id = EntityId::new("foo");

// After (correct):
let id = EntityId::new("entity:foo");
```

Or use `try_new()` for tests that intentionally test invalid input:
```rust
// Before:
let id = EntityId::new("foo");

// After:
let id = EntityId::try_new("foo").unwrap();
```

The family prefix for each type can be found by checking `family_name(stringify!($name))` — it lowercases the type name. So `EntityId` → `"entity:"`, `CodeEntityId` → `"code_entity:"` (or check the actual `family_name` function).

**Step 3:** Check what `family_name` produces.

```bash
grep -A10 "fn family_name" stack-ids/src/ids.rs | head -15
```

**Step 4:** Apply fixes to all 8 failing tests in `code_ids.rs`. Each test constructs IDs — update the construction to use the proper family prefix.

**Step 5:** Run the tests.

```bash
cargo test -p knowledge-runtime --lib entity::code_ids 2>&1 | tail -5
```

Expected: all `code_ids` tests pass.

### Task 2.3: Fix `entity::registry` tests (9 failing)

**Objective:** Fix the 9 failing tests in `entity::registry` — same root cause.

**Files:**
- Modify: `knowledge-runtime/src/entity/registry.rs` (the `tests` module)

**Step 1:** Read the current test code.

```bash
sed -n '/mod tests/,/^}/p' knowledge-runtime/src/entity/registry.rs | head -120
```

**Step 2:** Apply the same fix pattern — add family prefixes to all `EntityId::new(...)` and similar calls.

**Step 3:** Run the tests.

```bash
cargo test -p knowledge-runtime --lib entity::registry 2>&1 | tail -5
```

Expected: all `registry` tests pass.

### Task 2.4: Full knowledge-runtime test suite

**Objective:** Confirm all 57 tests pass.

**Step 1:** Run the full suite.

```bash
cargo test -p knowledge-runtime --lib 2>&1 | tail -5
```

Expected: `57 passed; 0 failed`.

**Step 2:** Commit the fix.

```bash
cd /home/sikmindz/Coding/Libraries
git add knowledge-runtime/src/entity/code_ids.rs knowledge-runtime/src/entity/registry.rs
git commit -m "fix(bound-006): knowledge-runtime entity ID family prefix fixes — all 57 tests pass"
```

---

## Phase 3: Wire `knowledge-router` as `pre_llm_call` hook

**Rationale:** The `sm-recall` hook currently fires raw queries at semantic-memory. `knowledge-router` classifies queries (semantic lookup vs entity lookup vs temporal lookup) and plans multi-leg retrieval routes. Inserting it before `sm-recall` means the recall hook can use the classified mode to choose better retrieval strategies.

**Design decision:** We will NOT replace `sm-recall`. We will add `knowledge-router` as a separate `pre_llm_call` hook that runs *before* `sm-recall` and writes a classification result to a temp file that `sm-recall` reads. This is the least invasive integration. If `knowledge-router` fails, `sm-recall` proceeds with its current behavior (fail-open).

### Task 3.1: Create the knowledge-router hook script

**Objective:** Python hook that classifies the user's latest message and writes the route plan for sm-recall to consume.

**Files:**
- Create: `~/.hermes/agent-hooks/kr-classify.py`

**Step 1:** Write the hook script.

```python
#!/usr/bin/env python3
"""
knowledge-router pre_llm_call hook for Hermes.

Classifies the latest user message and writes a route plan that sm-recall
can consume to choose better retrieval strategies.

Fail-open: any error is swallowed — sm-recall proceeds with its default behavior.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

KR_BINARY = os.environ.get("KNOWLEDGE_ROUTER_BIN", str(Path.home() / ".cargo/bin/knowledge-router"))
TIMEOUT_SEC = 10
ROUTE_FILE = Path(os.environ.get("KR_ROUTE_FILE", str(Path.home() / ".hermes/kr-last-route.json")))


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return

    # Extract the latest user message from the Hermes payload
    messages = payload.get("messages", [])
    latest_user = None
    for msg in reversed(messages):
        if msg.get("role") == "user":
            latest_user = msg.get("content", "")
            break

    if not latest_user or not latest_user.strip():
        return

    # Classify the query
    classify_request = {"query": latest_user[:2000]}  # truncate very long messages

    try:
        proc = subprocess.run(
            [KR_BINARY, "classify"],
            input=json.dumps(classify_request),
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SEC,
        )
        if proc.returncode != 0:
            print(f"knowledge-router warning: {proc.stderr.strip()}", file=sys.stderr)
            return

        classify_result = json.loads(proc.stdout)

        # Also get the route plan
        route_request = {
            "query": latest_user[:2000],
            "namespace": "general",
            "default_limit": 10,
        }
        proc2 = subprocess.run(
            [KR_BINARY, "route"],
            input=json.dumps(route_request),
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SEC,
        )
        route_result = json.loads(proc2.stdout) if proc2.returncode == 0 else None

        # Write the combined result for sm-recall to optionally read
        output = {
            "classify": classify_result,
            "route": route_result,
        }
        ROUTE_FILE.write_text(json.dumps(output, indent=2))

    except subprocess.TimeoutExpired:
        print("knowledge-router timeout (10s) — classification skipped", file=sys.stderr)
    except FileNotFoundError:
        print(f"knowledge-router not found at {KR_BINARY}", file=sys.stderr)
    except Exception as e:
        print(f"knowledge-router error: {e}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

**Step 2:** Make it executable.

```bash
chmod +x ~/.hermes/agent-hooks/kr-classify.py
```

### Task 3.2: Test the hook script standalone

**Objective:** Verify classification works with a simulated payload.

**Step 1:** Test with a semantic query.

```bash
echo '{"messages": [{"role": "user", "content": "what did we do about the ID migration?"}]}' | python3 ~/.hermes/agent-hooks/kr-classify.py
cat ~/.hermes/kr-last-route.json | python3 -m json.tool
```

Expected: `~/.hermes/kr-last-route.json` contains a `classify.mode` field (likely `SemanticLookup`) and a `route.legs` array.

**Step 2:** Test with an entity query.

```bash
echo '{"messages": [{"role": "user", "content": "tell me about @stack-ids"}]}' | python3 ~/.hermes/agent-hooks/kr-classify.py
cat ~/.hermes/kr-last-route.json | python3 -m json.tool
```

Expected: `classify.mode` should be `EntityLookup` with a `mention` field.

**Step 3:** Test fail-open with missing binary.

```bash
KNOWLEDGE_ROUTER_BIN=/nonexistent echo '{"messages": [{"role": "user", "content": "test"}]}' | python3 ~/.hermes/agent-hooks/kr-classify.py 2>&1
echo "exit=$?"
```

Expected: stderr warning, exit 0.

### Task 3.3: Wire the hook into config.yaml

**Objective:** Add kr-classify.py as the first pre_llm_call hook (before sm-recall).

**Files:**
- Modify: `~/.hermes/config.yaml`

**Step 1:** Edit the `pre_llm_call` section to add kr-classify BEFORE sm-recall.

Current:
```yaml
  pre_llm_call:
    - command: env SEMANTIC_MEMORY_DIR=... python3 /home/sikmindz/Coding/agent-memory-kits/hermes/hooks/sm-recall.py
      timeout: 15
```

New:
```yaml
  pre_llm_call:
    - command: python3 /home/sikmindz/.hermes/agent-hooks/kr-classify.py
      timeout: 10
    - command: env SEMANTIC_MEMORY_DIR=... python3 /home/sikmindz/Coding/agent-memory-kits/hermes/hooks/sm-recall.py
      timeout: 15
```

Note: Keep the existing sm-recall env vars exactly as they are. Just add the kr-classify entry before it.

**Step 2:** Verify config.

```bash
python3 -c "import yaml; yaml.safe_load(open('$HOME/.hermes/config.yaml'))" && echo "config valid"
```

### Task 3.4: End-to-end verification

**Objective:** Confirm a new Hermes session uses knowledge-router classification.

**Step 1:** Start a new Hermes session and ask a question.

**Step 2:** Check the route file was written:

```bash
cat ~/.hermes/kr-last-route.json | python3 -m json.tool
```

Expected: JSON with `classify` and `route` fields matching the query type.

**Step 3:** Confirm sm-recall still works (semantic-memory results still injected into context).

---

## Phase 4: Build `llm-output-parser` CLI and wire as `post_llm_call` hook

**Rationale:** Hermes parses LLM output in Python. `llm-output-parser` is a 141-test, MIT-licensed, zero-dependency Rust crate that strips think blocks, extracts JSON from fences, and repairs malformed JSON. Adding it as a `post_llm_call` hook hardens the parsing pipeline. The hook will strip `<think>` blocks and extract clean text/JSON from the model's raw output before Hermes processes it further.

### Task 4.1: Add a binary target to llm-output-parser

**Objective:** Create a CLI binary so the library can be called from Python hooks.

**Files:**
- Create: `llm-output-parser/src/bin/llm-parse.rs`
- Modify: `llm-output-parser/Cargo.toml` (add `[[bin]]` section)

**Step 1:** Add the bin target to Cargo.toml.

Add to `llm-output-parser/Cargo.toml`:
```toml
[[bin]]
name = "llm-parse"
path = "src/bin/llm-parse.rs"
```

**Step 2:** Write the CLI binary.

```rust
//! CLI for llm-output-parser — reads raw LLM output on stdin, writes parsed result on stdout.
//!
//! Usage:
//!   llm-parse json     — extract JSON from raw LLM output
//!   llm-parse text     — clean text (strip think blocks, fences)
//!   llm-parse list     — extract string list
//!   llm-parse strip    — strip think blocks only, pass through rest
//!   llm-parse think-check — exit 0 if think blocks found, 1 if not

use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: llm-parse <command>");
        eprintln!("commands: json, text, list, strip, think-check");
        std::process::exit(1);
    }

    let command = &args[1];
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap_or_else(|e| {
        eprintln!("error reading stdin: {e}");
        std::process::exit(1);
    });

    match command.as_str() {
        "json" => match llm_output_parser::parse_json_value(&input) {
            Ok(value) => {
                let mut stdout = io::stdout();
                serde_json::to_writer_pretty(&mut stdout, &value).unwrap_or_default();
                stdout.write_all(b"\n").unwrap_or_default();
            }
            Err(e) => {
                eprintln!("parse error: {e}");
                std::process::exit(1);
            }
        },
        "text" => match llm_output_parser::parse_text(&input) {
            Ok(text) => print!("{text}"),
            Err(e) => {
                eprintln!("parse error: {e}");
                std::process::exit(1);
            }
        },
        "list" => match llm_output_parser::parse_string_list(&input) {
            Ok(items) => {
                let json = serde_json::to_string_pretty(&items).unwrap_or_default();
                println!("{json}");
            }
            Err(e) => {
                eprintln!("parse error: {e}");
                std::process::exit(1);
            }
        },
        "strip" => {
            let cleaned = llm_output_parser::strip_think_tags(&input);
            print!("{cleaned}");
        }
        "think-check" => {
            let has_think = input.contains("<think>") || input.contains("</think>");
            std::process::exit(if has_think { 0 } else { 1 });
        }
        _ => {
            eprintln!("unknown command: {command}");
            eprintln!("commands: json, text, list, strip, think-check");
            std::process::exit(1);
        }
    }
}
```

**Step 3:** Build and install.

```bash
cd /home/sikmindz/Coding/Libraries
cargo install --path llm-output-parser --force
which llm-parse  # should be ~/.cargo/bin/llm-parse
```

**Step 4:** Verify the binary works.

```bash
echo '```json
{"key": "value", "items": [1, 2, 3]}
```' | llm-parse json | python3 -m json.tool
```

Expected: parsed JSON `{"key": "value", "items": [1, 2, 3]}`.

```bash
echo '<think>internal reasoning here</think>The answer is 42.' | llm-parse strip
```

Expected: `The answer is 42.` (think block removed).

```bash
echo '<think>internal reasoning here</think>The answer is 42.' | llm-parse think-check; echo "exit=$?"
```

Expected: exit 0.

**Step 5:** Run the existing test suite to confirm the new bin doesn't break anything.

```bash
cargo test -p llm-output-parser --lib 2>&1 | tail -3
```

Expected: `141 passed; 0 failed`.

### Task 4.2: Create the llm-parse hook script

**Objective:** Python hook that strips think blocks from LLM output before Hermes processes it.

**Files:**
- Create: `~/.hermes/agent-hooks/llm-clean.py`

**Step 1:** Write the hook script.

```python
#!/usr/bin/env python3
"""
llm-output-parser post_llm_call hook for Hermes.

Strips <think> blocks from LLM output and optionally extracts clean JSON
when the model wraps it in markdown fences. Fail-open: on any error, the
original LLM output passes through unchanged.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

LLM_PARSE_BIN = os.environ.get("LLM_PARSE_BIN", str(Path.home() / ".cargo/bin/llm-parse"))
TIMEOUT_SEC = 10


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return

    # Get the raw LLM response from the payload
    raw_output = payload.get("llm_response", "")
    if not raw_output or not isinstance(raw_output, str):
        return

    # Quick check: does the output even contain think blocks?
    if "<think>" not in raw_output and "</think>" not in raw_output:
        return  # nothing to clean — skip the subprocess call entirely

    try:
        proc = subprocess.run(
            [LLM_PARSE_BIN, "strip"],
            input=raw_output,
            capture_output=True,
            text=True,
            timeout=TIMEOUT_SEC,
        )
        if proc.returncode == 0:
            # Write the cleaned output back into the payload
            cleaned = proc.stdout
            payload["llm_response"] = cleaned
            # Emit the modified payload for Hermes to read
            json.dump(payload, sys.stdout)
            sys.stdout.write("\n")
    except subprocess.TimeoutExpired:
        print("llm-parse timeout (10s) — output not cleaned", file=sys.stderr)
    except FileNotFoundError:
        print(f"llm-parse not found at {LLM_PARSE_BIN}", file=sys.stderr)
    except Exception as e:
        print(f"llm-parse error: {e}", file=sys.stderr)


if __name__ == "__main__":
    main()
```

**Step 2:** Make it executable.

```bash
chmod +x ~/.hermes/agent-hooks/llm-clean.py
```

### Task 4.3: Test the hook script standalone

**Objective:** Verify think-block stripping works with simulated payload.

**Step 1:** Test with think blocks present.

```bash
echo '{"llm_response": "<think>I should help the user</think>Here is your answer: 42"}' | python3 ~/.hermes/agent-hooks/llm-clean.py
```

Expected: JSON output with `"llm_response": "Here is your answer: 42"`.

**Step 2:** Test without think blocks (should be silent no-op — no subprocess call).

```bash
echo '{"llm_response": "just a normal response"}' | python3 ~/.hermes/agent-hooks/llm-clean.py
```

Expected: no output (the script returns early without calling the binary).

**Step 3:** Test fail-open with missing binary.

```bash
LLM_PARSE_BIN=/nonexistent echo '{"llm_response": "<think>x</think>y"}' | python3 ~/.hermes/agent-hooks/llm-clean.py 2>&1
echo "exit=$?"
```

Expected: stderr warning, exit 0.

### Task 4.4: Wire the hook into config.yaml

**Objective:** Add llm-clean.py as a post_llm_call hook.

**Files:**
- Modify: `~/.hermes/config.yaml`

**Step 1:** Replace the current `post_llm_call` section.

Current:
```yaml
  post_llm_call:
    - command: python3 /home/sikmindz/.hermes/agent-hooks/sm-autocapture.py
      timeout: 30
```

New:
```yaml
  post_llm_call:
    - command: python3 /home/sikmindz/.hermes/agent-hooks/llm-clean.py
      timeout: 10
    - command: python3 /home/sikmindz/.hermes/agent-hooks/sm-autocapture.py
      timeout: 30
```

Note: llm-clean runs BEFORE sm-autocapture so autocapture sees the cleaned output.

**Step 2:** Verify config.

```bash
python3 -c "import yaml; yaml.safe_load(open('$HOME/.hermes/config.yaml'))" && echo "config valid"
```

### Task 4.5: End-to-end verification

**Objective:** Confirm think blocks are stripped in a real session.

**Step 1:** Build and confirm the binary is installed.

```bash
which llm-parse && llm-parse strip <<< '<think>test</think>hello' 
```

Expected: `hello`.

**Step 2:** Run a Hermes session with a model that emits think blocks and confirm they don't appear in the final response.

---

## Phase 5: Final validation and commit

### Task 5.1: Full integration smoke test

**Objective:** Verify all three integrations work together without breaking anything.

**Step 1:** Verify all binaries are installed.

```bash
which cea-bridge knowledge-router llm-parse context-governor agent-graph-mcp
```

Expected: all 5 paths under `~/.cargo/bin/`.

**Step 2:** Verify config is valid and all hooks are present.

```bash
python3 -c "
import yaml
c = yaml.safe_load(open('$HOME/.hermes/config.yaml'))
hooks = c.get('hooks', {})
print('on_session_start:', len(hooks.get('on_session_start', [])))
print('pre_llm_call:', len(hooks.get('pre_llm_call', [])))
print('post_llm_call:', len(hooks.get('post_llm_call', [])))
print('pre_tool_call:', len(hooks.get('pre_tool_call', [])))
print('post_tool_call:', len(hooks.get('post_tool_call', [])))
"
```

Expected:
- `pre_llm_call: 2` (kr-classify + sm-recall)
- `post_llm_call: 2` (llm-clean + sm-autocapture)
- `post_tool_call: 1` (cea-telemetry)

**Step 3:** Verify knowledge-runtime tests still pass.

```bash
cd /home/sikmindz/Coding/Libraries
cargo test -p knowledge-runtime --lib 2>&1 | tail -3
```

Expected: `57 passed; 0 failed`.

**Step 4:** Verify llm-output-parser tests still pass.

```bash
cargo test -p llm-output-parser --lib 2>&1 | tail -3
```

Expected: `141 passed; 0 failed`.

### Task 5.2: Commit the llm-output-parser binary addition

**Objective:** Commit the new bin target to the Libraries repo.

```bash
cd /home/sikmindz/Coding/Libraries
git add llm-output-parser/Cargo.toml llm-output-parser/src/bin/llm-parse.rs
git commit -m "feat(llm-output-parser): add llm-parse CLI binary for Hermes hook integration"
```

### Task 5.3: Commit the knowledge-runtime fix

**Objective:** Commit the test fixes (if not already committed in Task 2.4).

```bash
cd /home/sikmindz/Coding/Libraries
git add knowledge-runtime/src/entity/code_ids.rs knowledge-runtime/src/entity/registry.rs
git commit -m "fix(bound-006): knowledge-runtime entity ID family prefix fixes — all 57 tests pass"
```

### Task 5.4: Document the integration

**Objective:** Create a short reference doc for what was wired.

**Files:**
- Create: `~/.hermes/agent-hooks/INTEGRATIONS.md`

```markdown
# Hermes ← Libraries Crate Integrations

## Active integrations

| Crate | Binary | Hook | Phase | Purpose |
|-------|--------|------|-------|---------|
| semantic-memory | semantic-memory-mcp | MCP + 5 hooks | session/pre/post_llm/pre_tool | Durable semantic memory with evidence-scored retrieval |
| agent-graph | agent-graph-mcp | MCP server | — | Graph-orchestrated LLM workflows |
| context-governor | context-governor | context engine | pre_llm (via adapter) | Governed context compaction with receipts |
| cea-bridge | cea-bridge | post_tool_call | after patch/write_file | Tool telemetry with advisory relevance scoring |
| knowledge-router | knowledge-router | pre_llm_call | before sm-recall | Query classification and retrieval route planning |
| llm-output-parser | llm-parse | post_llm_call | before sm-autocapture | Strip think blocks, extract clean JSON/text from LLM output |

## Fail-open policy

All hook scripts are fail-open. If a Rust binary is missing, times out, or errors,
the hook logs to stderr and returns exit 0. Hermes proceeds with its default behavior.
No hook can block the agent loop.

## Configuration

All hooks are wired in `~/.hermes/config.yaml` under the `hooks:` section.
Binary paths can be overridden via env vars:
- `CEA_BRIDGE_BIN` — path to cea-bridge binary
- `KNOWLEDGE_ROUTER_BIN` — path to knowledge-router binary
- `LLM_PARSE_BIN` — path to llm-parse binary
```

---

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Hook adds latency to every turn | All hooks have 10-15s timeouts; cea-bridge and llm-parse skip entirely if the payload doesn't match their filter |
| knowledge-router classification is wrong and degrades retrieval | sm-recall ignores the route file by default — it's advisory only. The route file is written but sm-recall doesn't read it yet (Phase 2 enhancement) |
| llm-clean strips content that isn't think blocks | The `<think>` tag check is explicit — only fires if `<think>` or `</think>` is in the output. No regex, no fuzzy matching |
| cea-bridge DB grows unbounded | Add a cleanup cron job later (Phase 2). For now, the DB is small (SQLite, single row per tool call) |
| knowledge-runtime fix uses wrong family prefix | The `family_name()` function is checked before applying fixes. Tests will fail immediately if the prefix is wrong |
| llm-parse binary breaks existing llm-output-parser lib tests | The bin target is separate from the lib — `cargo test -p llm-output-parser --lib` only runs lib tests |

## What this plan does NOT do

- Does not modify sm-recall to actually *read* the knowledge-router route file (that's a Phase 2 enhancement once we confirm the classification is useful)
- Does not modify Hermes core (all changes are in hooks and config)
- Does not add new model tools (everything is hook-based, invisible to the model's tool schema)
- Does not touch the semantic-memory MCP server or its config
- Does not resolve the unmerged semantic-memory working tree conflicts (separate task)
- Does not fix kernel-oracles/kernel-conformance test failures (separate task, different root cause)

## Open questions

1. **Should sm-recall be modified to read the route file?** The plan writes it but doesn't wire sm-recall to consume it. This is intentional — confirm the classification quality first, then wire it. If yes, modify sm-recall to read `~/.hermes/kr-last-route.json` and use the `classify.mode` to choose between hybrid search, entity search, or temporal search.

2. **Should cea-bridge hook also record browser actions?** The plan includes `browser_click` and `browser_type` in `RECORDED_TOOLS`. If this is too noisy, remove them.

3. **Should llm-parse also extract JSON for Hermes?** Currently it only strips think blocks. If Hermes has a JSON-extraction pain point, the hook could also call `llm-parse json` and write the parsed JSON back. But this is riskier — it could break non-JSON responses. Leave as think-strip only for now.