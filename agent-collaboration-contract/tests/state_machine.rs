use agent_collaboration_contract::TaskStatusV1;

#[test]
fn lifecycle_transition_table_is_explicit() {
    assert!(TaskStatusV1::Submitted.can_transition_to(TaskStatusV1::Accepted));
    assert!(TaskStatusV1::Running.can_transition_to(TaskStatusV1::CompletionUnknown));
    assert!(TaskStatusV1::CompletionUnknown.can_transition_to(TaskStatusV1::ReconciledCompleted));
    assert!(!TaskStatusV1::Completed.can_transition_to(TaskStatusV1::Running));
    assert!(TaskStatusV1::Completed.is_terminal());
    assert!(TaskStatusV1::Quarantined.is_terminal());
}
