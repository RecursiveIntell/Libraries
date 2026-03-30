# PRIMITIVES_SPLIT

## Crate List

External primitive crates created under `/home/sikmindz/Coding/Libraries/Primitives/`:

| Crate | Path |
| --- | --- |
| `typed-patch` | `/home/sikmindz/Coding/Libraries/Primitives/typed-patch` |
| `sandbox-workspace` | `/home/sikmindz/Coding/Libraries/Primitives/sandbox-workspace` |
| `check-runner` | `/home/sikmindz/Coding/Libraries/Primitives/check-runner` |
| `effect-signature` | `/home/sikmindz/Coding/Libraries/Primitives/effect-signature` |
| `mindstate-core` | `/home/sikmindz/Coding/Libraries/Primitives/mindstate-core` |
| `stabilizer-core` | `/home/sikmindz/Coding/Libraries/Primitives/stabilizer-core` |
| `forge-policy` | `/home/sikmindz/Coding/Libraries/Primitives/forge-policy` |
| `cea-core` | `/home/sikmindz/Coding/Libraries/Primitives/cea-core` |
| `cea-store` | `/home/sikmindz/Coding/Libraries/Primitives/cea-store` |
| `cea-sqlite` | `/home/sikmindz/Coding/Libraries/Primitives/cea-sqlite` |

## Mapping

| Old module / file | New primitive crate | New path |
| --- | --- | --- |
| `src/error.rs` (`Violation`, `ViolationKind`) | `forge-policy` | `/home/sikmindz/Coding/Libraries/Primitives/forge-policy/src/lib.rs` |
| `src/invariants.rs` (generic DB/path/caps helpers) | `forge-policy` | `/home/sikmindz/Coding/Libraries/Primitives/forge-policy/src/lib.rs` |
| `src/runtime/patch/types.rs` | `typed-patch` | `/home/sikmindz/Coding/Libraries/Primitives/typed-patch/src/lib.rs` |
| `src/runtime/patch/validate.rs` | `typed-patch` | `/home/sikmindz/Coding/Libraries/Primitives/typed-patch/src/lib.rs` |
| `src/runtime/patch/apply.rs` | `typed-patch` + `sandbox-workspace` | `/home/sikmindz/Coding/Libraries/Primitives/typed-patch/src/lib.rs` |
| `src/runtime/patch/render_diff.rs` | `typed-patch` | `/home/sikmindz/Coding/Libraries/Primitives/typed-patch/src/lib.rs` |
| `src/exec/backend.rs` | `check-runner` | `/home/sikmindz/Coding/Libraries/Primitives/check-runner/src/lib.rs` |
| `src/exec/host.rs` | `check-runner` | `/home/sikmindz/Coding/Libraries/Primitives/check-runner/src/lib.rs` |
| `src/exec/container.rs` | `check-runner` | `/home/sikmindz/Coding/Libraries/Primitives/check-runner/src/lib.rs` |
| `src/exec/backend.rs` (`Workspace`, `PatchedWorkspace`) | `sandbox-workspace` | `/home/sikmindz/Coding/Libraries/Primitives/sandbox-workspace/src/lib.rs` |
| `src/exec/backend.rs` (`EffectSignature`, `LocatedEffect`) | `effect-signature` | `/home/sikmindz/Coding/Libraries/Primitives/effect-signature/src/lib.rs` |
| `src/runtime/mindstate.rs` | `mindstate-core` | `/home/sikmindz/Coding/Libraries/Primitives/mindstate-core/src/lib.rs` |
| `src/runtime/novelty.rs` | `stabilizer-core` | `/home/sikmindz/Coding/Libraries/Primitives/stabilizer-core/src/lib.rs` |
| `src/runtime/stabilizer.rs` | `stabilizer-core` | `/home/sikmindz/Coding/Libraries/Primitives/stabilizer-core/src/lib.rs` |
| `src/cea/instrumentation.rs` | `cea-core` | `/home/sikmindz/Coding/Libraries/Primitives/cea-core/src/lib.rs` |
| `src/cea/graph.rs` | `cea-core` | `/home/sikmindz/Coding/Libraries/Primitives/cea-core/src/lib.rs` |
| `src/cea/predictor.rs` | `cea-core` | `/home/sikmindz/Coding/Libraries/Primitives/cea-core/src/lib.rs` |
| `src/cea/store.rs` | `cea-store` + `cea-sqlite` | `/home/sikmindz/Coding/Libraries/Primitives/cea-store/src/lib.rs` |

## Dependency Graph

- `forge-policy` is the lowest-level policy rail crate.
- `sandbox-workspace` depends on `forge-policy`.
- `typed-patch` depends on `forge-policy` and `sandbox-workspace`.
- `effect-signature` is independent.
- `check-runner` depends on `forge-policy`, `sandbox-workspace`, and `effect-signature`.
- `mindstate-core` is independent.
- `stabilizer-core` depends on `typed-patch`.
- `cea-core` depends on `typed-patch` and `check-runner`.
- `cea-store` depends on `cea-core` and `check-runner`.
- `cea-sqlite` depends on `cea-store` (and keeps sqlite-specific persistence isolated).
- `forge-engine` depends on all primitives via `path` dependencies and keeps the
  public surface stable via shims and re-exports.

## Feature Flags

- `forge-engine`
  - `danger-sm-write`: unchanged, still off by default, and still only gates the
    compatibility-only direct memory import escape hatch.
- `check-runner`
  - `container`: enables the container backend implementation. The machine crate enables this explicitly.

No new default feature enables dangerous behavior.

## Compatibility

- Existing public module paths remain valid.
- The machine crate keeps `src/runtime/patch/*`, `src/exec/*`, `src/runtime/mindstate.rs`, `src/runtime/novelty.rs`, `src/runtime/stabilizer.rs`, and `src/cea/*` as compatibility shims.
- `ForgeStore`, lab modules, config, and the Forge-specific invariants remain in the machine crate.
- Safety behavior is unchanged: `ForgeStore::open()` still calls `invariants::refuse_to_open_db()` first, the forge DB identity check is still enforced, and `danger-sm-write` semantics were not relaxed.

## Validation

Commands used during this split:

```bash
cargo test -p forge-engine
cargo test                    # in each primitive crate directory
cargo clippy -p forge-engine -- -D warnings
```
