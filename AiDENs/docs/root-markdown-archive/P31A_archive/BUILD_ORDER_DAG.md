# Build Order DAG

```mermaid
flowchart TD
  P00[P00 Source lock and fake-ready freeze] --> P01[P01 API honesty]
  P01 --> P02[P02 Provider runtime truth]
  P02 --> P03[P03 Turn executor/tool loop]
  P03 --> P04[P04 Capability/permit/approval]
  P03 --> P05[P05 Durable receipts]
  P04 --> P05
  P05 --> P06[P06 Boundary compiler]
  P06 --> P07[P07 Schema registry/migration]
  P07 --> P08[P08 Reference interpreters]
  P05 --> P09[P09 Episode memory]
  P08 --> P09
  P04 --> P10[P10 Coding tools/sandbox]
  P05 --> P10
  P09 --> P11[P11 Queue/schedule/daemon]
  P05 --> P11
  P09 --> P12[P12 Verification/repair/governance]
  P08 --> P12
  P12 --> P13[P13 Multi-view runtime]
  P10 --> P14[P14 Product surface]
  P13 --> P14
  P12 --> P15[P15 Regional decoder]
  P13 --> P15
  P15 --> P16[P16 Lawful subtraction]
  P12 --> P17[P17 Attested federation]
  P16 --> P17
  P12 --> P18[P18 Mechanism/theory runtime]
  P15 --> P18
  P14 --> P19[P19 Final release audit]
  P17 --> P19
  P18 --> P19
```

## Critical path

P00 -> P01 -> P02 -> P03 -> P05 -> P06 -> P07 -> P08 -> P09 -> P12 -> P13 -> P14 -> P19

## Parallelizable after gates

- P04 can partially proceed after P02 but must integrate with P03/P05.
- P10 can proceed after P04/P05.
- P11 can proceed after P05/P09.
- P15-P18 are deliberately late.
