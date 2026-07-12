#!/usr/bin/env python3
"""
Prompt ablation harness for context-governor LLM summarization.

Builds a realistic multi-turn conversation, compacts it, renders different
prompt variants, calls the LLM for each, parses outputs, and scores them
on fact preservation, ref integrity, question queue, and structure.
"""

import json
import subprocess
import sys
import time
import os
import re
from pathlib import Path

OLLAMA_URL = "http://127.0.0.1:11434/v1/chat/completions"
CRATE_DIR = "/home/sikmindz/Coding/Libraries/context-governor"
RESULTS_DIR = "/home/sikmindz/Coding/Libraries/context-governor/prompt_ablation_results"

# ── Realistic conversation scenario ──────────────────────────────────────

# A multi-turn coding conversation with files, commands, errors, decisions,
# and unresolved questions. This is the kind of context that degrades badly
# under iterated generic summarization.

CONVERSATION = [
    {"role": "system", "content": "You are a coding assistant. Track all decisions and unresolved questions explicitly."},
    {"role": "user", "content": "I need to fix the authentication module in src/auth.rs. The login endpoint is returning 500 errors when users try to log in with email/password."},
    {"role": "assistant", "content": "I'll investigate the auth module. Let me look at the login handler in src/auth.rs. The issue is likely in the token validation logic around line 45-60. The function `validate_credentials` is calling `bcrypt::verify` with incorrect parameters."},
    {"role": "user", "content": "Yes, I see it. The password hash comparison is using `bcrypt::verify(password, hash)` but it should be `bcrypt::verify(hash, password)`. The argument order is reversed."},
    {"role": "assistant", "content": "Exactly. I'll fix the argument order in `validate_credentials` at line 52. Also, I notice the error handling doesn't catch `bcrypt::Error` properly — it's being mapped to a generic 500 instead of returning a 401 for invalid credentials."},
    {"role": "user", "content": "Good catch. Let's also add rate limiting to the login endpoint. We should use a token bucket approach with max 5 attempts per minute per IP."},
    {"role": "assistant", "content": "I'll implement rate limiting using `governor` crate. Here's the plan:\n1. Fix bcrypt::verify argument order in validate_credentials\n2. Add proper error mapping (401 for invalid creds, 500 for server errors)\n3. Add rate limiting with governor crate (5 attempts/min/IP)\n4. Add tests for all three changes\n\nLet me run the current tests first to see the baseline."},
    {"role": "user", "content": "Run cargo test and show me the output."},
    {"role": "assistant", "content": "Running cargo test...\n```\n$ cargo test auth\n   Compiling myapp v0.3.2\n    Finished test [unoptimized + debuginfo] target(s) in 2.1s\n     Running unittests src/auth.rs\n\ntest result: ok. 3 passed; 0 failed; 0 ignored\n\ntest auth::tests::test_login_with_valid_credentials ... ok\ntest auth::tests::test_login_with_invalid_password ... ok\ntest auth::tests::test_login_with_missing_user ... ok\n```"},
    {"role": "user", "content": "Now implement the fixes. Also, I decided we should use Argon2 instead of bcrypt for the password hashing. It's more secure."},
    {"role": "assistant", "content": "Understood. Decision: Switch from bcrypt to Argon2 for password hashing. I'll:\n1. Add `argon2` crate to Cargo.toml\n2. Replace `bcrypt::verify` with `argon2::verify` in validate_credentials\n3. Update the password hashing function to use Argon2\n4. Update tests to use Argon2 hashes\n\nOne question: should we migrate existing bcrypt hashes to Argon2, or support both during a transition period?"},
    {"role": "user", "content": "Support both during transition. We'll migrate users on next login."},
    {"role": "assistant", "content": "Got it. Unresolved question resolved: support both bcrypt and Argon2 during transition. I'll implement a `verify_password` function that tries Argon2 first, then falls back to bcrypt.\n\nLet me also check if the rate limiter needs a Redis backend for distributed deployments."},
    {"role": "user", "content": "Not yet, keep it in-memory for now. We can add Redis later if we need to scale horizontally."},
    {"role": "assistant", "content": "Decision: In-memory rate limiting for now, Redis backend deferred to future sprint.\n\nFiles modified so far:\n- src/auth.rs: Fixed bcrypt::verify argument order, added Argon2 support\n- Cargo.toml: Added argon2 and governor crates\n- src/auth.rs: Added verify_password function (Argon2 first, bcrypt fallback)\n- tests/auth_tests.rs: Added 5 new tests for Argon2, rate limiting, and error mapping\n\nRunning cargo test again...\n```\n$ cargo test\n   Compiling myapp v0.3.2\n    Finished test [unoptimized + debuginfo] target(s) in 3.4s\n\ntest result: ok. 8 passed; 0 failed; 0 ignored\n\nAll tests pass. Here's a summary of changes:\n- Fixed bcrypt::verify arg order (was reversed)\n- Added Argon2 support with bcrypt fallback for migration\n- Added rate limiting (5 attempts/min/IP via governor)\n- Added proper error mapping (401 for invalid creds, 500 for server errors)\n- 8 tests covering all changes\n```"},
    {"role": "user", "content": "Great. Now I'm concerned about the token expiry logic. JWT tokens expire after 1 hour but we're not refreshing them. Can you look into that?"},
    {"role": "assistant", "content": "I see the issue. The `validate_token` function in src/auth.rs line 78 checks expiry but doesn't handle refresh. We need:\n1. A refresh token endpoint\n2. Token rotation on refresh\n3. Short-lived access tokens (15 min) + long-lived refresh tokens (7 days)\n\nUnresolved question: Should we use JWT refresh tokens or opaque tokens?"},
    {"role": "user", "content": "Let's go with JWT refresh tokens for now. We can revisit if we need to revoke them."},
]


