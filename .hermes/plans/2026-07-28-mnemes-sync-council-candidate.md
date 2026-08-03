# Unified Report: Protocol, Identity, and Operations Architecture for Mnemes Synchronization

## Executive Summary
This report synthesizes the findings from the Protocol & Identity Architecture (ws0) and the Rollout, Verification, and Operations Plan (ws2) for the Mnemes synchronization feature in the `semantic-memory` stack. The current system is limited to local primitives and lacks the wire protocols, cryptographic identity, and operational safety nets required for distributed synchronization. To bridge this gap, this report outlines a unified architecture combining robust transport and identity layers with a strictly gated, phased rollout strategy.

---

## 1. Current State and Observed Gaps

The current `semantic-memory` / `semantic-memory-mcp` stack is local-only and lacks end-to-end synchronization capabilities. The following critical gaps have been observed:

*   **Networking & Transport:** No network client or pluggable transport abstraction exists. 
*   **Identity & Authentication:** `--mnemes-device-id` and `--mnemes-store-id` are untrusted CLI strings with no cryptographic binding or peer authentication.
*   **Protocol:** No batch envelope, per-entry acknowledgments, or protocol version negotiation exists. Sync is currently one-record-at-a-time.
*   **State & Watermarks:** No durable record of the last successfully replicated sequence (watermark) survives a crash.
*   **Operational Safety:** Live store mutation without backup or explicit operator approval is a major identified failure mode that must be prevented.

---

## 2. Proposed Technical Architecture (Protocol & Identity)

To support safe, distributed synchronization, the following architecture is proposed:

### 2.1 Transport Layer
**[proposed]** Introduce a `SyncTransport` trait to decouple framing from journal and inventory logic. 
*   **Implementations:** `LocalLoopbackTransport` (for tests), `HttpTransport` (HTTP/1.1 + TLS for server-to-server), and `StdioTransport` (deferred for CLI-to-daemon).
*   **Contract:** Must expose `send(frame)`, `stream(frames)`, and a strict `max_frame_bytes` ceiling with fail-closed behavior on overflow.

### 2.2 Peer Authentication
**[proposed]** Mutual-TLS (mTLS) with device certificates is the primary authentication mechanism.
*   **Identity:** `--mnemes-device-id` becomes a derived identifier: `did:mnemes:<base32(pubkey)>`. Free-form device IDs are rejected at startup.
*   **Provisioning:** A short-lived bootstrap token authorizes the first connection, after which the server pins the device cert fingerprint. Server identity is verified via TOFU (Trust On First Use) with an explicit `--mnemes-server-pin` flag.
*   **Authorization:** Application-layer authorization (which stores a device may touch) is enforced *after* mTLS using store-id scope rules.

### 2.3 Batch Protocol & Watermarks
**[proposed]** A `BatchEnvelope` framed as a length-prefixed CBOR or MessagePack blob.
*   **Structure:** Contains a signed header (protocol version, device/store IDs, timestamp) and a body containing up to 1,024 `BatchEntry` items (max 256 KiB each, 8 MiB total envelope limit).
*   **Acks:** Per-entry acknowledgments (`BatchAck`) are required so a single quarantined entry does not force retransmission of the entire batch.
*   **Watermarks:** A durable watermark store must be introduced to track the `high_watermark` (last-acked seq) per peer, surviving crashes.

---

## 3. Rollout and Operations Strategy

To transition from local primitives to a live device/server sync service without risking data loss, a strict, phased rollout is required.

### 3.1 Rollout Phases
The rollout is **opt-in, reversible, and observable**. The local store remains the system of record until Phase 4.

1.  **Phase 0-1:** Foundation, interfaces, and local-only primitive hardening. (No live mutations).
2.  **Phase 2:** Shadow-store sync (device → staging). Writes go to a shadow store.
3.  **Phase 3:** Reconcile shadow vs. live (read-only diff).
4.  **Phase 4:** Controlled cutover (shadow → live). **Mandatory backup and approval required.**
5.  **Phase 5:** Continuous daemon sync (live).
6.  **Phase 6:** Multi-device / server authoritative sync.

### 3.2 Backup and Approval Mechanism
**[proposed]** Define an explicit `SyncApproval` artifact for any phase that mutates state.
*   **Requirements:** Must include a content-addressed `backup_ref`, `backup_verified` status (verified via a restore-drill), operator ID, and an expiry timestamp.
*   **