# HyperQuant README + crates.io Publish Receipt — 2026-06-26

Repo: /home/sikmindz/Coding/Libraries
Crate: /home/sikmindz/Coding/Libraries/hyperquant
Published crate: https://crates.io/crates/hyperquant/0.1.0
Docs target: https://docs.rs/hyperquant

## What changed

The `hyperquant` README was rewritten from an 18-line stub into a 290-line release-quality README modeled on the semantic-memory-mcp standard: clear purpose, install instructions, API overview, examples, claim boundaries, verification receipts, integration path, roadmap, and license section.

Files changed:
- `hyperquant/README.md`
- `hyperquant/Cargo.toml`

Cargo metadata fix:
- Added `LICENSE-MIT` and `LICENSE-APACHE` to the package `include` list so the published tarball contains the license files.

## README sections verified

- `# hyperquant`
- `## What this gives you`
- `## Claim boundary`
- `## Install`
- `## Quick start`
- `## API overview`
- `## Error handling`
- `## Integration path`
- `## Verification`
- `## Roadmap`
- `## License`

Receipt:

```text
wc -l hyperquant/README.md
290 hyperquant/README.md
```

## Pre-publish verification

```text
cargo fmt -p hyperquant: PASS
cargo test -p hyperquant -- --nocapture: PASS, 18 tests
cargo check -p hyperquant --all-targets: PASS
cargo clippy -p hyperquant --all-targets -- -D warnings: PASS
cargo publish -p hyperquant --dry-run --allow-dirty: PASS
cargo package -p hyperquant --allow-dirty --list: PASS, package includes README and both license files
readme quality scan: PASS
```

Package dry-run receipt:

```text
Packaged 15 files, 36.6KiB (11.8KiB compressed)
Verifying hyperquant v0.1.0
Finished `dev` profile
Uploading hyperquant v0.1.0
warning: aborting upload due to dry run
```

## Publish receipt

Command:

```bash
cargo publish -p hyperquant --allow-dirty
```

Result:

```text
Uploaded hyperquant v0.1.0 to registry `crates-io`
Published hyperquant v0.1.0 at registry `crates-io`
```

Registry verification:

```text
cargo search hyperquant --limit 5
hyperquant = "0.1.0"    # Experimental lattice quantization primitives with explicit receipts and conservative claim boundaries
```

```text
cargo info hyperquant --registry crates-io
version: 0.1.0
crates.io: https://crates.io/crates/hyperquant/0.1.0
```

## Downstream unblock

After publishing `hyperquant`, this now passes:

```text
cargo publish -p quant-eval --dry-run --allow-dirty: PASS
```

Important: actual `quant-eval` publish was not run because `quant-eval@0.1.0` already exists on crates.io. To publish the quant-eval integration, bump quant-eval first.

## Claim boundary preserved

The README explicitly says `hyperquant` does not claim:
- HyperQuant paper parity;
- model-quality preservation;
- production readiness;
- CUDA support;
- HuggingFace integration;
- superiority over GPTQ/AWQ/TurboQuant/FibQuant or any other codec.
