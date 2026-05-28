# boundary-compiler

RFC 8785 JSON Canonicalization (JCS) implementation with boundary profiles and duplicate-key rejection.

## Overview

This crate implements [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html) — JSON Canonicalization Scheme (JCS):
- Canonicalizer that serializes JSON into deterministic byte sequences
- Duplicate-key rejection (RFC 8785 mandates duplicate object keys are errors)
- blake3 Content-Digest of the JCS string
- Boundary profiles for dialect, schema ID+version, canonicalization profile, unknown-field policy, and resource ceilings
- Optional JSON Schema validation before/after canonicalization

## Integration

Replaces `semantic_memory::graph::canonical_json_string()` with a standards-compliant JCS implementation.

## Modules

- `canonicalizer` — Core RFC 8785 JCS canonicalization (strict JSON parser, duplicate-key detection, RFC 8785 escaping rules)
- `digest` — blake3 Content-Digest computation over JCS bytes
- `profile` — BoundaryProfile enum with dialect, schema ID+version, canonicalization profile, unknown-field policy, resource ceilings
- `schema` — Optional JSON Schema validation before/after canonicalization
