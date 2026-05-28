# P30 Known Limitations

- Full v11A/v11B conformance is not claimed. P30 added and revalidated executable seed coverage, not a complete release bar.
- P30 did not implement full process-group or job-object termination for command timeouts. Direct child kill errors are now surfaced; grandchild containment remains quarantined under P30-ABSORB-0017.
- `display_only_unstable_id` still exists for display/test/default DTO construction. The old `generated_artifact_id` symbol and random agency UUID path were removed, but a full material-ID migration remains P31 debt.
- `p30_guard` still reports warning-class dynamic JSON, `expect`, and related pattern debt. The final guard result is `hard=0`, not warning-free.
- Parent `make -C .. gate` is blocked by missing parent-root pack-truth documents outside the AiDENs-local runtime changes.
