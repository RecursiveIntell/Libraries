"""Debug test for per-key quantization."""

import torch

def test_per_key_quantization():
    device = 'cuda'
    n_keys = 64
    dim = 128
    bits = 8
    
    # Generate random data
    keys = torch.randn(n_keys, dim, device=device, dtype=torch.float32)
    query = torch.randn(dim, device=device, dtype=torch.float32)
    
    # Compute scales
    scales = keys.abs().max(dim=1).values.clamp(min=1e-6)
    print(f"Scales: min={scales.min():.3f}, max={scales.max():.3f}, mean={scales.mean():.3f}")
    
    # Quantize
    levels = (1 << (bits - 1)) - 1  # 127 for 8-bit
    print(f"Levels: {levels}")
    
    qk_float = torch.round(keys / scales.unsqueeze(1) * levels)
    qk = qk_float.clamp(-levels, levels).to(torch.int8)
    print(f"qk: min={qk.min()}, max={qk.max()}, mean={qk.float().mean():.2f}")
    
    # Reconstruct
    keys_recon = qk.float() * scales.unsqueeze(1) / levels
    
    # Check reconstruction quality
    recon_error = (keys - keys_recon).norm(dim=1) / keys.norm(dim=1)
    print(f"Relative reconstruction error: mean={recon_error.mean():.4f}, max={recon_error.max():.4f}")
    
    # Compute scores
    scores_dense = torch.mv(keys, query)
    scores_recon = torch.mv(keys_recon, query)
    
    cos = torch.nn.functional.cosine_similarity(
        scores_dense.unsqueeze(0), scores_recon.unsqueeze(0)).item()
    print(f"Cosine similarity (dense vs reconstructed): {cos:.6f}")
    
    # Check if the issue is in how we compute the quantized score
    scores_quantized = ((qk.float() / levels) * scales.unsqueeze(1) * query.unsqueeze(0)).sum(dim=-1)
    cos2 = torch.nn.functional.cosine_similarity(
        scores_dense.unsqueeze(0), scores_quantized.unsqueeze(0)).item()
    print(f"Cosine similarity (dense vs quantized formula): {cos2:.6f}")
    
    # Check if scores_recon and scores_quantized are the same
    print(f"Max diff between scores_recon and scores_quantized: {(scores_recon - scores_quantized).abs().max():.6f}")

if __name__ == '__main__':
    test_per_key_quantization()
