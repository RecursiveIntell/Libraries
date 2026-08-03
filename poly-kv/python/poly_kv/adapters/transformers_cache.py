"""Correctness-first Hugging Face Transformers KV-cache adapter."""
from __future__ import annotations

import hashlib
import json
from typing import Any


class TransformersKVExtractor:
    """Extract genuine KV tensors from DynamicCache or legacy tuples."""

    def extract(self, past_key_values, model_config, tokenizer, input_ids, position_ids=None) -> dict:
        family = self._family(model_config)
        if family == "qwen3.5":
            raise ValueError("Qwen3.5 hybrid-state caches are unsupported; refusing lossy KV-only extraction")
        layers = []
        key_values = getattr(past_key_values, "key_cache", None)
        value_values = getattr(past_key_values, "value_cache", None)
        if key_values is None:
            key_values = [pair[0] for pair in past_key_values]
            value_values = [pair[1] for pair in past_key_values]
        for idx, (key, value) in enumerate(zip(key_values, value_values)):
            layers.append({"layer_idx": idx, "key_tensor": self._flat(key), "value_tensor": self._flat(value)})
        ids = self._flat(input_ids)
        positions = ids if position_ids is None else self._flat(position_ids)
        revision = str(getattr(model_config, "_commit_hash", None) or getattr(model_config, "revision", "unknown"))
        model_id = str(getattr(model_config, "_name_or_path", "unknown"))
        tok_id = str(getattr(tokenizer, "name_or_path", "unknown"))
        shape = {"layers": len(layers), "key_heads": int(key_values[0].shape[-3]), "value_heads": int(value_values[0].shape[-3]), "seq_len": len(ids), "head_dim": int(key_values[0].shape[-1]), "layout": "transformers", "dtype": "F32"}
        bundle = {"model_fingerprint": self._digest(model_id), "tokenizer_fingerprint": self._digest(tok_id), "revision": revision, "config_digest": self._digest(json.dumps(vars(model_config), sort_keys=True, default=str)), "shape": shape, "dtype": "F32", "layers": layers, "token_ids": ids, "position_ids": positions, "seq_len": len(ids), "model_family": family}
        return bundle

    def verify_fingerprint(self, bundle: dict) -> bool:
        required = ("model_fingerprint", "tokenizer_fingerprint", "revision", "config_digest")
        return all(isinstance(bundle.get(k), str) and bundle[k] for k in required) and bundle.get("seq_len") == len(bundle.get("token_ids", []))

    @staticmethod
    def _flat(value):
        return value.detach().cpu().reshape(-1).tolist() if hasattr(value, "detach") else list(value)

    @staticmethod
    def _digest(value: str) -> str:
        return hashlib.blake2b(value.encode(), digest_size=32).hexdigest()

    @staticmethod
    def _family(config) -> str:
        name = str(getattr(config, "model_type", "") or getattr(config, "_name_or_path", "")).lower()
        if "qwen3.5" in name or "qwen3_5" in name:
            return "qwen3.5"
        if any(x in name for x in ("llama", "smollm", "mistral")):
            return name
        raise ValueError(f"unsupported Transformers model family: {name or 'unknown'}")


class TransformersKVInjector:
    """Restore extracted tensors into a DynamicCache without shared mutation."""

    def inject(self, bundle: dict, target_cache=None):
        try:
            from transformers import DynamicCache
        except ImportError as exc:
            raise RuntimeError("transformers is required for cache injection") from exc
        cache = target_cache or DynamicCache()
        import torch
        for layer in bundle["layers"]:
            key = torch.tensor(layer["key_tensor"], dtype=torch.float32).reshape(1, bundle["shape"]["key_heads"], bundle["seq_len"], bundle["shape"]["head_dim"])
            value = torch.tensor(layer["value_tensor"], dtype=torch.float32).reshape(1, bundle["shape"]["value_heads"], bundle["seq_len"], bundle["shape"]["head_dim"])
            cache.update(key.clone(), value.clone(), layer["layer_idx"])
        return cache

    def compare_logits(self, model, token_ids, cache_a, cache_b, atol=1e-5) -> dict:
        import torch
        ids = torch.as_tensor(token_ids)
        with torch.no_grad():
            a = model(input_ids=ids, past_key_values=cache_a).logits
            b = model(input_ids=ids, past_key_values=cache_b).logits
        diff = (a - b).abs()
        cosine = torch.nn.functional.cosine_similarity(a.reshape(1, -1), b.reshape(1, -1)).item()
        return {"cosine": cosine, "max_diff": diff.max().item(), "match_rate": (diff <= atol).float().mean().item()}
