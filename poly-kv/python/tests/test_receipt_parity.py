import pytest

import poly_kv


def require_native():
    if not poly_kv.native_available():
        pytest.skip("poly_kv._native is not built; run maturin develop or maturin build")


def test_synthetic_build_receipts_are_json_compatible():
    require_native()
    shape = poly_kv.ShapeV2(
        batch=1,
        layers=2,
        num_q_heads=2,
        num_kv_heads=2,
        seq_len=8,
        head_dim=4,
        attention_kind="mha",
    )
    receipts = poly_kv.build_synthetic_pool(shape)

    build = receipts["build_receipt"]
    assert build["compression_evals"]
    assert build["memory"]["manifest_bytes"] > 0
    assert build["memory"]["encoded_shared_bytes"] == build["encoded_bytes"]
    assert any(item["role"] == "Key" for item in build["compression_evals"])
