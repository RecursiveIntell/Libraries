
# Risk register — post-v24 profiles

## R1 — Fake v25 inflation
The biggest strategic risk is marketing profile work as a new base-spec wave.

**Mitigation:** keep the no-v25 rule in root docs and release bar.

## R2 — Policy scatter
Profile families may get spread across too many crates and lose a canonical home.

**Mitigation:** one primary owner per family, with consumers named explicitly.

## R3 — SaaS folklore
Pager, approval, or vendor behaviors may remain hidden in third-party tooling.

**Mitigation:** require typed local profile artifacts plus fixture coverage.

## R4 — Boundary semantics drift
Residency and privacy exceptions can silently become permanent.

**Mitigation:** every exception gets expiry, approvals, and post-hoc review requirements.

## R5 — Hazard taxonomy without operations
A hazard library with no monitor or playbook linkage becomes decorative.

**Mitigation:** P5 requires monitor and mitigation families, not just labels.

## R6 — Translation laundering
Vendor adapters may silently drop caveats.

**Mitigation:** require lossiness declaration and revocation handling artifacts.

## R7 — Documentation truth drift
The repo may teach “profile” as if it were “base law”.

**Mitigation:** documentation-truth update is a release gate.
