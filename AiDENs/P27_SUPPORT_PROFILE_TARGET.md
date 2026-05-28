# P27 Support Profile Target

## supported-local target

- `aidens agent validate|doctor|run|inspect|new` for local `AgentSpecV1` agents.
- `scripts/verify_current.sh` as working current verifier.
- Mock-provider Plan→Act→Verify path runnable without cloud credentials.
- Durable file-backed run receipt store for `AiDENsRunBundleV3` or successor local evidence.
- Local coding-agent sandbox tools with permit-gated writes/checks and patch receipts.
- `aidens inspect-run` can inspect durable receipts.

## partial target

- Local Ollama smoke path where available, skip-safe otherwise.
- Memory-grounded agents through canonical adapter route with backpointers/degradation labels.
- Repair/abstention display records as AiDENs-local operator evidence.
- Contracts/CLI megafile containment.

## fixture-backed target

- AgentSpec fixtures.
- RunBundle fixtures.
- Patch engine hostile cases.
- Agency eval cases.
- Memory-grounding fixture cases.

## deferred-cloud

- Hosted provider execution requiring API keys.
- Native provider tool loops over hosted services.
- Production streaming loops.

## deferred-autonomy

- Broad autonomous daemon scheduling.
- Multi-run autonomous operations beyond explicitly tested safe-mode/local queue fixtures.

## design-only

- V10 regional runtime geometry.
- V11 full proof-governed runtime.
- V12 regional protocol.
- Federation/mechanism/theory runtime.
