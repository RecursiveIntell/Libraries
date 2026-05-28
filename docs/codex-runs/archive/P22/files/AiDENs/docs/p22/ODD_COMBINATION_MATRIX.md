# Odd Combination Matrix

| Combination | Failure if unfenced | Required behavior |
|---|---|---|
| disabled provider + run | fake answer | hard error, receipt/warning truth |
| mock provider + production profile | accidental fake success | mock only when explicit config/profile says mock |
| provider advertises native tools + unknown kind | route lie | degraded/openai-compatible only with reason code |
| parser fallback + tool call | hidden degradation | fallback route receipt |
| repo-read + path traversal | sandbox escape | reject before read |
| dangerous tool + no approval | local damage | absent or blocked with permit failure |
| profile coding-agent + shell | magic grant | shell requires approval, disabled by default |
| doctor + deferred advanced crates | false green | deferred/disabled section status |
| generated app + internal crates | app complexity returns | facade-only app template |
| config changes mid-run | mixed truth | run pins config generation |
| memory scope + coding repo | personal memory contamination | memory disabled/optional unless explicit |
| queue retry + provider failover | duplicate logical job | defer until queue kit; never fake current support |
