use quant_codec_core::EvalReport;

pub fn mse(exact: &[f32], decoded: &[f32]) -> Option<f64> {
    if exact.len() != decoded.len() || exact.is_empty() {
        return None;
    }
    let sum = exact
        .iter()
        .zip(decoded)
        .map(|(a, b)| {
            let delta = f64::from(*a) - f64::from(*b);
            delta * delta
        })
        .sum::<f64>();
    Some(sum / exact.len() as f64)
}

pub fn max_abs_error(exact: &[f32], decoded: &[f32]) -> Option<f64> {
    if exact.len() != decoded.len() || exact.is_empty() {
        return None;
    }
    exact
        .iter()
        .zip(decoded)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs())
        .reduce(f64::max)
}

pub fn cosine_similarity(exact: &[f32], decoded: &[f32]) -> Option<f64> {
    if exact.len() != decoded.len() || exact.is_empty() {
        return None;
    }
    let mut dot = 0.0;
    let mut a_norm = 0.0;
    let mut b_norm = 0.0;
    for (a, b) in exact.iter().zip(decoded) {
        let a = f64::from(*a);
        let b = f64::from(*b);
        dot += a * b;
        a_norm += a * a;
        b_norm += b * b;
    }
    if a_norm == 0.0 && b_norm == 0.0 {
        return Some(1.0);
    }
    if a_norm == 0.0 || b_norm == 0.0 {
        return Some(0.0);
    }
    Some(dot / (a_norm.sqrt() * b_norm.sqrt()))
}

pub fn eval_report(
    exact: &[f32],
    decoded: &[f32],
    bytes_exact: u64,
    bytes_encoded: u64,
    max_mse: f64,
    note: impl Into<String>,
) -> EvalReport {
    let mse = mse(exact, decoded);
    let cosine_similarity = cosine_similarity(exact, decoded);
    let max_abs_error = max_abs_error(exact, decoded);
    let finite = decoded.iter().all(|v| v.is_finite());
    let passed = finite && mse.map(|value| value <= max_mse).unwrap_or(false);
    EvalReport {
        mse,
        cosine_similarity,
        max_abs_error,
        bytes_exact,
        bytes_encoded,
        passed,
        notes: vec![note.into()],
    }
}
