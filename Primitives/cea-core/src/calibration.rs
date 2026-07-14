const MIN_SAMPLE_EPSILON: f64 = 1e-12;
const CONFIDENCE_SHRINKAGE_Z: f64 = 1.281_551_565_545;
const SAMPLE_GROWTH_SCALE: f64 = 8.0;

pub const MIN_SAMPLES_PER_SIGNATURE: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct ConfidenceEnvelope {
    pub raw_confidence: f64,
    pub coverage_fraction: f64,
    pub sample_units: f64,
    pub signature_count: usize,
}

impl ConfidenceEnvelope {
    pub fn bounded_sample_factor(self, min_samples_per_signature: usize) -> f64 {
        sample_sufficiency_factor(
            self.sample_units,
            self.signature_count,
            min_samples_per_signature,
        )
        .clamp(0.0, 1.0)
    }

    pub fn bounded_confidence(self, min_samples_per_signature: usize) -> f64 {
        advisory_confidence(
            self.raw_confidence,
            self.coverage_fraction,
            self.sample_units,
            self.signature_count,
            min_samples_per_signature,
        )
    }
}

pub fn sample_sufficiency_factor(
    sample_units: f64,
    signature_count: usize,
    min_samples_per_signature: usize,
) -> f64 {
    if signature_count == 0 || min_samples_per_signature == 0 {
        return 0.0;
    }

    let required = (signature_count * min_samples_per_signature) as f64;
    if required <= MIN_SAMPLE_EPSILON {
        return 0.0;
    }

    (sample_units / required).clamp(0.0, 1.0)
}

pub fn advisory_confidence(
    base_confidence: f64,
    coverage_fraction: f64,
    sample_units: f64,
    signature_count: usize,
    min_samples_per_signature: usize,
) -> f64 {
    let bounded_confidence = base_confidence.clamp(0.0, 1.0);
    let bounded_coverage = coverage_fraction.clamp(0.0, 1.0);
    let sufficiency =
        sample_sufficiency_factor(sample_units, signature_count, min_samples_per_signature);
    let effective_samples = effective_sample_size(sample_units);
    let sample_weight = sample_factor(effective_samples);

    (bounded_confidence.min(bounded_coverage) * sample_weight * sufficiency).clamp(0.0, 1.0)
}

pub fn posterior_mean(alpha: f64, beta: f64) -> f64 {
    let total = (alpha + beta).max(MIN_SAMPLE_EPSILON);
    (alpha / total).clamp(0.0, 1.0)
}

pub fn posterior_variance(alpha: f64, beta: f64) -> f64 {
    let total = (alpha + beta).max(MIN_SAMPLE_EPSILON);
    (alpha * beta) / ((total * total) * (total + 1.0)).max(MIN_SAMPLE_EPSILON)
}

pub fn conservative_reliability(alpha: f64, beta: f64) -> f64 {
    let mean = posterior_mean(alpha, beta);
    let std_dev = posterior_variance(alpha, beta).sqrt();
    let lower = mean - (CONFIDENCE_SHRINKAGE_Z * std_dev);
    lower.clamp(0.0, 1.0)
}

/// Return the observed number of independent sample units.
///
/// `EdgeStats` keeps the Beta prior in `alpha` and `beta`; callers of this
/// function pass observations only. Subtracting the two prior pseudo-counts
/// here would therefore erase the first two real observations.
pub fn effective_sample_size(observed_sample_units: f64) -> f64 {
    observed_sample_units.max(0.0)
}

pub fn sample_factor(effective_sample_size: f64) -> f64 {
    (1.0 - (-effective_sample_size / SAMPLE_GROWTH_SCALE).exp()).clamp(0.0, 1.0)
}
