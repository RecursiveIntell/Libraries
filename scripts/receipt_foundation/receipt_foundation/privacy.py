"""Conservative candidates, not a claim of comprehensive secret detection."""
from __future__ import annotations
import re

PATTERNS = (
    ('private_key', re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH |DSA |ENCRYPTED )?PRIVATE KEY-----')),
    ('github_token', re.compile(rb'(?<![A-Za-z0-9_])(?:gh[pousr]_[A-Za-z0-9]{20,255}|github_pat_[A-Za-z0-9_]{20,255})(?![A-Za-z0-9_])')),
    ('aws_access_key', re.compile(rb'(?<![A-Z0-9])(?:AKIA|ASIA)[A-Z0-9]{16}(?![A-Z0-9])')),
    ('api_key_candidate', re.compile(rb'(?<![A-Za-z0-9])sk-(?:proj-|ant-)?[A-Za-z0-9_-]{20,255}(?![A-Za-z0-9_-])')),
    ('credential_assignment', re.compile(rb'''(?i)["'](?:api_key|access_token|refresh_token|secret_key|client_secret|password|authorization)["']\s*:\s*["']([^"'\r\n]{8,512})["']''')),
    ('bearer_credential', re.compile(rb'(?i)\bBearer[ \t]+[A-Za-z0-9._~+/=-]{16,512}')),
)
EMAIL = re.compile(rb'(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b')
PLACEHOLDER = re.compile(rb'(?i)^(?:redacted|\*+|<[^>]+>|\$\{[^}]+\}|your[_ -].*|example.*|placeholder.*)$')


def scan(raw: bytes) -> list[dict]:
    out=[]
    for kind, pattern in PATTERNS:
        for m in pattern.finditer(raw):
            group=1 if kind=='credential_assignment' else 0
            value=m.group(group)
            if PLACEHOLDER.fullmatch(value):continue
            out.append({'kind':kind,'byte_start':m.start(group),'byte_end':m.end(group),
                        'certainty':'candidate','rule':'secret-candidates-v1'})
    return sorted(out,key=lambda x:(x['byte_start'],x['kind'],x['byte_end']))


def scalar_safe(s: str) -> bool:
    try:b=s.encode('utf-8','strict')
    except UnicodeEncodeError:return False
    return len(b)<=4096 and not scan(b)


def sensitivity(raw: bytes, findings: list[dict]) -> str:
    if findings:return 'secret/credential'
    if EMAIL.search(raw):return 'sensitive'
    # Never make an automatic public-safe assertion.
    return 'unknown'
