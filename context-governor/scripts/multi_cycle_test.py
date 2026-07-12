#!/usr/bin/env python3
"""
Multi-cycle iterated compaction test.

Simulates 4 compaction cycles with new messages arriving between each.
Measures how well the merged prompt preserves facts, refs, and unresolved
questions across cycles, compared to the minimal (generic) prompt.

This is the real test of the anti-degradation framing: does the merged prompt
hold up when the LLM's own output is fed back as input for the next cycle?
"""

import json
import subprocess
import sys
import time
import os
import re
import urllib.request

OLLAMA_URL = "http://127.0.0.1:11434/v1/chat/completions"
CRATE_DIR = "/home/sikmindz/Coding/Libraries/context-governor"
RESULTS_DIR = os.path.join(CRATE_DIR, "multi_cycle_results")

# ── Conversation phases (new messages added between each cycle) ──────────

CYCLE_MESSAGES = [
    # Cycle 0: initial conversation (same as ablation)
    [
        {"role": "system", "content": "You are a coding assistant. Track all decisions and unresolved questions explicitly."},
        {"role": "user", "content": "I need to fix the authentication module in src/auth.rs. The login endpoint is returning 500 errors when users try to log in with email/password."},
        {"role": "assistant", "content": "I'll investigate the auth module. The issue is likely in the token validation logic around line 45-60. The function `validate_credentials` is calling `bcrypt::verify` with incorrect parameters."},
        {"role": "user", "content": "Yes, the password hash comparison is using `bcrypt::verify(password, hash)` but it should be `bcrypt::verify(hash, password)`. The argument order is reversed."},
        {"role": "assistant", "content": "Exactly. I'll fix the argument order in `validate_credentials` at line 52. Also, I notice the error handling doesn't catch `bcrypt::Error` properly — it's being mapped to a generic 500 instead of returning a 401 for invalid credentials."},
        {"role": "user", "content": "Good catch. Let's also add rate limiting to the login endpoint. We should use a token bucket approach with max 5 attempts per minute per IP."},
        {"role": "assistant", "content": "I'll implement rate limiting using `governor` crate. Let me run the current tests first to see the baseline."},
        {"role": "user", "content": "Run cargo test and show me the output."},
        {"role": "assistant", "content": "Running cargo test...\n$ cargo test auth\ntest result: ok. 3 passed; 0 failed; 0 ignored"},
        {"role": "user", "content": "Now implement the fixes. Also, I decided we should use Argon2 instead of bcrypt for the password hashing."},
        {"role": "assistant", "content": "Decision: Switch from bcrypt to Argon2. One question: should we migrate existing bcrypt hashes to Argon2, or support both during a transition period?"},
        {"role": "user", "content": "Support both during transition. We'll migrate users on next login."},
        {"role": "assistant", "content": "Got it. I'll implement a verify_password function that tries Argon2 first, then falls back to bcrypt. Running cargo test again...\n$ cargo test\ntest result: ok. 8 passed; 0 failed; 0 ignored"},
        {"role": "user", "content": "Great. Now I'm concerned about the token expiry logic. JWT tokens expire after 1 hour but we're not refreshing them."},
        {"role": "assistant", "content": "I see the issue. validate_token in src/auth.rs line 78 checks expiry but doesn't handle refresh. Unresolved question: Should we use JWT refresh tokens or opaque tokens?"},
        {"role": "user", "content": "Let's go with JWT refresh tokens for now. We can revisit if we need to revoke them."},
    ],
    # Cycle 1: new messages after first compaction
    [
        {"role": "user", "content": "I also want to add logging to the auth module. We need to log all login attempts, both successful and failed, with timestamp and IP address."},
        {"role": "assistant", "content": "I'll add structured logging using the `tracing` crate. We need:\n1. Log every login attempt (success/failure, IP, timestamp)\n2. Log rate limit triggers\n3. Log token refresh events\n\nUnresolved question: Should logs go to stdout, a file, or a structured log aggregator like Loki?"},
        {"role": "user", "content": "Start with stdout for now, we can add a log aggregator later. Make sure failed login attempts include the reason (invalid password, user not found, rate limited)."},
        {"role": "assistant", "content": "I'll add tracing spans for the login flow:\n- `auth::login` span with user_id, ip, timestamp\n- `auth::login_failed` event with reason enum (InvalidPassword, UserNotFound, RateLimited)\n- `auth::token_refreshed` event with user_id\n\nFiles modified: src/auth.rs (added tracing instrumentation), Cargo.toml (added tracing and tracing-subscriber crates)."},
    ],
    # Cycle 2: more new messages
    [
        {"role": "user", "content": "The rate limiter is too aggressive. Some legitimate users are getting blocked. Can we increase it to 10 attempts per minute and add a cooldown period of 30 seconds after 5 failed attempts?"},
        {"role": "assistant", "content": "I'll adjust the rate limiter:\n- Increase from 5 to 10 attempts per minute per IP\n- After 5 failed attempts, enforce a 30-second cooldown\n- The cooldown resets on a successful login\n\nThis requires a more sophisticated state machine than the basic governor token bucket. I'll implement a custom RateLimitState enum: Normal, Cooldown { until: Instant, attempts: u32 }.\n\nUnresolved question: Should the cooldown be per-IP or per-user? If an attacker is behind NAT, per-IP could block legitimate users sharing the same IP."},
        {"role": "user", "content": "Make it per-user, not per-IP. We don't want to block everyone behind the same NAT."},
        {"role": "assistant", "content": "Updated: cooldown is per-user. Rate limit is still per-IP (10/min) but the cooldown after 5 failures is per-user. Files modified: src/auth.rs (RateLimitState enum, updated rate_limit function), tests/auth_tests.rs (3 new tests for cooldown behavior)."},
    ],
    # Cycle 3: final new messages
    [
        {"role": "user", "content": "We need to add a password reset flow. Users should be able to request a reset link via email, and the link should expire after 15 minutes."},
        {"role": "assistant", "content": "I'll implement the password reset flow:\n1. POST /auth/reset-request — accepts email, sends reset link with signed token (JWT, 15 min expiry)\n2. POST /auth/reset-confirm — accepts token + new password, validates token, updates password hash\n3. The reset token is a JWT signed with a separate key from the access/refresh tokens\n\nUnresolved question: Should we invalidate all existing sessions (access + refresh tokens) when a password is reset?"},
        {"role": "user", "content": "Yes, invalidate all existing sessions on password reset. Also, log the reset event with tracing."},
        {"role": "assistant", "content": "Done. Password reset flow implemented in src/auth.rs:\n- reset_request: generates JWT reset token (15 min), sends via email\n- reset_confirm: validates token, updates password, revokes all existing sessions\n- Added tracing events for reset request, confirm, and session revocation\n- Added 4 new tests (total now 15 tests, all passing)\n\nFiles modified: src/auth.rs (reset flow + session revocation), Cargo.toml (no new deps, using existing jsonwebtoken), tests/auth_tests.rs (4 new tests)."},
    ],
]

