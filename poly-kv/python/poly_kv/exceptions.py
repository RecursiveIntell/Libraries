class PolyKvError(Exception):
    """Base exception for the optional poly-kv Python sidecar."""


class PolyKvNativeUnavailable(PolyKvError):
    """Raised when the PyO3 extension has not been built in the current environment."""


class PolyKvShapeError(PolyKvError):
    """Raised when shape, dtype, layout, or role validation fails closed."""
