define_id!(
    /// Opaque identifier for one exported or imported support-set artifact.
    ///
    /// Stable for one snapshot of the support expression attached to a claim.
    SupportSetId
);

define_id!(
    /// Opaque identifier for one contradiction-witness artifact.
    ///
    /// Used to point at a stored explanation of why a claim is simultaneously
    /// supported and refuted within one semantics profile.
    ContradictionWitnessId
);

define_id!(
    /// Opaque identifier for one retraction / supersession artifact.
    ///
    /// This identifies the transaction-time event that closes currentness for
    /// a claim version without erasing historical visibility.
    RetractionRecordId
);
