"""Bounded streaming tar observer. No extraction, path writes, or native code loading.

Supports strict USTAR/GNU regular/directory entries and bounded GNU L long names.
PAX, sparse, links, devices and unknown extension formats fail closed or are rejected.
Control headers have independent accounting; logical occurrence IDs use header offset.
"""
from __future__ import annotations
import dataclasses, hashlib, os, pathlib, re, subprocess, threading, unicodedata
from typing import BinaryIO, Iterator
from .common import FoundationError, Limits, digest, identifier, open_source

ZERO = b'\0' * 512

@dataclasses.dataclass
class Member:
    ordinal: int
    header_offset: int
    data_offset: int
    path: str
    path_key: str | None
    member_type: str
    size: int
    mode: int
    mtime_header: str
    blob_sha256: str | None
    data: bytes | None
    issues: list[str]
    header_sha256: str
    longname_header_offset: int | None = None


def path_problem(path: str, limits: Limits = Limits()) -> tuple[str | None, list[str]]:
    issues: list[str] = []
    try: raw = path.encode('utf-8')
    except UnicodeEncodeError: return None, ['PATH_INVALID_UNICODE']
    if len(raw) > limits.path_bytes: issues.append('PATH_LENGTH_LIMIT')
    if path.startswith('/') or re.match(r'^[A-Za-z]:', path): issues.append('PATH_ABSOLUTE')
    if '\\' in path: issues.append('PATH_BACKSLASH')
    if any(unicodedata.category(c) in {'Cc', 'Cf', 'Cs'} for c in path): issues.append('PATH_CONTROL_CHARACTER')
    if unicodedata.normalize('NFC', path) != path: issues.append('PATH_NON_NFC')
    # A single conventional ./ prefix and a directory slash are transport syntax,
    # not a rewrite of the historical path stored on the occurrence.
    key = path[2:] if path.startswith('./') else path
    if key in {'', '.'}: return ('.', issues) if path in {'.', './'} else (None, issues + ['PATH_EMPTY'])
    if key.endswith('/'): key = key[:-1]
    parts = key.split('/')
    if any(p in {'', '.', '..'} for p in parts): issues.append('PATH_TRAVERSAL_OR_ALIAS')
    if len(parts) > 128: issues.append('PATH_DEPTH_LIMIT')
    # Colon and trailing dot/space names are not portable to Windows.
    if any(':' in p or p.endswith((' ', '.')) for p in parts): issues.append('PATH_NONPORTABLE_COMPONENT')
    reserved = {'CON', 'PRN', 'AUX', 'NUL'} | {f'{p}{i}' for p in ('COM','LPT') for i in range(1,10)}
    if any(p.split('.')[0].upper() in reserved for p in parts): issues.append('PATH_RESERVED_COMPONENT')
    return (None if issues else key), issues


def octal(raw: bytes, code: str, offset: int) -> int:
    text = raw.strip(b'\0 ')
    if not text: return 0
    if not re.fullmatch(b'[0-7]+', text): raise FoundationError(code, offset=offset)
    return int(text, 8)


def cstring(raw: bytes, offset: int) -> str:
    a, _, tail = raw.partition(b'\0')
    if tail.strip(b'\0'): raise FoundationError('TAR_NUL_PADDING_INVALID', offset=offset)
    try: return a.decode('utf-8', 'strict')
    except UnicodeDecodeError as exc: raise FoundationError('TAR_PATH_UTF8_INVALID', offset=offset) from exc


