"""Bulk-oriented Python sidecar for poly-kv alpha receipt experiments."""

from .exceptions import PolyKvError, PolyKvNativeUnavailable, PolyKvShapeError
from .receipts import (
    ShapeV2,
    attach_synthetic_reader,
    build_pool_from_fixture,
    build_synthetic_pool,
    decode_synthetic_slice,
    native_available,
    validate_shape,
)

__all__ = [
    "PolyKvError",
    "PolyKvNativeUnavailable",
    "PolyKvShapeError",
    "ShapeV2",
    "attach_synthetic_reader",
    "build_pool_from_fixture",
    "build_synthetic_pool",
    "decode_synthetic_slice",
    "native_available",
    "validate_shape",
]
