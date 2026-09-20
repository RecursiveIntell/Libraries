# Falsification seed corpus

Seven synthetic finite-dimensional cases motivated by constitutive residual
analysis: energy sign, feasible nonnegative control, operator range, coupled
material transport, search-budget exhaustion, and two exact periodic-shear
matrices. This is **not** a broad model benchmark or an OpenAI singularity
reconstruction. All cases and witnesses are public ground truth; no model
accuracy, discovery, speedup, or novelty claim is justified by passing them.

```sh
python receipt-bench/fixtures/falsify/check_corpus.py
cargo build --manifest-path constitutive-witness/Cargo.toml --locked --offline
python receipt-bench/fixtures/falsify/check_corpus.py --binary constitutive-witness/target/debug/constitutive-witness
python receipt-bench/fixtures/falsify/check_corpus.py --export-case energy-sign --out /tmp/new-cw-case
```

The no-binary mode checks static witnesses only and reports that limitation.
The binary mode invokes the actual compiled CLI and independently rechecks the
returned rational certificates. The exported case can be used by ClaimLedger's
science CLI and the Ares optional falsify skill. Fixture statements have empty
source references intentionally: they claim only the explicit synthetic matrix
problem, not a source-to-continuum theorem mapping.

The harness executes an explicitly selected trusted binary; it is a test tool,
not a hostile-executable sandbox. The Ares runner owns streamed output/deadline
controls for operator workflows. The static test witnesses' attempt metadata
is illustrative, not a measured native search count.
