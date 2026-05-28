//! Runtime capability truth surfaces.

use aidens_contracts::{
    CapabilityGateDecisionV1, CapabilityGateOutcomeV1, CapabilityStateV1, RuntimeCapabilityTruthV1,
    ToolLifecycleStateV1,
};

pub fn truth(
    capability_id: impl Into<String>,
    states: Vec<CapabilityStateV1>,
    reason: Option<String>,
) -> RuntimeCapabilityTruthV1 {
    RuntimeCapabilityTruthV1 {
        capability_id: capability_id.into(),
        states,
        reason,
    }
}

pub fn disabled(
    capability_id: impl Into<String>,
    reason: impl Into<String>,
) -> RuntimeCapabilityTruthV1 {
    RuntimeCapabilityTruthV1 {
        capability_id: capability_id.into(),
        states: vec![CapabilityStateV1::Disabled],
        reason: Some(reason.into()),
    }
}

pub fn healthy(capability_id: impl Into<String>) -> RuntimeCapabilityTruthV1 {
    RuntimeCapabilityTruthV1 {
        capability_id: capability_id.into(),
        states: vec![
            CapabilityStateV1::Configured,
            CapabilityStateV1::Available,
            CapabilityStateV1::Healthy,
        ],
        reason: None,
    }
}

pub fn executable_this_turn(capability: &RuntimeCapabilityTruthV1) -> bool {
    capability
        .states
        .contains(&CapabilityStateV1::ExecutableThisTurn)
        && !capability.states.contains(&CapabilityStateV1::Disabled)
        && !capability
            .states
            .contains(&CapabilityStateV1::BlockedByPolicy)
}

pub fn gate_decision(
    capability_id: impl Into<String>,
    outcome: CapabilityGateOutcomeV1,
    lifecycle: Vec<ToolLifecycleStateV1>,
    executable_this_turn: bool,
    reason_codes: Vec<String>,
) -> CapabilityGateDecisionV1 {
    CapabilityGateDecisionV1::new(
        capability_id,
        outcome,
        lifecycle,
        executable_this_turn,
        reason_codes,
    )
}

pub fn gate_exposes(decision: &CapabilityGateDecisionV1) -> bool {
    decision.outcome == CapabilityGateOutcomeV1::Exposed
        && (decision
            .lifecycle
            .contains(&ToolLifecycleStateV1::ExposedThisTurn)
            || decision.lifecycle.contains(&ToolLifecycleStateV1::Exposed))
        && decision.executable_this_turn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_is_not_healthy() {
        let state = disabled("web", "local-only mode");
        assert!(state.states.contains(&CapabilityStateV1::Disabled));
        assert!(!state.states.contains(&CapabilityStateV1::Healthy));
        assert!(!executable_this_turn(&state));
    }

    #[test]
    fn can_represent_registered_exposed_executable_degraded_and_blocked() {
        let state = truth(
            "tool:test:1",
            vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::Available,
                CapabilityStateV1::Registered,
                CapabilityStateV1::ExposedThisTurn,
                CapabilityStateV1::ExecutableThisTurn,
                CapabilityStateV1::Degraded,
            ],
            Some("parser fallback active".into()),
        );
        assert!(executable_this_turn(&state));

        let blocked = truth(
            "tool:test:1",
            vec![
                CapabilityStateV1::Registered,
                CapabilityStateV1::ExecutableThisTurn,
                CapabilityStateV1::BlockedByPolicy,
            ],
            Some("missing permit".into()),
        );
        assert!(!executable_this_turn(&blocked));
    }

    #[test]
    fn gate_decision_requires_exposed_lifecycle_to_expose() {
        let decision = gate_decision(
            "tool:test:1",
            CapabilityGateOutcomeV1::Exposed,
            vec![
                ToolLifecycleStateV1::Declared,
                ToolLifecycleStateV1::Registered,
                ToolLifecycleStateV1::Executable,
                ToolLifecycleStateV1::ExposedThisTurn,
            ],
            true,
            Vec::new(),
        );

        assert!(gate_exposes(&decision));
    }
}
