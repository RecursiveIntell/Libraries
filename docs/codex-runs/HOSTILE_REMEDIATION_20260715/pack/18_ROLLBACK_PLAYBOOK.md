# Rollback playbook

Rollback restores executable behavior while preserving canonical/history data.

## Source

One revertable commit/range per task; phase tags; no history rewrite after receipts.

## IDs

Retain legacy columns/fields; alias mappings; dual-read; stop new canonical writes by feature/config
if necessary; never mint a second canonical ID for the same material.

## Digests/ledger

Preserve V1 bytes/verifier; write V2 only after activation; rollback readers without deleting V2
or supersession/head receipts.

## Codecs

Disable derived/lossy backends; read raw authority; invalidate/rebuild sidecars; preserve encoded
artifacts for forensics; never reinterpret envelopes.

## Queues

Stop executors before schema/connection rollback; preserve job/owner state; reconcile claimed work;
never reset processing blindly.

## Evidence/CI

Reverting a gate also reverts release-ready status. Old receipts are superseded, never overwritten.
