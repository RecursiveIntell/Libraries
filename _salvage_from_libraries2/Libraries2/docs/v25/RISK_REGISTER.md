# Risk register — v25 repo pack

| ID | Risk | Why it is dangerous | Mitigation |
|---|---|---|---|
| R1 | Historical doc contradiction | Readers land in the older no-v25 pack and miss the current v25 seam. | Supersession note + updated entry points + docs/v25 pack. |
| R2 | Local recomposition drift | Consumers quietly rebuild profile interaction privately. | Keep `profile-runtime` central; document next consumer deltas explicitly. |
| R3 | Fixture poverty | The four baseline bundles under-describe real consumer pressure. | Expand the corpus and add a fixture manifest. |
| R4 | Mirror drift | `libraries-source/` stops matching the active repo root. | Replace the hand-maintained path list with whole-tree sync. |
| R5 | Toolchain gap | JSON files exist but schemas/tests are not regenerated or executed. | Ship local-check wrapper and state the gap explicitly in release docs. |
| R6 | Fake completion | The pack is mistaken for full v25 consumer adoption. | Gap report and implementation status remain explicit about downstream work. |
| R7 | V26 leakage | Readers interpret v26 as active law rather than horizon work. | Keep v26 spec clearly marked horizon-only and advisory. |
