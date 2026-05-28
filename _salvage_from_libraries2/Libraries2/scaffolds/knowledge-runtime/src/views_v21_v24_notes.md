# v21/v24 runtime view notes

Add bounded read/query surfaces for:

- active effect intents by execution window and reversibility class,
- currently valid authority leases and revocations,
- deployability state by deployment profile and assurance readiness,
- open incidents, recovery plans, and replay completeness.

Every view should preserve:
- advisory vs admitted state,
- missing backpointer visibility,
- and replay/degradation markers.
