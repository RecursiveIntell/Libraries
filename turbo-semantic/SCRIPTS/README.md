# Assertion Scripts

Copy these scripts into the workspace `scripts/` directory or run them from this bundle.

Recommended use from `/home/sikmindz/Coding/Libraries`:

```bash
bash path/to/SCRIPTS/run_superpass_checks.sh .
```

If `turbo-quant` remains outside the Libraries workspace, run its hardening script separately:

```bash
bash path/to/SCRIPTS/assert_turbo_quant_hardening.sh /home/sikmindz/Documents/turbo-quant
```

The scripts are intentionally conservative. A false positive should be investigated, not ignored.