# ── Prompt variants ─────────────────────────────────────────────────────

PROMPT_VARIANTS = {
    "current": {
        "description": "Current 7-principle prompt (bitemporal, fallback refs, living queue, category compression, loss tracking, step graph, authority anchoring)",
        "system": None,  # Will be rendered by the Rust crate (now the merged prompt)
    },
    "minimal": {
        "description": "Minimal generic summarization prompt (baseline)",
        "system": "You are a conversation summarizer. Summarize the following conversation, preserving all important facts, decisions, and unresolved questions. Be concise but thorough.",
    },
    "structured_no_rules": {
        "description": "Structured output format only, no governing rules",
        "system": "You are a context summarizer. Output your summary in this exact format:\n\n=== ACTIVE TASK ===\n[one sentence]\n\n=== ACCEPTANCE GATES ===\n[list or None]\n\n=== FILES ===\n[path | state | pending action]\n\n=== COMMANDS ===\n[command | exit status | key output]\n\n=== ERRORS ===\n[error | blocks task? | context]\n\n=== DECISIONS ===\n[decision | rationale | timestamp]\n\n=== UNRESOLVED QUESTIONS ===\n[question]\n\n=== EXACT FALLBACK REFS ===\n[copy all ref IDs from input]\n\n=== SUMMARY LOSSES ===\n[what was dropped]\n\n=== PRIOR CONTEXT SUMMARY ===\n[dense factual summary]",
    },
    "rules_no_structure": {
        "description": "Seven rules but no output format constraint",
        "system": "You are a context compaction summarizer. Follow these rules strictly:\n\n1. BITEMPORAL FACT PRESERVATION: Keep (timestamp, actor, decision, outcome) tuples verbatim. Never rephrase or merge them.\n2. FALLBACK REF INTEGRITY: Never summarize or omit exact_fallback_ref IDs. Copy the full list unchanged.\n3. LIVING QUESTION QUEUE: Carry forward every unresolved question. Mark resolved only when explicit evidence appears.\n4. CATEGORY COMPRESSION: Compress files, commands, errors, and decisions independently, not as a monolithic block.\n5. EXPLICIT LOSS TRACKING: After your summary, list what was dropped or generalized.\n6. STEP GRAPH LINEAGE: Extend the prior plan state, do not replace it.\n7. AUTHORITY ANCHORING: The latest user message is the authority. Re-rank older summaries against the new focus.\n\nSummarize the following context.",
    },
    "old_current": {
        "description": "Previous 7-principle prompt (before merge with anti-degradation framing)",
        "system": "You are a context compaction summarizer operating inside a structured compaction pipeline. Your output will be re-fed into future compaction cycles, so you must preserve maximum signal per token.\n\n## CRITICAL RULES\n\n### 1. Bitemporal Fact Preservation\nKeep (timestamp, actor, decision, outcome) tuples VERBATIM. Never rephrase, merge, or paraphrase them. If a decision was made at a specific time by a specific actor, that fact must survive every compaction cycle intact.\n\n### 2. Fallback Ref Integrity\nNEVER summarize, rephrase, or omit `exact_fallback_ref` IDs. These are content-addressed retrieval keys. Losing a ref ID destroys the ability to recover exact original content. Copy the full list of ref IDs into your output unchanged.\n\n### 3. Living Question Queue\nCarry forward EVERY unresolved question. Mark a question as resolved ONLY when explicit evidence appears in new messages. Never drop an unresolved question. If you are unsure whether it is resolved, keep it as unresolved.\n\n### 4. Category Compression\nCompress each category independently, not as a monolithic block:\n- FILES: (path, last known state, pending action)\n- COMMANDS: (command, exit status, key output line)\n- ERRORS: verbatim if they block a task; otherwise (error type, context)\n- DECISIONS: (decision, rationale, timestamp)\n\n### 5. Explicit Loss Tracking\nAfter your summary, list what was dropped or generalized under SUMMARY LOSSES. Be specific.\n\n### 6. Step Graph Lineage\nUse the compaction lineage to EXTEND the prior plan state, not replace it.\n\n### 7. Authority Anchoring\nThe latest user message is the authority. Re-rank older summaries against the new focus. If the user's focus has shifted, demote older context that is no longer relevant, but do not drop it if it contains unresolved questions or active acceptance gates.\n\n## OUTPUT FORMAT\n\nYour output MUST follow this structure exactly:\n\n=== ACTIVE TASK ===\n[One sentence describing the current task]\n\n=== ACCEPTANCE GATES ===\n[List of active acceptance gates, or None]\n\n=== FILES ===\n[path | state | pending action]\n[One per line, or None]\n\n=== COMMANDS ===\n[command | exit status | key output]\n[One per line, or None]\n\n=== ERRORS ===\n[error | blocks task? | context]\n[One per line, or None]\n\n=== DECISIONS ===\n[decision | rationale | timestamp]\n[One per line, or None]\n\n=== UNRESOLVED QUESTIONS ===\n[question]\n[One per line, or None]\n\n=== EXACT FALLBACK REFS ===\n[Copy the full list of ref IDs from the input, one per line. NEVER omit any.]\n\n=== SUMMARY LOSSES ===\n[What was dropped or generalized in this pass, or None]\n\n=== PRIOR CONTEXT SUMMARY ===\n[Your compressed summary of prior context, organized by the categories above. This is the main body of your output. Be dense and factual. No filler. No 'The user discussed...'. Just the facts.]",
    },
}

