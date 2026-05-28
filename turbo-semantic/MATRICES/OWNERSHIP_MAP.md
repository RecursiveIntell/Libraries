# Ownership / Source-of-Truth Map

| Object / behavior | Canonical owner | Non-owner allowed behavior | Forbidden behavior |
|---|---|---|---|
| TurboQuant algorithm | `turbo-quant` | semantic-memory adapter calls public API | Reimplementing TurboQuant math in semantic-memory |
| Polar encoding | `turbo-quant/src/polar.rs` or successor | none | local duplicate polar quantizer |
| QJL sketching | `turbo-quant/src/qjl.rs` or successor | none | local random projection/sign logic |
| Rotation | `turbo-quant/src/rotation.rs` or successor | select profile only | local matrix/SRHT hidden copy |
| Codec profile digest | `turbo-quant` | semantic-memory stores and references digest | semantic-memory invents unrelated digest semantics |
| Raw embedding storage | `semantic-memory` | turbo-quant never writes DB | turbo-quant modifies memory DB |
| SQ8 codec | `semantic-memory` | turbo adapter may compare to it | remove/weaken SQ8 without explicit migration |
| Search ranking/result APIs | `semantic-memory` | turbo codec supplies approximate scores | turbo-quant directly owns search policy |
| Evaluation episodes/records | `semantic-memory` | turbo reports codec metrics | turbo-quant silently decides production promotion |
| Execution/encode receipts | `semantic-memory` for storage; `turbo-quant` for encode metadata | both expose data | logs-only evidence |
