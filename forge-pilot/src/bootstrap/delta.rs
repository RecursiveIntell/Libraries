pub(crate) fn compute_manifest_delta(
    previous: Option<&crate::bootstrap::types::BootstrapManifestSnapshot>,
    current: &crate::bootstrap::types::BootstrapManifestSnapshot,
) -> crate::bootstrap::types::BootstrapManifestDelta {
    crate::bootstrap::manifest::compute_manifest_delta(previous, current)
}