# ── Scoring metrics ─────────────────────────────────────────────────────

def score_output(output: str, conversation: list) -> dict:
    """Score an LLM summarization output on multiple dimensions."""
    scores = {}

    # 1. Fact preservation: check for key facts from the conversation
    key_facts = [
        "auth.rs",
        "bcrypt::verify",
        "argon2",
        "governor",
        "rate limit",
        "5 attempts",
        "JWT",
        "refresh token",
        "validate_credentials",
        "token expiry",
        "Cargo.toml",
        "401",
        "500",
        "in-memory",
    ]
    output_lower = output.lower()
    facts_found = sum(1 for f in key_facts if f.lower() in output_lower)
    scores["fact_preservation"] = facts_found / len(key_facts)

    # 2. Fallback ref integrity: check that ref IDs are preserved
    # We'll count how many ref-like patterns appear in the output
    ref_pattern = r"exact-[a-zA-Z0-9_-]+"
    refs_in_output = len(re.findall(ref_pattern, output))
    scores["ref_integrity"] = refs_in_output  # raw count, higher is better

    # 3. Unresolved question preservation
    # The conversation has one unresolved question: "Should we use JWT refresh tokens or opaque tokens?"
    # and one resolved: "should we migrate existing bcrypt hashes to Argon2"
    unresolved_keywords = ["jwt", "refresh", "opaque"]
    has_unresolved = any(k in output_lower for k in unresolved_keywords)
    scores["unresolved_preserved"] = 1.0 if has_unresolved else 0.0

    # 4. Decision preservation
    decisions = [
        "argon2",
        "in-memory",
        "both",  # support both bcrypt and argon2 during transition
    ]
    decisions_found = sum(1 for d in decisions if d.lower() in output_lower)
    scores["decision_preservation"] = decisions_found / len(decisions)

    # 5. Structure compliance: does the output follow the section format?
    expected_sections = [
        "=== ACTIVE TASK ===",
        "=== FILES ===",
        "=== DECISIONS ===",
        "=== UNRESOLVED QUESTIONS ===",
    ]
    sections_found = sum(1 for s in expected_sections if s in output)
    scores["structure_compliance"] = sections_found / len(expected_sections)

    # 6. Loss tracking: does it explicitly list losses?
    has_loss_section = "=== SUMMARY LOSSES ===" in output
    loss_content = ""
    if has_loss_section:
        loss_match = re.search(r"=== SUMMARY LOSSES ===\n(.*?)(?:=== |$)", output, re.DOTALL)
        if loss_match:
            loss_content = loss_match.group(1).strip()
    has_specific_losses = has_loss_section and "None" not in loss_content and len(loss_content) > 10
    scores["loss_tracking"] = 1.0 if has_specific_losses else 0.0

    # 7. Information density: non-filler content ratio
    words = output.split()
    filler_words = {"the", "a", "an", "is", "was", "to", "and", "of", "in", "that", "it", "for", "on", "with", "as", "by"}
    non_filler = sum(1 for w in words if w.lower().strip(".,;:!?") not in filler_words)
    scores["info_density"] = non_filler / max(len(words), 1)

    # 8. Token efficiency: how compact is the summary (lower is better, but not too low)
    scores["output_tokens"] = len(output.split())

    # 9. Hallucination check: any facts NOT in the conversation?
    hallucination_indicators = [
        "redis",  # explicitly deferred, shouldn't appear as implemented
        "oauth",
        "session",
        "cookie",
    ]
    # Only count as hallucination if presented as implemented/used
    hallucinations = 0
    for h in hallucination_indicators:
        if h in output_lower:
            # Check context — if it says "deferred" or "future" it's ok
            if "defer" not in output_lower and "future" not in output_lower and "not yet" not in output_lower:
                hallucinations += 1
    scores["hallucinations"] = -hallucinations  # negative is bad

    # Overall composite score
    scores["composite"] = (
        scores["fact_preservation"] * 3.0
        + scores["ref_integrity"] * 0.5  # raw count, weighted low
        + scores["unresolved_preserved"] * 2.0
        + scores["decision_preservation"] * 2.0
        + scores["structure_compliance"] * 1.5
        + scores["loss_tracking"] * 1.0
        + scores["info_density"] * 1.0
        + scores["hallucinations"] * 2.0  # penalty
    )

    return scores


