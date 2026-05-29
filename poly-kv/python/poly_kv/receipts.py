from __future__ import annotations

from dataclasses import asdict, dataclass
import json
from typing import Any, Mapping, Sequence

from .exceptions import PolyKvNativeUnavailable, PolyKvShapeError

try:
    from . import _native
except ImportError:
    _native = None


@dataclass(frozen=True)
class ShapeV2:
    batch: int
    layers: int
    num_q_heads: int
    num_kv_heads: int
    seq_len: int
    head_dim: int
    layout: str = "layers_heads_tokens_dim"
    dtype: str = "f32"
    attention_kind: str = "mha"

    def to_json(self) -> str:
        return json.dumps(asdict(self), sort_keys=True)


def native_available() -> bool:
    return _native is not None


def validate_shape(shape: ShapeV2) -> dict[str, Any]:
    return _loads(_call_shape(_require_native().validate_shape_json, shape.to_json()))


def build_synthetic_pool(shape: ShapeV2) -> dict[str, Any]:
    return _loads(
        _call_shape(_require_native().build_synthetic_pool_receipts_json, shape.to_json())
    )


def attach_synthetic_reader(shape: ShapeV2) -> dict[str, Any]:
    return _loads(
        _call_shape(_require_native().attach_synthetic_reader_receipt_json, shape.to_json())
    )


def decode_synthetic_slice(
    shape: ShapeV2,
    *,
    role: str,
    layer: int,
    start: int,
    end: int,
) -> dict[str, Any]:
    return _loads(
        _call_shape(
            _require_native().decode_synthetic_slice_receipt_json,
            shape.to_json(),
            role,
            layer,
            start,
            end,
        )
    )


def build_pool_from_fixture(
    shape: ShapeV2, blocks: Sequence[Mapping[str, Any]]
) -> dict[str, Any]:
    return _loads(
        _call_shape(
            _require_native().build_pool_from_f32_json,
            shape.to_json(),
            json.dumps(list(blocks), sort_keys=True),
        )
    )


def _require_native() -> Any:
    if _native is None:
        raise PolyKvNativeUnavailable(
            "poly_kv._native is not installed; run maturin develop or maturin build"
        )
    return _native


def _call_shape(fn: Any, *args: Any) -> str:
    try:
        return fn(*args)
    except ValueError as exc:
        raise PolyKvShapeError(str(exc)) from exc


def _loads(value: str) -> dict[str, Any]:
    loaded = json.loads(value)
    if not isinstance(loaded, dict):
        raise PolyKvShapeError("native sidecar returned non-object JSON")
    return loaded
