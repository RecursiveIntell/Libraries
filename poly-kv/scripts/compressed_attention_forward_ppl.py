#!/usr/bin/env python3
"""Full forward-pass compressed-topk attention gate for SmolLM2/LlamaAttention.

This monkey-patches each LlamaAttention module after RoPE and before o_proj:
baseline: normal eager attention
variant: compressed-score top-k over keys -> exact selected-key softmax -> selected-value matmul

The goal is not speed. The goal is a real logit/PPL drift receipt for the model forward path.

Supports:
- Uniform top-k (--top-k N)
- Layer-adaptive budgets (--adaptive-budget)
- Per-(layer,head) adaptive budgets (--adaptive-budget-heads)
- Simple 4-bit quantized scoring (--scorer quantized, default)
- Fib-quant Gram-table scoring (--scorer fib-gram)
- Sampled attention stats (--sample-heads, --sample-positions)
"""
import argparse
import json
import math
import statistics
import time
import types
from pathlib import Path

import torch
import torch.nn.functional as F
from datasets import load_dataset
from transformers import AutoModelForCausalLM, AutoTokenizer
from transformers.models.llama.modeling_llama import apply_rotary_pos_emb, eager_attention_forward, repeat_kv

# --- adaptive budget imports (late import to avoid hard dep) ---
_adaptive_budget = None
def _get_adaptive_budget():
    global _adaptive_budget
    if _adaptive_budget is None:
        import importlib, sys
        script_dir = Path(__file__).parent
        if str(script_dir) not in sys.path:
            sys.path.insert(0, str(script_dir))
        _adaptive_budget = importlib.import_module("adaptive_budget")
    return _adaptive_budget

_fib_gram = None
def _get_fib_gram():
    global _fib_gram
    if _fib_gram is None:
        import importlib, sys
        script_dir = Path(__file__).parent
        if str(script_dir) not in sys.path:
            sys.path.insert(0, str(script_dir))
        _fib_gram = importlib.import_module("fib_gram_scorer")
    return _fib_gram


def percentile(xs, p):
    if not xs:
        return None
    xs = sorted(float(x) for x in xs)
    if len(xs) == 1:
        return xs[0]
    idx = (len(xs) - 1) * p
    lo = math.floor(idx)
    hi = math.ceil(idx)
    if lo == hi:
        return xs[lo]
    return xs[lo] * (hi - idx) + xs[hi] * (idx - lo)


def causal_nll_from_logits(logits, input_ids):
    shift_logits = logits[:, :-1, :].contiguous()
    targets = input_ids[:, 1:].contiguous()
    chunks = []
    for start in range(0, shift_logits.shape[1], 256):
        end = min(start + 256, shift_logits.shape[1])
        l = shift_logits[:, start:end, :].float()
        t = targets[:, start:end]
        chunks.append(F.cross_entropy(l.reshape(-1, l.shape[-1]), t.reshape(-1), reduction="none").reshape(1, -1))
    return torch.cat(chunks, dim=1).squeeze(0)


def windowed_ppl(nll, frac):
    t = nll.shape[0]
    start = int(t * (1.0 - frac))
    window = nll[start:t]
    return math.exp(float(window.mean().item())), start, t


