
# Release bar and acceptance — post-v24 profiles

A release may call this pass landed only if all of the following are true:

## Mandatory
- v24 remains explicitly described as the terminal general-purpose base horizon;
- no speculative `v25` is advertised or implied;
- every profile family in the registry has:
  - one schema,
  - one example,
  - one owner,
  - one fixture presence,
  - one conformance note;
- the taught repo surface describes these as **profiles / overlays**, not new truth planes;
- vendor translations preserve caveats and lossiness explicitly;
- locality exceptions are time-bounded and reviewable;
- approval and conflict rules are typed, not PagerDuty folklore;
- hazard libraries point to monitors and mitigation playbooks rather than sitting as static taxonomy;
- incident routing and escalation clocks are replayable artifacts rather than external-only service configuration.

## Disqualifying failures
- a profile family exists only in prose,
- a vendor adapter flattens external assurance into score-only local truth,
- a boundary or residency exception has no expiry or review path,
- approval rules live only in human memory or SaaS configuration,
- a hazard library has no operational binding,
- docs imply a new base-spec wave when only profiles were added.

## Truth rule

Status claims for this pass must cite:
- published schemas/examples,
- fixture bundle presence,
- conformance notes,
- and root release-bar docs.

Prose alone is not enough.
