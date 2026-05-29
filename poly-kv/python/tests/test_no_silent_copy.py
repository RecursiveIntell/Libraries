import pytest

import poly_kv


def require_native():
    if not poly_kv.native_available():
        pytest.skip("poly_kv._native is not built; run maturin develop or maturin build")


def test_decode_receipt_discloses_full_block_decode_and_copy():
    require_native()
    shape = poly_kv.ShapeV2(
        batch=1,
        layers=1,
        num_q_heads=2,
        num_kv_heads=2,
        seq_len=8,
        head_dim=4,
        attention_kind="mha",
    )
    decoded = poly_kv.decode_synthetic_slice(
        shape, role="key", layer=0, start=0, end=2
    )
    receipt = decoded["receipt"]
    assert receipt["full_block_decoded"] is True
    assert receipt["returned_values"] < receipt["decoded_full_values"]
    assert receipt["copy_performed"] is True