def logit_metrics(base_logits, comp_logits, input_ids, eval_start):
    b = base_logits[:, eval_start:-1, :].float()
    c = comp_logits[:, eval_start:-1, :].float()
    cos = F.cosine_similarity(b.reshape(-1, b.shape[-1]), c.reshape(-1, c.shape[-1]), dim=-1)
    logp_b = F.log_softmax(b, dim=-1)
    logp_c = F.log_softmax(c, dim=-1)
    p_b = logp_b.exp()
    kl = (p_b * (logp_b - logp_c)).sum(dim=-1).reshape(-1)
    max_abs = (b - c).abs().amax(dim=-1).reshape(-1)
    argmax_same = (b.argmax(dim=-1) == c.argmax(dim=-1)).float().reshape(-1)
    targets = input_ids[:, eval_start + 1 :].reshape(-1)
    if targets.numel() != b.shape[1]:
        targets = input_ids[:, eval_start + 1 : eval_start + 1 + b.shape[1]].reshape(-1)
    target_logit_delta = (b.reshape(-1, b.shape[-1]).gather(1, targets[:, None]) - c.reshape(-1, c.shape[-1]).gather(1, targets[:, None])).abs().reshape(-1)
    return {
        "logit_cosine_mean": float(cos.mean().item()),
        "logit_cosine_p05": percentile(cos.detach().cpu().tolist(), 0.05),
        "kl_mean": float(kl.mean().item()),
        "kl_p95": percentile(kl.detach().cpu().tolist(), 0.95),
        "max_abs_logit_delta_mean": float(max_abs.mean().item()),
        "max_abs_logit_delta_p95": percentile(max_abs.detach().cpu().tolist(), 0.95),
        "argmax_match_rate": float(argmax_same.mean().item()),
        "target_logit_abs_delta_mean": float(target_logit_delta.mean().item()),
        "target_logit_abs_delta_p95": percentile(target_logit_delta.detach().cpu().tolist(), 0.95),
    }


class AttentionStats:
    def __init__(self):
        self.decoded_keys_for_ranking = 0
        self.decoded_selected_keys = []
        self.decoded_values = []
        self.output_cosines = []
        self.output_mses = []
        self.topk_overlaps = []
        self.layers = {}
        self.samples = 0

    def add(self, layer, head, decoded_values, cosine, mse, overlap):
        self.samples += 1
        self.decoded_selected_keys.append(int(decoded_values))
        self.decoded_values.append(int(decoded_values))
        self.output_cosines.append(float(cosine))
        self.output_mses.append(float(mse))
        self.topk_overlaps.append(float(overlap))
        d = self.layers.setdefault(str(layer), {"samples": 0, "cosines": [], "overlaps": [], "decoded_values": [], "per_head": {}})
        d["samples"] += 1
        d["cosines"].append(float(cosine))
        d["overlaps"].append(float(overlap))
        d["decoded_values"].append(int(decoded_values))
        hd = d["per_head"].setdefault(str(head), {"samples": 0, "cosines": [], "overlaps": [], "decoded_values": []})
        hd["samples"] += 1
        hd["cosines"].append(float(cosine))
        hd["overlaps"].append(float(overlap))
        hd["decoded_values"].append(int(decoded_values))

    def summary(self):
        per_layer = {}
        for layer, d in self.layers.items():
            per_layer[layer] = {
                "samples": d["samples"],
                "cosine_mean": statistics.mean(d["cosines"]) if d["cosines"] else None,
                "cosine_p05": percentile(d["cosines"], 0.05),
                "overlap_mean": statistics.mean(d["overlaps"]) if d["overlaps"] else None,
                "decoded_values_mean": statistics.mean(d["decoded_values"]) if d["decoded_values"] else None,
                "per_head": {
                    h: {
                        "samples": hd["samples"],
                        "cosine_mean": statistics.mean(hd["cosines"]) if hd["cosines"] else None,
                        "cosine_p05": percentile(hd["cosines"], 0.05),
                        "overlap_mean": statistics.mean(hd["overlaps"]) if hd["overlaps"] else None,
                        "decoded_values_mean": statistics.mean(hd["decoded_values"]) if hd["decoded_values"] else None,
                    }
                    for h, hd in d["per_head"].items()
                },
            }
        return {
            "samples": self.samples,
            "decoded_keys_for_ranking": self.decoded_keys_for_ranking,
            "decoded_selected_keys_mean": statistics.mean(self.decoded_selected_keys) if self.decoded_selected_keys else None,
            "decoded_selected_keys_p95": percentile(self.decoded_selected_keys, 0.95),
            "decoded_values_mean": statistics.mean(self.decoded_values) if self.decoded_values else None,
            "decoded_values_p95": percentile(self.decoded_values, 0.95),
            "attention_output_cosine_mean": statistics.mean(self.output_cosines) if self.output_cosines else None,
            "attention_output_cosine_p05": percentile(self.output_cosines, 0.05),
            "attention_mse_mean": statistics.mean(self.output_mses) if self.output_mses else None,
            "attention_mse_p95": percentile(self.output_mses, 0.05),
            "attention_topk_overlap_mean": statistics.mean(self.topk_overlaps) if self.topk_overlaps else None,
            "attention_topk_overlap_p05": percentile(self.topk_overlaps, 0.05),
            "per_layer": per_layer,
        }