# Key facts that MUST survive all 4 cycles
KEY_FACTS = [
    "auth.rs",
    "bcrypt::verify",
    "argon2",
    "governor",
    "validate_credentials",
    "JWT",
    "refresh token",
    "token expiry",
    "401",
    "500",
    "in-memory",
    "tracing",
    "rate limit",
    "cooldown",
    "30 second",
    "password reset",
    "15 min",
    "invalidate",
    "session",
]

# Refs that must survive (injected in cycle 0)
EXPECTED_REFS = ["exact-auth-rs", "exact-cargo-toml", "exact-test-output"]

# Unresolved questions that appear across cycles
UNRESOLVED_QUESTIONS = [
    "log aggregator",  # from cycle 1: stdout vs Loki
    "per-IP",  # from cycle 2: cooldown per-IP vs per-user (resolved to per-user)
]


def call_llm(model, system, user, max_tokens=3000):
    payload = json.dumps({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "max_tokens": max_tokens,
        "temperature": 0.3,
    }).encode("utf-8")
    req = urllib.request.Request(OLLAMA_URL, data=payload, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            content = data["choices"][0]["message"].get("content", "")
            reasoning = data["choices"][0]["message"].get("reasoning", "")
            if not content and reasoning:
                content = reasoning
            # Post-process: if the output has === sections but also has
            # reasoning text before the first ===, strip everything before
            # the first === section header
            first_section = content.find("=== ")
            if first_section > 0:
                # Check if there's actual structured content after the preamble
                rest = content[first_section:]
                if "=== ACTIVE TASK ===" in rest and "=== PRIOR CONTEXT SUMMARY ===" in rest:
                    content = rest
            return content
    except Exception as e:
        return f"ERROR: {e}"


def extract_refs_from_section(output):
    """Extract ref IDs only from the EXACT FALLBACK REFS section."""
    ref_section = re.search(r'=== EXACT FALLBACK REFS ===\n(.*?)(?:=== |\Z)', output, re.DOTALL)
    if not ref_section:
        return []
    text = ref_section.group(1)
    return re.findall(r'exact-[a-zA-Z0-9_-]+|ctxi_[a-zA-Z0-9_]+', text)


def score_cycle(output, cycle_num):
    """Score a single cycle's output."""
    output_lower = output.lower()
    
    # Fact preservation
    facts_found = sum(1 for f in KEY_FACTS if f.lower() in output_lower)
    fact_pct = facts_found / len(KEY_FACTS)
    
    # Ref integrity (only from proper section)
    refs = extract_refs_from_section(output)
    refs_found = sum(1 for r in EXPECTED_REFS if r in refs)
    ref_pct = refs_found / len(EXPECTED_REFS)
    
    # Structure compliance
    expected_sections = ["=== ACTIVE TASK ===", "=== FILES ===", "=== DECISIONS ===", "=== UNRESOLVED QUESTIONS ==="]
    sections_found = sum(1 for s in expected_sections if s in output)
    struct_pct = sections_found / len(expected_sections)
    
    # Loss tracking
    has_loss = "=== SUMMARY LOSSES ===" in output
    loss_match = re.search(r'=== SUMMARY LOSSES ===\n(.*?)(?:=== |\Z)', output, re.DOTALL)
    has_specific_loss = has_loss and loss_match and "None" not in loss_match.group(1).strip() and len(loss_match.group(1).strip()) > 10
    
    # Token count
    tokens = len(output.split())
    
    return {
        "cycle": cycle_num,
        "fact_preservation": fact_pct,
        "facts_found": facts_found,
        "facts_total": len(KEY_FACTS),
        "ref_integrity": ref_pct,
        "refs_found": refs_found,
        "refs_total": len(EXPECTED_REFS),
        "structure_compliance": struct_pct,
        "loss_tracking": 1.0 if has_specific_loss else 0.0,
        "output_tokens": tokens,
    }


def get_compacted_response(messages, focus):
    """Run the Rust compactor on messages and return the compacted response as JSON."""
    conv_json = json.dumps(messages)
    conv_path = os.path.join(CRATE_DIR, "multi_cycle_conversation.json")
    with open(conv_path, "w") as f:
        f.write(conv_json)
    
    test_code = '''
use context_governor::*;
use serde_json;

fn main() {
    let messages: Vec<Message> = serde_json::from_str(include_str!("../multi_cycle_conversation.json")).unwrap();
    let request = CompactRequest {
        session_id: "multi-cycle-test".to_string(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 2000,
            allocator: AllocatorMode::UtilityV2.as_str().to_string(),
            budget_mode: BudgetMode::SoftWarn,
            ..Default::default()
        },
        focus: Some("auth module fix and enhancement".to_string()),
    };
    let mut response = compact_context(request).unwrap();

    // Inject exact_store items for ref tracking
    response.exact_store.push(ExactStoredItemV1 {
        item_id: "exact-auth-rs".to_string(),
        source_indices: vec![1],
        content: "src/auth.rs login handler validate_credentials bcrypt::verify".to_string(),
        content_blake3: "abc123".to_string(),
    });
    response.exact_store.push(ExactStoredItemV1 {
        item_id: "exact-cargo-toml".to_string(),
        source_indices: vec![7],
        content: "Cargo.toml argon2 governor tracing dependencies".to_string(),
        content_blake3: "def456".to_string(),
    });
    response.exact_store.push(ExactStoredItemV1 {
        item_id: "exact-test-output".to_string(),
        source_indices: vec![9],
        content: "cargo test auth 3 passed 8 passed 15 passed".to_string(),
        content_blake3: "ghi789".to_string(),
    });
    response.receipt.exact_fallback_refs.push(ExactFallbackRefV1 {
        item_id: "exact-auth-rs".to_string(),
        start_index: 1, end_index: 2,
        content_blake3: "abc123".to_string(),
        approx_tokens: 10,
    });
    response.receipt.exact_fallback_refs.push(ExactFallbackRefV1 {
        item_id: "exact-cargo-toml".to_string(),
        start_index: 7, end_index: 8,
        content_blake3: "def456".to_string(),
        approx_tokens: 8,
    });
    response.receipt.exact_fallback_refs.push(ExactFallbackRefV1 {
        item_id: "exact-test-output".to_string(),
        start_index: 9, end_index: 10,
        content_blake3: "ghi789".to_string(),
        approx_tokens: 12,
    });

    let config = PromptConfigV1::default();
    let prompt = render_summary_prompt(&response, &[], &config);
    let output = serde_json::json!({
        "system_prompt": prompt.system,
        "user_prompt": prompt.user,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
'''
    example_path = os.path.join(CRATE_DIR, "examples", "multi_cycle_render.rs")
    os.makedirs(os.path.dirname(example_path), exist_ok=True)
    with open(example_path, "w") as f:
        f.write(test_code)
    
    result = subprocess.run(
        ["cargo", "run", "--example", "multi_cycle_render", "--quiet"],
        capture_output=True, text=True, cwd=CRATE_DIR, timeout=120
    )
    os.remove(conv_path)
    os.remove(example_path)
    
    if result.returncode != 0:
        print(f"Rust error: {result.stderr[:500]}")
        sys.exit(1)
    
    return json.loads(result.stdout.strip())


def run_multi_cycle(model, prompt_variant, prompt_system):
    """Run 4 compaction cycles with the given prompt variant."""
    os.makedirs(RESULTS_DIR, exist_ok=True)
    
    all_scores = []
    all_outputs = []
    
    # The "running context" — starts as cycle 0 messages, then accumulates
    # the LLM's summary as prior context for subsequent cycles
    accumulated_context = ""
    
    for cycle_idx, new_messages in enumerate(CYCLE_MESSAGES):
        cycle_num = cycle_idx + 1
        print(f"  Cycle {cycle_num}...")
        
        if cycle_idx == 0:
            # First cycle: compact the original conversation
            data = get_compacted_response(new_messages, "auth module fix")
            system_prompt = prompt_system if prompt_system else data["system_prompt"]
            user_prompt = data["user_prompt"]
        else:
            # Subsequent cycles: use accumulated context as the "compacted messages"
            # and add new messages on top
            all_messages = []
            
            # Build a synthetic conversation: system + prior summary + new messages
            all_messages.append({
                "role": "system",
                "content": "You are a coding assistant. Track all decisions and unresolved questions explicitly.",
            })
            # The accumulated context from previous LLM output becomes a system-level summary
            all_messages.append({
                "role": "assistant", 
                "content": f"[PRIOR CONTEXT SUMMARY]\n{accumulated_context}",
            })
            # Add new messages
            all_messages.extend(new_messages)
            
            data = get_compacted_response(all_messages, "auth module fix and enhancement")
            system_prompt = prompt_system if prompt_system else data["system_prompt"]
            user_prompt = data["user_prompt"]
        
        # Call LLM
        t0 = time.time()
        llm_output = call_llm(model, system_prompt, user_prompt, max_tokens=2000)
        elapsed = time.time() - t0
        
        # Score
        score = score_cycle(llm_output, cycle_num)
        score["elapsed"] = elapsed
        score["model"] = model
        score["variant"] = prompt_variant
        all_scores.append(score)
        all_outputs.append(llm_output)
        
        print(f"    Facts: {score['facts_found']}/{score['facts_total']} ({score['fact_preservation']:.0%})  "
              f"Refs: {score['refs_found']}/{score['refs_total']} ({score['ref_integrity']:.0%})  "
              f"Struct: {score['structure_compliance']:.0%}  "
              f"Loss: {score['loss_tracking']:.0%}  "
              f"Tokens: {score['output_tokens']}")
        
        # Accumulate context for next cycle
        accumulated_context = llm_output
    
    # Save outputs
    outpath = os.path.join(RESULTS_DIR, f"{model.replace(':', '_')}_{prompt_variant}.txt")
    with open(outpath, "w") as f:
        for i, (score, output) in enumerate(zip(all_scores, all_outputs)):
            f.write(f"=== CYCLE {i+1} ===\n")
            f.write(f"Score: {json.dumps(score, indent=2)}\n\n")
            f.write(f"--- OUTPUT ---\n{output}\n\n")
            f.write("=" * 80 + "\n\n")
    
    return all_scores, all_outputs


def main():
    models = ["glm-5.2:cloud", "kimi-k2.7-code:cloud", "deepseek-v4-flash:cloud"]
    
    # We test two variants: merged (current crate prompt) and minimal (baseline)
    variants = {
        "merged": None,  # Use crate-rendered system prompt (the merged anti-degradation prompt)
        "minimal": "You are a conversation summarizer. Summarize the following conversation, preserving all important facts, decisions, and unresolved questions. Be concise but thorough.",
    }
    
    print("=== Multi-Cycle Iterated Compaction Test ===")
    print(f"Models: {models}")
    print(f"Variants: {list(variants.keys())}")
    print(f"Cycles: {len(CYCLE_MESSAGES)}")
    print()
    
    all_results = {}
    
    for model in models:
        all_results[model] = {}
        for variant_name, system_override in variants.items():
            print(f"\n{'='*60}")
            print(f"Model: {model} | Variant: {variant_name}")
            print(f"{'='*60}")
            
            scores, outputs = run_multi_cycle(model, variant_name, system_override)
            all_results[model][variant_name] = {"scores": scores, "outputs": outputs}
    
    # Summary table
    print(f"\n\n{'='*100}")
    print("MULTI-CYCLE RESULTS")
    print(f"{'='*100}")
    print(f"{'Model':<25} {'Variant':<10} {'Cycle':>6} {'Facts':>12} {'Refs':>12} {'Struct':>8} {'Loss':>6} {'Tokens':>8}")
    print("-" * 90)
    
    for model in models:
        for variant in variants:
            scores = all_results[model][variant]["scores"]
            for s in scores:
                print(f"{model:<25} {variant:<10} {s['cycle']:>6} "
                      f"{s['facts_found']:>3}/{s['facts_total']:<3} ({s['fact_preservation']:.0%})  "
                      f"{s['refs_found']:>3}/{s['refs_total']:<3} ({s['ref_integrity']:.0%})  "
                      f"{s['structure_compliance']:>8.0%} "
                      f"{s['loss_tracking']:>6.0%} "
                      f"{s['output_tokens']:>8}")
            # Average across cycles
            avg_facts = sum(s['fact_preservation'] for s in scores) / len(scores)
            avg_refs = sum(s['ref_integrity'] for s in scores) / len(scores)
            avg_struct = sum(s['structure_compliance'] for s in scores) / len(scores)
            avg_loss = sum(s['loss_tracking'] for s in scores) / len(scores)
            print(f"{'':>25} {'AVG':>10} {'':>6} {'':>12} {'':>12} {'':>8} {'':>6} {'':>8}")
            print(f"{'':>25} {'':>10} {'':>6} {avg_facts:>11.0%}  {avg_refs:>11.0%}  {avg_struct:>7.0%} {avg_loss:>5.0%}")
            print()
    
    # Degradation analysis
    print(f"\n{'='*100}")
    print("DEGRADATION ANALYSIS (Cycle 1 vs Cycle 4)")
    print(f"{'='*100}")
    for model in models:
        for variant in variants:
            scores = all_results[model][variant]["scores"]
            first = scores[0]
            last = scores[-1]
            fact_delta = last['fact_preservation'] - first['fact_preservation']
            ref_delta = last['ref_integrity'] - first['ref_integrity']
            print(f"{model:<25} {variant:<10}  "
                  f"Fact: {first['fact_preservation']:.0%} -> {last['fact_preservation']:.0%} ({fact_delta:+.0%})  "
                  f"Ref: {first['ref_integrity']:.0%} -> {last['ref_integrity']:.0%} ({ref_delta:+.0%})")
    
    # Save full results
    results_path = os.path.join(RESULTS_DIR, "full_results.json")
    # Convert sets to lists for JSON
    serializable = {}
    for model in all_results:
        serializable[model] = {}
        for variant in all_results[model]:
            serializable[model][variant] = {
                "scores": all_results[model][variant]["scores"],
            }
    with open(results_path, "w") as f:
        json.dump(serializable, f, indent=2)
    print(f"\nFull results saved to {results_path}")


if __name__ == "__main__":
    main()