# ── LLM call helper ──────────────────────────────────────────────────────

def call_llm(model: str, system: str, user: str, max_tokens: int = 2000) -> str:
    """Call ollama API and return the assistant's content."""
    import urllib.request

    payload = json.dumps({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "max_tokens": max_tokens,
        "temperature": 0.3,  # low temp for factual summarization
    }).encode("utf-8")

    req = urllib.request.Request(OLLAMA_URL, data=payload, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            content = data["choices"][0]["message"].get("content", "")
            reasoning = data["choices"][0]["message"].get("reasoning", "")
            # Some models put everything in reasoning; combine if content is empty
            if not content and reasoning:
                # Try to extract just the final answer from reasoning
                # Look for the last section that looks like actual output
                content = reasoning
            return content
    except Exception as e:
        return f"ERROR: {e}"


# ── Rust bridge: render prompt via cargo test ───────────────────────────

def get_compacted_context():
    """Run a Rust test that compacts the conversation and renders the user prompt."""
    # We'll write a temporary Rust test that serializes the compacted response
    # and the rendered prompt to JSON, then run it and capture stdout.
    test_code = '''
use context_governor::*;
use serde_json;

fn main() {
    let messages: Vec<Message> = serde_json::from_str(include_str!("../prompt_ablation_conversation.json")).unwrap();
    let request = CompactRequest {
        session_id: "ablation-test".to_string(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 2000,
            allocator: AllocatorMode::UtilityV2.as_str().to_string(),
            budget_mode: BudgetMode::SoftWarn,
            ..Default::default()
        },
        focus: Some("auth fix JWT refresh token".to_string()),
    };
    let response = compact_context(request).unwrap();

    // Render with current prompt
    let config = PromptConfigV1::default();
    let prompt = render_summary_prompt(&response, &[], &config);

    // Also inject exact_store items for ref testing
    let mut response_with_refs = response.clone();
    response_with_refs.exact_store.push(ExactStoredItemV1 {
        item_id: "exact-auth-rs".to_string(),
        source_indices: vec![1],
        content: "src/auth.rs login handler validate_credentials bcrypt::verify".to_string(),
        content_blake3: "abc123".to_string(),
    });
    response_with_refs.exact_store.push(ExactStoredItemV1 {
        item_id: "exact-cargo-toml".to_string(),
        source_indices: vec![7],
        content: "Cargo.toml argon2 governor dependencies".to_string(),
        content_blake3: "def456".to_string(),
    });
    response_with_refs.exact_store.push(ExactStoredItemV1 {
        item_id: "exact-test-output".to_string(),
        source_indices: vec![9],
        content: "cargo test auth 3 passed 0 failed 8 passed 0 failed".to_string(),
        content_blake3: "ghi789".to_string(),
    });
    response_with_refs.receipt.exact_fallback_refs.push(ExactFallbackRefV1 {
        item_id: "exact-auth-rs".to_string(),
        start_index: 1,
        end_index: 2,
        content_blake3: "abc123".to_string(),
        approx_tokens: 10,
    });
    response_with_refs.receipt.exact_fallback_refs.push(ExactFallbackRefV1 {
        item_id: "exact-cargo-toml".to_string(),
        start_index: 7,
        end_index: 8,
        content_blake3: "def456".to_string(),
        approx_tokens: 8,
    });
    response_with_refs.receipt.exact_fallback_refs.push(ExactFallbackRefV1 {
        item_id: "exact-test-output".to_string(),
        start_index: 9,
        end_index: 10,
        content_blake3: "ghi789".to_string(),
        approx_tokens: 12,
    });

    let prompt_with_refs = render_summary_prompt(&response_with_refs, &[], &config);

    let output = serde_json::json!({
        "system_prompt": prompt_with_refs.system,
        "user_prompt": prompt_with_refs.user,
        "compact_response": response_with_refs,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
'''
    # Write the conversation JSON
    conv_path = os.path.join(CRATE_DIR, "prompt_ablation_conversation.json")
    with open(conv_path, "w") as f:
        json.dump(CONVERSATION, f, indent=2)

    # Write the example binary
    example_dir = os.path.join(CRATE_DIR, "examples")
    os.makedirs(example_dir, exist_ok=True)
    example_path = os.path.join(example_dir, "prompt_ablation.rs")
    with open(example_path, "w") as f:
        f.write(test_code)

    # Run it
    result = subprocess.run(
        ["cargo", "run", "--example", "prompt_ablation", "--quiet"],
        capture_output=True, text=True, cwd=CRATE_DIR, timeout=120
    )

    # Clean up
    os.remove(conv_path)
    os.remove(example_path)

    if result.returncode != 0:
        print(f"Rust build error: {result.stderr[:500]}")
        sys.exit(1)

    return json.loads(result.stdout.strip())


# ── Main ─────────────────────────────────────────────────────────────────

def main():
    models = ["glm-5.2:cloud", "kimi-k2.7-code:cloud", "deepseek-v4-flash:cloud"]

    print("=== Prompt Ablation Harness ===")
    print(f"Models: {models}")
    print(f"Variants: {list(PROMPT_VARIANTS.keys())}")
    print()

    # Get compacted context and rendered prompts from Rust
    print("Compacting conversation via Rust crate...")
    data = get_compacted_context()
    system_prompt_current = data["system_prompt"]
    user_prompt = data["user_prompt"]
    print(f"System prompt: {len(system_prompt_current)} chars")
    print(f"User prompt: {len(user_prompt)} chars")
    print()

    os.makedirs(RESULTS_DIR, exist_ok=True)

    all_results = {}

    for model in models:
        print(f"\n{'='*60}")
        print(f"Model: {model}")
        print(f"{'='*60}")

        all_results[model] = {}

        for variant_name, variant in PROMPT_VARIANTS.items():
            print(f"\n--- {variant_name} ---")
            print(f"  {variant['description'][:80]}")

            if variant["system"] is None:
                # Use the current crate-rendered system prompt
                system = system_prompt_current
            else:
                system = variant["system"]

            t0 = time.time()
            output = call_llm(model, system, user_prompt, max_tokens=2000)
            elapsed = time.time() - t0

            # Clean up reasoning prefix if present
            # Some models output thinking process; try to extract actual output
            if "=== ACTIVE TASK ===" not in output and "=== ACTIVE TASK ===" in output.replace("\\n", "\n"):
                output = output.replace("\\n", "\n")

            scores = score_output(output, CONVERSATION)

            print(f"  Time: {elapsed:.1f}s")
            print(f"  Output tokens: {scores['output_tokens']}")
            print(f"  Composite score: {scores['composite']:.3f}")
            print(f"  Fact preservation: {scores['fact_preservation']:.2f}")
            print(f"  Ref integrity: {scores['ref_integrity']}")
            print(f"  Unresolved preserved: {scores['unresolved_preserved']:.1f}")
            print(f"  Decision preserved: {scores['decision_preservation']:.2f}")
            print(f"  Structure compliance: {scores['structure_compliance']:.2f}")
            print(f"  Loss tracking: {scores['loss_tracking']:.1f}")
            print(f"  Info density: {scores['info_density']:.2f}")
            print(f"  Hallucinations: {scores['hallucinations']}")

            # Save raw output
            result = {
                "model": model,
                "variant": variant_name,
                "description": variant["description"],
                "elapsed_seconds": elapsed,
                "output": output,
                "scores": scores,
            }
            all_results[model][variant_name] = result

            # Save individual output
            outpath = os.path.join(RESULTS_DIR, f"{model.replace(':', '_')}_{variant_name}.txt")
            with open(outpath, "w") as f:
                f.write(f"Model: {model}\nVariant: {variant_name}\nTime: {elapsed:.1f}s\n\n--- SYSTEM PROMPT ---\n{system}\n\n--- USER PROMPT ---\n{user_prompt}\n\n--- OUTPUT ---\n{output}\n\n--- SCORES ---\n{json.dumps(scores, indent=2)}\n")

    # Summary table
    print(f"\n\n{'='*80}")
    print("SUMMARY TABLE")
    print(f"{'='*80}")
    print(f"{'Model':<25} {'Variant':<25} {'Composite':>10} {'Facts':>8} {'Refs':>6} {'Unres':>8} {'Decis':>8} {'Struct':>8} {'Loss':>6} {'Dens':>8} {'Halluc':>8} {'Tokens':>8}")
    print("-" * 120)

    for model in models:
        for variant_name in PROMPT_VARIANTS:
            r = all_results[model][variant_name]
            s = r["scores"]
            print(f"{model:<25} {variant_name:<25} {s['composite']:>10.3f} {s['fact_preservation']:>8.2f} {s['ref_integrity']:>6} {s['unresolved_preserved']:>8.1f} {s['decision_preservation']:>8.2f} {s['structure_compliance']:>8.2f} {s['loss_tracking']:>6.1f} {s['info_density']:>8.2f} {s['hallucinations']:>8} {s['output_tokens']:>8}")

    # Save full results JSON
    results_path = os.path.join(RESULTS_DIR, "full_results.json")
    with open(results_path, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\nFull results saved to {results_path}")


if __name__ == "__main__":
    main()