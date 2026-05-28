# semantic-memory type notes for v14/v15

## First-pass persistence rule

Store canonical artifact JSON plus stable IDs first.
Normalize later only after query obligations are frozen.

## Questions the storage layer must be able to answer

### v14
- what intervention produced this decision?
- what cohort and comparability judgment were used?
- what counterfactual slice and refuters were involved?
- what decision trace authorized rollout or rollback?
- what cheap checks remained?

### v15
- what was exchanged?
- under what attestation envelope?
- under what trust root and admission policy?
- with what replay/disclosure class?
- what dispute or revocation state exists?

If any of those answers require “go read some logs”, the storage shape is still wrong.
