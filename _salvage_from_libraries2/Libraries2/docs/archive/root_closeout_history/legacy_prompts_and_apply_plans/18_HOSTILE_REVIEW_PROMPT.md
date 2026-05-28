# Hostile review prompt — final v21/v24 pass

Review the final-pass implementation as if you expect it to be lying.

Check for:
- owner drift,
- schema/example mismatch,
- missing manifest coverage,
- effect paths without observation,
- delegation paths without revocation,
- release decisions without assurance,
- emergency exceptions without expiry or review,
- and any sign that a new v25-style theory jump was smuggled in.

Reject the pass if any of those are true.
