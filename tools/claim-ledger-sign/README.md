# claim-ledger-sign

Operator-side signing tool for exact artifact bytes. It binds a file's
domain-separated SHA-256 into a `claim-ledger::ArtifactEnvelopeV1` and signs
the envelope identity fields with Ed25519 using a 32-byte seed file you supply.

## What it does

- **Reads** the artifact file (the exact bytes are what get bound — the tool
  never accepts a digest in place of bytes),
- **validates** the seed file is exactly 32 bytes (one trailing newline is
  tolerated for editor convenience),
- **signs** the envelope with `Ed25519KeyPair::from_seed_unchecked`,
- **prints** a receipt containing the artifact digest, signer id, policy id,
  trusted timestamp, signer **public key** (hex), and the complete signed
  envelope, and optionally writes the envelope JSON to `--out`.

## What it does NOT do

- It does not decide whether anything is *accepted*. Acceptance is a
  verifier-side configuration of the **public** key, authorized signer ids,
  the time window, and the admitted policy ids — see
  `claim_ledger::EnvelopeVerificationContext`.
- It does not store, print, transmit, or derive the signing seed.
- It does not certify claim truth; the signature binds bytes and asserts a
  policy, nothing more.

## Usage

```console
# one-time: create a key (this machine only; never commit it)
python3 -c "import secrets, sys; sys.stdout.buffer.write(secrets.token_bytes(32))" > <seed-dir>/operator-signing.key

# sign
claim-ledger-sign \
  --artifact path/to/claim-bundle-v2.json \
  --seed <seed-dir>/operator-signing.key \
  --trusted-timestamp 2026-09-29T18:50:00Z \
  --policy operator-admitted \
  --signer-id local-operator \
  --out path/to/envelope.json

# print the public key to install in the verifier context
claim-ledger-sign ... | jq -r .signer_public_key
```

Policy id currently asserted: `recursiveintell:operator-admitted-support:v1`.

## Tests

```console
cargo test -p claim-ledger-sign
```

This is a tool within the Libraries workspace, `publish = false`; it is not a
library API, not a release, and not an authority grant.