class TarReader:
    def __init__(self, stream: BinaryIO, limits: Limits = Limits()):
        self.stream, self.limits, self.offset = stream, limits, 0
        self.controls: list[dict] = []
        self.header_count = 0
        self.member_count = 0
        self.padding_bytes = 0

    def read(self, size: int, code: str = 'TAR_TRUNCATED') -> bytes:
        if self.offset + size > self.limits.decompressed_bytes:
            raise FoundationError('DECOMPRESSED_SIZE_LIMIT', offset=self.offset)
        out = bytearray()
        while len(out) < size:
            b = self.stream.read(min(size-len(out), 65536))
            if not b: raise FoundationError(code, offset=self.offset + len(out))
            out.extend(b)
        self.offset += size
        return bytes(out)

    def __iter__(self) -> Iterator[Member]:
        seen: dict[str, int] = {}; portable: dict[str,str] = {}
        longname: str | None = None; long_offset: int | None = None
        while True:
            off = self.offset; header = self.read(512)
            if header == ZERO:
                if longname is not None: raise FoundationError('TAR_ORPHAN_LONGNAME', offset=off)
                if self.read(512) != ZERO: raise FoundationError('TAR_INVALID_END_MARKER', offset=off)
                self.padding_bytes = 1024
                while True:
                    b = self.stream.read(65536)
                    if not b: break
                    self.offset += len(b); self.padding_bytes += len(b)
                    if self.offset > self.limits.decompressed_bytes: raise FoundationError('DECOMPRESSED_SIZE_LIMIT', offset=self.offset)
                    if b.strip(b'\0'): raise FoundationError('TAR_NONZERO_TRAILING_DATA', offset=self.offset-len(b))
                return
            self.header_count += 1
            if self.header_count > self.limits.physical_headers: raise FoundationError('TAR_HEADER_COUNT_LIMIT', offset=off)
            expected = octal(header[148:156], 'TAR_CHECKSUM_FORMAT', off)
            actual = sum(header[:148]) + 8*32 + sum(header[156:])
            if expected != actual: raise FoundationError('TAR_CHECKSUM_MISMATCH', offset=off)
            magic = header[257:265]
            if magic not in {b'ustar\x0000', b'ustar  \x00'}:
                raise FoundationError('TAR_FORMAT_UNSUPPORTED', offset=off)
            name = cstring(header[:100], off)
            if magic == b'ustar\x0000':
                prefix = cstring(header[345:500], off)
                if prefix: name = prefix + '/' + name
            typ = header[156:157] or b'\0'
            size = octal(header[124:136], 'TAR_SIZE_FORMAT', off)
            mode = octal(header[100:108], 'TAR_MODE_FORMAT', off)
            octal(header[108:116], 'TAR_UID_FORMAT', off)
            octal(header[116:124], 'TAR_GID_FORMAT', off)
            mtime = str(octal(header[136:148], 'TAR_MTIME_FORMAT', off))
            if size > self.limits.member_bytes: raise FoundationError('MEMBER_SIZE_LIMIT', offset=off)
            if typ in {b'L', b'K', b'x', b'g'} and size > self.limits.extension_bytes:
                raise FoundationError('TAR_EXTENSION_SIZE_LIMIT', offset=off)
            data_off = self.offset
            data = self.read(size) if size else b''
            pad = (-size) % 512
            if pad and self.read(pad).strip(b'\0'): raise FoundationError('TAR_NONZERO_MEMBER_PADDING', offset=off)
            if typ == b'L':
                if longname is not None: raise FoundationError('TAR_STACKED_LONGNAME', offset=off)
                if not data.endswith(b'\0') or b'\0' in data[:-1]: raise FoundationError('TAR_LONGNAME_FORMAT', offset=off)
                try: longname = data[:-1].decode('utf-8', 'strict')
                except UnicodeDecodeError as exc: raise FoundationError('TAR_PATH_UTF8_INVALID', offset=off) from exc
                if len(data) > self.limits.path_bytes+1: raise FoundationError('PATH_LENGTH_LIMIT', offset=off)
                long_offset = off
                self.controls.append({'header_offset': off, 'kind':'GNU_LONGNAME', 'header_sha256':digest(header), 'payload_sha256':digest(data), 'payload_bytes':size})
                continue
            if typ in {b'K', b'x', b'g', b'S'}:
                # Do not silently ignore semantics that may replace names/sizes.
                self.controls.append({'header_offset':off,'kind':'UNSUPPORTED_CONTROL','header_sha256':digest(header),'payload_sha256':digest(data),'payload_bytes':size})
                raise FoundationError('TAR_EXTENSION_UNSUPPORTED', offset=off)
            if longname is not None:
                name = longname
            key, issues = path_problem(name, self.limits)
            if key is not None:
                if key in seen: issues.append('DUPLICATE_MEMBER_PATH')
                seen[key] = off
                folded = unicodedata.normalize('NFC', key).casefold()
                if folded in portable and portable[folded] != key: issues.append('PATH_CASEFOLD_COLLISION')
                portable[folded] = key
            if mode & 0o7000: issues.append('SPECIAL_PERMISSION_BITS')
            member_type = {b'0':'file',b'\0':'file',b'5':'directory',b'1':'hardlink',b'2':'symlink',b'3':'character_device',b'4':'block_device',b'6':'fifo'}.get(typ,'unsupported')
            if member_type not in {'file','directory'}: issues.append('UNSAFE_MEMBER_TYPE')
            if member_type != 'file' and size: issues.append('NONFILE_PAYLOAD')
            self.member_count += 1
            if self.member_count > self.limits.members: raise FoundationError('MEMBER_COUNT_LIMIT', offset=off)
            yield Member(self.member_count-1, off, data_off, name, key, member_type, size, mode, mtime,
                         digest(data) if member_type == 'file' else None,
                         data if member_type == 'file' else None, issues, digest(header),long_offset)
            longname = None; long_offset = None


