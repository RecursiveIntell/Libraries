import pytest

import poly_kv


def require_native():
    if not poly_kv.native_available():
        pytest.skip("poly_kv._native is not built; run maturin develop or maturin build")


def test_bad_shape_rejected_by_native_sidecar():
    require_native()
    shape = poly_kv.ShapeV2(
        batch=1,
        layers=2,
        num_q_heads=8,
        num_kv_heads=3,
        seq_len=8,
        head_dim=4,
        attention_kind="gqa",
    )
    with pytest.raises(poly_kv.PolyKvShapeError):
        poly_kv.validate_shape(shape)