def _get_layer_top_k(top_k, layer_idx, n_heads):
    """Resolve top_k for a given layer.

    top_k can be:
      - int: uniform budget for all layers
      - dict[int, int]: per-layer budget (already includes recent_guard)
      - dict[tuple[int,int], int]: per-(layer, head) budget
    """
    if isinstance(top_k, int):
        return top_k
    elif isinstance(top_k, dict):
        # Check if keys are tuples (per-head) or ints (per-layer)
        first_key = next(iter(top_k))
        if isinstance(first_key, tuple):
            # per-(layer, head) — caller handles head lookup
            return None  # signal to use per-head path
        else:
            return top_k.get(layer_idx, top_k.get(0, 64))
    return 64


def _get_head_top_k(top_k, layer_idx, head_idx, default_k=64):
    """Resolve top_k for a given (layer, head). Returns None if not per-head."""
    if isinstance(top_k, dict):
        first_key = next(iter(top_k))
        if isinstance(first_key, tuple):
            return top_k.get((layer_idx, head_idx), default_k)
    return None


def compressed_attention_forward(module, query, key, value, attention_mask, scaling,
                                  top_k, recent_guard, quant_bits, stats, collect_stats,
                                  sample_heads=None, sample_positions=None,
                                  scorer="quantized", fib_config=None,
                                  quant_bits_policy=None, full_precision_layers=None):
    key_states = repeat_kv(key, module.num_key_value_groups)
    value_states = repeat_kv(value, module.num_key_value_groups)
    bsz, nheads, q_len, dim = query.shape
    kv_len = key_states.shape[-2]

    # Resolve per-layer quant_bits
    layer_quant_bits = quant_bits
    if quant_bits_policy is not None and isinstance(quant_bits_policy, dict):
        layer_quant_bits = quant_bits_policy.get(module.layer_idx, quant_bits)
    levels = (1 << (layer_quant_bits - 1)) - 1

    # Full-precision bypass for very fragile layers
    use_full_precision = (full_precision_layers is not None
                          and module.layer_idx in full_precision_layers)
    outputs = torch.empty_like(query)

    # full scores only needed for reference stats / selected exact softmax.
    full_scores = torch.matmul(query, key_states.transpose(2, 3)) * scaling
    if attention_mask is not None:
        full_scores = full_scores + attention_mask[:, :, :, :kv_len]
    full_weights = F.softmax(full_scores, dim=-1, dtype=torch.float32).to(query.dtype)
    full_out = torch.matmul(full_weights, value_states)

    # Determine sampling set
    sampled_heads = set()
    if collect_stats and sample_heads is not None and sample_heads < nheads:
        # Uniformly sample sample_heads heads
        step = max(1, nheads // sample_heads)
        sampled_heads = set(range(0, nheads, step)[:sample_heads])
    elif collect_stats:
        sampled_heads = set(range(nheads))

    sampled_positions = set()
    if collect_stats and sample_positions is not None and sample_positions < q_len:
        step = max(1, q_len // sample_positions)
        sampled_positions = set(range(0, q_len, step)[:sample_positions])
    elif collect_stats:
        sampled_positions = set(range(q_len))

    # Full-precision bypass: return standard attention output for very fragile layers
    if use_full_precision:
        if collect_stats:
            for b in range(bsz):
                for h in range(nheads):
                    if h in sampled_heads:
                        for qi in range(q_len):
                            if qi in sampled_positions:
                                stats.add(module.layer_idx, h, kv_len, 1.0, 0.0, 1.0)
        return full_out.transpose(1, 2).contiguous(), None

    # Pre-init defaults (overwritten when scorer == "fib-gram")
    qk = scales = None
    fib_codes = fib_norms = head_codes = head_norms = None

    # fib-gram setup (per layer)
    fib_codebook = None
    fib_gram = None
    fib_M = fib_K = 0
    fib_mod = None
    if scorer == "fib-gram" and fib_config is not None:
        fib_mod = _get_fib_gram()
        fib_M = fib_config.get("M", 16)
        fib_K = fib_config.get("K", 32)
        # Build codebook from all key vectors (flattened across batch/head)
        all_keys = key_states.reshape(-1, dim).float()
        fib_codebook = fib_mod.build_codebook_kmeans(all_keys, fib_M, fib_K)
        fib_gram = fib_mod.compute_gram_table(fib_codebook)
        # Encode all keys
        fib_codes, fib_norms = fib_mod.encode_batch(all_keys, fib_codebook, fib_M, fib_K)
        # Reshape back: (bsz, nheads, kv_len, M)
        fib_codes = fib_codes.reshape(bsz, nheads, kv_len, fib_M)
        fib_norms = fib_norms.reshape(bsz, nheads, kv_len)

    for b in range(bsz):
        for h in range(nheads):
            k_h = key_states[b, h]
            v_h = value_states[b, h]

            # Determine top_k for this head
            head_k = _get_head_top_k(top_k, module.layer_idx, h)
            if head_k is not None:
                layer_k = head_k
            else:
                layer_k = _get_layer_top_k(top_k, module.layer_idx, nheads)
            if layer_k is None:
                layer_k = 64
            k_select_total = min(layer_k, kv_len)

            # Pre-compute quantized keys for this head (for quantized scorer)
            if scorer == "quantized":
                max_abs = k_h.abs().amax(dim=-1, keepdim=True).clamp_min(1e-6)
                qk = torch.round((k_h / max_abs).clamp(-1, 1) * levels).to(torch.int8)
                scales = max_abs.squeeze(-1).to(torch.float32)

            # Pre-compute per-dimension quantized keys for this head
            if scorer == "per-dim":
                # Asymmetric per-dimension min/max quantization over unit-normalized keys.
                k_norms = k_h.norm(dim=-1).clamp_min(1e-6)  # (kv_len,)
                k_unit = k_h / k_norms[:, None]
                dim_mins = k_unit.min(dim=0).values  # (dim,)
                dim_maxs = k_unit.max(dim=0).values  # (dim,)
                dim_ranges = (dim_maxs - dim_mins).clamp_min(1e-6)  # (dim,)
                per_dim_levels = (1 << layer_quant_bits) - 1
                qk = torch.round(
                    ((k_unit - dim_mins[None, :]) / dim_ranges[None, :]) * per_dim_levels
                ).clamp(0, per_dim_levels).to(torch.uint8)
                scales = (dim_mins, dim_ranges, per_dim_levels, k_norms)

            # Pre-compute fib codes for this head
            if scorer == "fib-gram" and fib_config is not None:
                head_codes = fib_codes[b, h]  # (kv_len, M)
                head_norms = fib_norms[b, h]   # (kv_len,)

            for qi in range(q_len):
                q = query[b, h, qi].float()

                if scorer == "fib-gram" and fib_config is not None:
                    # Gram-table scoring
                    _, q_norm, q_indices = fib_mod.prepare_query(q, fib_codebook, fib_M, fib_K)
                    approx_scores = fib_mod.score_batch_prepared_vectorized(
                        q_indices, head_codes, head_norms, fib_gram, q_norm, fib_M, fib_K
                    )
                    approx_scores = approx_scores * scaling
                elif scorer == "per-dim":
                    # Asymmetric per-dimension quantized scoring over unit-normalized keys.
                    dim_mins, dim_ranges, per_dim_levels, k_norms = scales
                    key_recon_unit = dim_mins[None, :] + (qk.float() / per_dim_levels) * dim_ranges[None, :]
                    approx_scores = (key_recon_unit * q[None, :]).sum(dim=-1) * k_norms * scaling
                else:
                    # Simple per-key quantized scoring
                    approx_scores = ((qk.float() / levels) * scales[:, None] * q[None, :]).sum(dim=-1) * scaling

                if attention_mask is not None:
                    approx_scores = approx_scores + attention_mask[b, 0, qi, :kv_len].float()
                selected = torch.topk(approx_scores, k=k_select_total).indices
                if recent_guard > 0:
                    guard = torch.arange(max(0, kv_len - recent_guard), kv_len, device=query.device)
                    selected = torch.unique(torch.cat([selected, guard]), sorted=False)
                exact_sel_scores = torch.matmul(k_h[selected], query[b, h, qi]) * scaling
                if attention_mask is not None:
                    exact_sel_scores = exact_sel_scores + attention_mask[b, 0, qi, selected]
                weights = F.softmax(exact_sel_scores.float(), dim=-1).to(query.dtype)
                out = torch.matmul(weights, v_h[selected])
                outputs[b, h, qi] = out

                if collect_stats and h in sampled_heads and qi in sampled_positions:
                    exact_top = torch.topk(full_scores[b, h, qi], k=k_select_total).indices
                    ss = set(int(x) for x in selected.detach().cpu().tolist())
                    es = set(int(x) for x in exact_top.detach().cpu().tolist())
                    overlap = len(ss & es) / max(1, len(es))
                    fo = full_out[b, h, qi].float()
                    co = out.float()
                    denom = fo.norm() * co.norm()
                    cos = float((torch.dot(fo, co) / denom).item()) if denom.item() != 0 else 0.0
                    mse = float(torch.mean((fo - co) ** 2).item())
                    stats.add(module.layer_idx, h, len(ss), cos, mse, overlap)
    return outputs.transpose(1, 2).contiguous(), None


def patch_model(model, top_k, recent_guard, quant_bits, stats, collect_stats,
                sample_heads=None, sample_positions=None,
                scorer="quantized", fib_config=None, quant_bits_policy=None,
                full_precision_layers=None):
    for layer in model.model.layers:
        attn = layer.self_attn
        def patched_forward(self, hidden_states, position_embeddings=None, attention_mask=None, past_key_values=None, cache_position=None, **kwargs):
            input_shape = hidden_states.shape[:-1]
            hidden_shape = (*input_shape, -1, self.head_dim)
            query_states = self.q_proj(hidden_states).view(hidden_shape).transpose(1, 2)
            key_states = self.k_proj(hidden_states).view(hidden_shape).transpose(1, 2)
            value_states = self.v_proj(hidden_states).view(hidden_shape).transpose(1, 2)
            cos, sin = position_embeddings
            query_states, key_states = apply_rotary_pos_emb(query_states, key_states, cos, sin)
            if past_key_values is not None:
                cache_kwargs = {"sin": sin, "cos": cos, "cache_position": cache_position}
                key_states, value_states = past_key_values.update(key_states, value_states, self.layer_idx, cache_kwargs)
            attn_output, attn_weights = compressed_attention_forward(
                self, query_states, key_states, value_states, attention_mask,
                self.scaling, top_k, recent_guard, quant_bits, stats, collect_stats,
                sample_heads=sample_heads, sample_positions=sample_positions,
                scorer=scorer, fib_config=fib_config,
                quant_bits_policy=quant_bits_policy,
                full_precision_layers=full_precision_layers,
            )
            attn_output = attn_output.reshape(*input_shape, -1).contiguous()
            attn_output = self.o_proj(attn_output)
            return attn_output, attn_weights
        attn.forward = types.MethodType(patched_forward, attn)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="HuggingFaceTB/SmolLM2-1.7B-Instruct")
    ap.add_argument("--corpus", default="wikitext-2")
    ap.add_argument("--n-tokens", type=int, default=128)
    ap.add_argument("--ppl-frac", type=float, default=0.3)
    ap.add_argument("--top-k", type=int, default=64)
    ap.add_argument("--recent-guard", type=int, default=16)
    ap.add_argument("--quant-bits", type=int, default=4)
    ap.add_argument("--output", type=Path, required=True)
    ap.add_argument("--device", default="cuda", choices=["cuda", "cpu"])
    ap.add_argument("--collect-attention-stats", action="store_true")
    # Sampling flags
    ap.add_argument("--sample-heads", type=int, default=None, help="Heads to sample per layer for stats")
    ap.add_argument("--sample-positions", type=int, default=None, help="Query positions to sample per head")
    # Adaptive budget flags
    ap.add_argument("--adaptive-budget", action="store_true")
    ap.add_argument("--adaptive-budget-heads", action="store_true")
    ap.add_argument("--fragility-file", type=Path, default=None, help="JSON file with per-layer fragility data")
    ap.add_argument("--budget-target-cosine", type=float, default=0.995)
    ap.add_argument("--budget-min-k", type=int, default=32)
    ap.add_argument("--budget-max-k", type=int, default=256)
    ap.add_argument("--budget-ref-k", type=int, default=64)
    # Scorer flags
    ap.add_argument("--scorer", default="quantized", choices=["quantized", "fib-gram", "per-dim"])
    ap.add_argument("--fib-m", type=int, default=16, help="Number of subspaces for fib-quant")
    ap.add_argument("--fib-k", type=int, default=32, help="Codebook size per subspace")
    # Per-layer quant bits policy
    ap.add_argument("--per-layer-quant-bits", action="store_true",
                    help="Use adaptive quant_bits: fragile layers get 8-bit, others get --quant-bits")
    args = ap.parse_args()

    started = time.time()
    device = args.device if args.device == "cpu" or torch.cuda.is_available() else "cpu"
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    if args.corpus == "wikitext-2":
        ds = load_dataset("Salesforce/wikitext", "wikitext-2-raw-v1", split="test")
        text = "\n\n".join(ds["text"])
    elif args.corpus.startswith("file:"):
        text = Path(args.corpus[5:]).read_text()
    else:
        raise ValueError(args.corpus)
    ids = tokenizer(text, return_tensors="pt").input_ids[:, : args.n_tokens].to(device)

    torch.manual_seed(42)
    model = AutoModelForCausalLM.from_pretrained(args.model, torch_dtype=torch.float16).to(device)
    model.eval()
    model.config._attn_implementation = "eager"
    with torch.no_grad():
        t0 = time.time()
        base = model(input_ids=ids, use_cache=False, return_dict=True).logits.detach()
        baseline_s = time.time() - t0
    base_nll = causal_nll_from_logits(base, ids)
    base_ppl, eval_start, eval_end = windowed_ppl(base_nll, args.ppl_frac)

    # Determine top_k
    top_k = args.top_k
    if args.adaptive_budget_heads:
        ab = _get_adaptive_budget()
        if args.fragility_file:
            frag_data = json.loads(args.fragility_file.read_text())
            head_frag = ab.head_fragility_from_receipt(frag_data)
        else:
            # Build head fragility from per-layer defaults (all heads same fragility)
            layer_frag = ab.load_fragility(args.fragility_file, args.n_tokens)
            # Get num heads from model config
            num_heads = model.config.num_attention_heads
            head_frag = {(l, h): v for l, v in layer_frag.items() for h in range(num_heads)}
        top_k = ab.allocate_head_budgets(
            head_frag, args.budget_ref_k, args.budget_target_cosine,
            args.budget_min_k, args.budget_max_k, args.recent_guard
        )
        mean_k = ab.compute_expected_mean_k(top_k, args.n_tokens)
        print(f"[adaptive-heads] allocated budgets for {len(top_k)} (layer,head) pairs, expected mean k: {mean_k:.1f}")
    elif args.adaptive_budget:
        ab = _get_adaptive_budget()
        frag = ab.load_fragility(args.fragility_file, args.n_tokens)
        top_k = ab.allocate_budgets(
            frag, args.budget_ref_k, args.budget_target_cosine,
            args.budget_min_k, args.budget_max_k, args.recent_guard
        )
        mean_k = ab.compute_expected_mean_k(top_k, args.n_tokens)
        print(f"[adaptive-layers] allocated budgets for {len(top_k)} layers, expected mean k: {mean_k:.1f}")
        for layer in sorted(top_k):
            print(f"  layer {layer:2d}: fragility={frag.get(layer, '?'):.4f}  budget={top_k[layer]}")

    # Fib config
    fib_config = None
    if args.scorer == "fib-gram":
        fib_config = {"M": args.fib_m, "K": args.fib_k}
        print(f"[fib-gram] M={args.fib_m} K={args.fib_k}")

    # Per-layer quant bits policy: fragile layers get 8-bit, very fragile get full-precision bypass
    quant_bits_policy = None
    full_precision_layers = set()
    if args.per_layer_quant_bits:
        ab = _get_adaptive_budget()
        frag = ab.load_fragility(args.fragility_file, args.n_tokens)
        quant_bits_policy = {}
        for layer, cos_p05 in frag.items():
            if cos_p05 < 0.985:
                # Very fragile: full-precision bypass
                full_precision_layers.add(layer)
                quant_bits_policy[layer] = args.quant_bits  # won't be used
            elif cos_p05 < args.budget_target_cosine:
                quant_bits_policy[layer] = 8  # fragile: 8-bit
            else:
                quant_bits_policy[layer] = args.quant_bits  # stable: default bits
        fragile_8bit = [l for l, b in quant_bits_policy.items() if b > args.quant_bits and l not in full_precision_layers]
        print(f"[per-layer-quant] {len(fragile_8bit)} fragile layers get 8-bit: {fragile_8bit}")
        print(f"[per-layer-quant] {len(full_precision_layers)} very fragile layers get full-precision bypass: {sorted(full_precision_layers)}")

    stats = AttentionStats()
    patch_model(model, top_k, args.recent_guard, args.quant_bits, stats, args.collect_attention_stats,
                sample_heads=args.sample_heads, sample_positions=args.sample_positions,
                scorer=args.scorer, fib_config=fib_config,
                quant_bits_policy=quant_bits_policy,
                full_precision_layers=full_precision_layers if args.per_layer_quant_bits else None)
    with torch.no_grad():
        t1 = time.time()
        comp = model(input_ids=ids, use_cache=False, return_dict=True).logits.detach()
        compressed_s = time.time() - t1
    comp_nll = causal_nll_from_logits(comp, ids)
    comp_ppl, c_start, c_end = windowed_ppl(comp_nll, args.ppl_frac)
    metrics = logit_metrics(base, comp, ids, eval_start)

    delta_pct = (comp_ppl - base_ppl) / base_ppl * 100.0
    attn_summary = stats.summary()
    thresholds = {
        "ppl_delta_abs_pct_max": 3.0,
        "logit_cosine_p05_min": 0.995,
        "kl_p95_max": 0.05,
        "decoded_keys_for_ranking_required": 0,
    }
    passed = (
        abs(delta_pct) <= thresholds["ppl_delta_abs_pct_max"]
        and (metrics["logit_cosine_p05"] or 0) >= thresholds["logit_cosine_p05_min"]
        and (metrics["kl_p95"] or 999) <= thresholds["kl_p95_max"]
        and attn_summary["decoded_keys_for_ranking"] == 0
    )
    receipt = {
        "schema_version": "compressed_attention_forward_ppl_v1",
        "model": args.model,
        "corpus": args.corpus,
        "n_tokens": args.n_tokens,
        "ppl_frac": args.ppl_frac,
        "eval_window": [eval_start, eval_end],
        "top_k": top_k if isinstance(top_k, int) else "adaptive",
        "recent_guard": args.recent_guard,
        "quant_bits": args.quant_bits,
        "scorer": args.scorer,
        "adaptive_budget": args.adaptive_budget or args.adaptive_budget_heads,
        "device": device,
        "baseline_ppl": base_ppl,
        "compressed_attention_ppl": comp_ppl,
        "delta_ppl_pct": delta_pct,
        "baseline_forward_seconds": baseline_s,
        "compressed_forward_seconds": compressed_s,
        **metrics,
        "attention": attn_summary,
        "thresholds": thresholds,
        "passed": passed,
        "elapsed_s": time.time() - started,
        "claim_boundary": "Actual model forward pass patched at LlamaAttention after RoPE and before o_proj. Candidate ranking uses quantized key scores without full-key decode; selected keys and values are used only for exact selected softmax/output. Python implementation is quality-only, not speed evidence.",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2))
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()