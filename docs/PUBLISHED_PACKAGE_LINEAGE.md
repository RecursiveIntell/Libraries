# Historical published-package source index

This index distinguishes registry bytes, recorded historical source and current package identity. It does not certify a clean reproduction or current maintenance.

| Registry package | SHA-256 of verified registry tarball | Recorded historical source | Current identity boundary |
|---|---|---|---|
| [`agent-queue 0.2.0`](https://crates.io/crates/agent-queue/0.2.0) | `10559f4e8e95693b2f42f4626ac1a06eee330f8e0580dae5c7c1662e41fcd92e` | [job-queue at 7dbe1cf1631a](https://github.com/RecursiveIntell/Libraries/tree/7dbe1cf1631af50c0bd73eb8e29f969ee4a2148e/job-queue) | Current path declares `job-queue`, not `agent-queue`; no renaming/equivalence asserted. |
| [`aidens-autonomous 0.1.0`](https://crates.io/crates/aidens-autonomous/0.1.0) | `a867a17a34eb2a910358fae0b928b6c675d53acdf062ea0868be3973cb2eca76` | [AiDENs/crates/aidens-autonomous at 62fc139c9f0a](https://github.com/RecursiveIntell/Libraries/tree/62fc139c9f0ab1c84eedd45a68edc13fd20ee57e/AiDENs/crates/aidens-autonomous) | Recorded package path absent at observed Libraries default-branch revision; no restoration inferred. |
| [`aidens-memory-tools 0.1.0`](https://crates.io/crates/aidens-memory-tools/0.1.0) | `e518f289985626b5302be0bba161ce8bebb2d9c59f39a330c2c1aba7f0216d8e` | [AiDENs/crates/aidens-memory-tools at 62fc139c9f0a](https://github.com/RecursiveIntell/Libraries/tree/62fc139c9f0ab1c84eedd45a68edc13fd20ee57e/AiDENs/crates/aidens-memory-tools) | Recorded package path absent at observed Libraries default-branch revision; no restoration inferred. |
| [`aidens-tui 0.1.0`](https://crates.io/crates/aidens-tui/0.1.0) | `0dfd022fbb7140c598bf158cec59a6157a97a459abd0e6132336a5480ea00943` | [AiDENs/crates/aidens-tui at 62fc139c9f0a](https://github.com/RecursiveIntell/Libraries/tree/62fc139c9f0ab1c84eedd45a68edc13fd20ee57e/AiDENs/crates/aidens-tui) | Recorded package path absent at observed Libraries default-branch revision; no restoration inferred. |

## Evidence boundaries

The four archives were downloaded from the registry and matched the registry-reported SHA-256 values. Each archive records `dirty: true` in `.cargo_vcs_info.json`. The recorded historical paths exist in Libraries, but the published original manifests and additional packaged source files differ from those Git snapshots. A matching archive checksum verifies registry bytes, not the correctness of the software or equivalence to a clean checkout.

Cargo-generated/normalized manifests, lockfiles and VCS metadata were classified separately from original source files. The inspected local history is shallow; an empty local history listing is not evidence that earlier ownership or source never existed.

These references preserve historical discoverability. They do not introduce four maintained packages into the current workspace, rewrite old registry metadata, or establish byte-reproducible builds. Old registry versions remain historical; correcting package metadata on the registry requires a separately qualified new version.

Observed Libraries default branch: `9f2a2c08659b7ded321a68741d5d8be76a2051db`. Registry and historical source comparisons were made October 11, 2026. No build scripts from the downloaded archives were executed.
