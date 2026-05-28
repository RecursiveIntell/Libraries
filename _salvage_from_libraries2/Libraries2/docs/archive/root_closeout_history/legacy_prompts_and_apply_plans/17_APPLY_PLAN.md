# Apply plan

## Recommended adoption path
1. keep this pack external first
2. use the issue matrix as the v10 queue
3. merge implementation/test pieces into existing repo surfaces, not as another root-doc dump
4. after proof surfaces land, create a single canonical `docs/v10/` directory
5. keep root entrypoints minimal

## Merge targets
- geometry plan -> current v10 docs
- implementation sequence -> existing phased plan
- conformance plan -> current conformance docs
- benchmark plan -> benchmark docs
- prompts -> ops/prompts directory, not root