class Archive:
    """One descriptor anchors input identity and decompression. No network access."""
    def __init__(self, path: pathlib.Path, limits: Limits = Limits(), *, compression: str = 'zstd'):
        self.path, self.limits, self.compression = path, limits, compression
        self.sha256 = ''; self.bytes = 0; self.reader: TarReader | None = None
        self.zstd_version: str | None = None

    def members(self) -> Iterator[Member]:
        with open_source(self.path) as source:
            st = os.fstat(source.fileno()); self.bytes = st.st_size
            if self.bytes > self.limits.archive_bytes: raise FoundationError('COMPRESSED_SIZE_LIMIT')
            h = hashlib.sha256()
            for b in iter(lambda:source.read(65536), b''): h.update(b)
            self.sha256 = h.hexdigest(); source.seek(0)
            if self.compression == 'tar':
                self.reader = TarReader(source, self.limits)
                yield from self.reader
            elif self.compression == 'zstd':
                try:
                    v = subprocess.run(['zstd','--version'], capture_output=True, timeout=5, check=True)
                    self.zstd_version = v.stdout.decode('ascii','replace').strip()[:256]
                    process = subprocess.Popen(['zstd','-d','-q','-M128MB','-c'], stdin=source, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
                except (OSError, subprocess.SubprocessError) as exc: raise FoundationError('ZSTD_UNAVAILABLE') from exc
                expired = threading.Event()
                def kill() -> None:
                    expired.set(); process.kill()
                timer = threading.Timer(self.limits.decompression_seconds, kill); timer.daemon = True; timer.start()
                try:
                    assert process.stdout is not None
                    self.reader = TarReader(process.stdout, self.limits)
                    try: yield from self.reader
                    except FoundationError:
                        if expired.is_set(): raise FoundationError('DECOMPRESSION_TIME_LIMIT')
                        raise
                    rc = process.wait(timeout=5)
                    if expired.is_set(): raise FoundationError('DECOMPRESSION_TIME_LIMIT')
                    if rc: raise FoundationError('ZSTD_INVALID_OR_TRUNCATED')
                finally:
                    timer.cancel()
                    if process.poll() is None: process.kill()
                    process.wait(); process.stdout.close()
            else:
                raise FoundationError('COMPRESSION_MODE_UNSUPPORTED')
            # The final pass detects source replacement/modification during ingestion.
            source.seek(0); after = hashlib.sha256()
            for b in iter(lambda:source.read(65536),b''): after.update(b)
            if after.hexdigest() != self.sha256 or os.fstat(source.fileno()).st_size != self.bytes:
                raise FoundationError('SOURCE_CHANGED_DURING_READ')

    def receipt(self) -> dict:
        reader = self.reader
        return {'schema':'ReceiptArchiveSafetyV1','archive_sha256':self.sha256,'archive_bytes':self.bytes,
                'compression':self.compression,'zstd_version':self.zstd_version,'limits':dataclasses.asdict(self.limits),
                'logical_members':reader.member_count if reader else 0,
                'physical_headers':reader.header_count if reader else 0,
                'control_headers':reader.controls if reader else [],
                'decompressed_bytes':reader.offset if reader else 0,
                'padding_bytes':reader.padding_bytes if reader else 0,
                'extraction_performed':False,'source_modified':False}
