from __future__ import annotations
import dataclasses, hashlib, json, os, pathlib, stat
from typing import Any

VERSION = 'receipt-foundation/0.2.0'
RULES_VERSION = 'receipt-observation-rules/2'
SCHEMA_VERSION = 2

class FoundationError(Exception):
    """Safe typed error: the message must never interpolate source content."""
    def __init__(self, code: str, *, offset: int | None = None):
        self.code, self.offset = code, offset
        self.context: dict = {}
        super().__init__(code)
    def as_dict(self) -> dict:
        return {'code': self.code, 'offset': self.offset}

@dataclasses.dataclass(frozen=True)
class Limits:
    archive_bytes: int = 128 * 1024 * 1024
    decompressed_bytes: int = 256 * 1024 * 1024
    member_bytes: int = 32 * 1024 * 1024
    extension_bytes: int = 64 * 1024
    members: int = 20000
    physical_headers: int = 40000
    path_bytes: int = 8192
    json_depth: int = 128
    json_values: int = 100000
    json_tokens: int = 1000000
    json_string_chars: int = 8 * 1024 * 1024
    json_number_chars: int = 1024
    decompression_seconds: int = 120
    projection_records: int = 100000
    projection_bytes: int = 512 * 1024 * 1024


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True, allow_nan=False)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def identifier(domain: str, *parts: Any) -> str:
    """Projection-local ID, deliberately not stack-ids or native ArtifactId."""
    h = hashlib.sha256()
    h.update(('recursiveintell:receipt-observation:' + domain + ':v1\0').encode())
    for part in parts:
        b = canonical_json(part).encode()
        h.update(len(b).to_bytes(8, 'big')); h.update(b)
    return h.hexdigest()


def open_source(path: pathlib.Path):
    """No symlink following; regular files only. Caller owns returned descriptor."""
    try:
        fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
    except OSError as exc:
        raise FoundationError('SOURCE_OPEN_FAILURE') from exc
    if not stat.S_ISREG(os.fstat(fd).st_mode):
        os.close(fd); raise FoundationError('SOURCE_NOT_REGULAR')
    return os.fdopen(fd, 'rb')


def private_write(path: pathlib.Path, data: bytes, *, exclusive: bool = True) -> None:
    """Creation only by default: never overwrite user state or follow symlinks."""
    flags = os.O_WRONLY | os.O_CREAT | getattr(os, 'O_NOFOLLOW', 0)
    flags |= os.O_EXCL if exclusive else os.O_TRUNC
    try:
        fd = os.open(path, flags, 0o600)
    except OSError as exc:
        raise FoundationError('OUTPUT_CREATE_FAILURE') from exc
    with os.fdopen(fd, 'wb') as f:
        f.write(data); f.flush(); os.fsync(f.fileno())


def write_json(path: pathlib.Path, obj: Any, *, exclusive: bool = True) -> None:
    private_write(path, (json.dumps(obj, sort_keys=True, indent=2, ensure_ascii=True, allow_nan=False) + '\n').encode(), exclusive=exclusive)
