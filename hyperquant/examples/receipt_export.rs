use hyperquant::{estimate_best_rice_profile, quantize_a2, quantize_z1};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ExportedReceipt<'a> {
    profile: &'a str,
    receipt: hyperquant::HyperQuantReceiptV1,
    rice_k: u8,
    rice_bits_per_scalar: f32,
    rice_encoded_bytes: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = [0.125, -0.5, 1.25, 2.0, 0.5, 0.866_025_4];
    let z1 = quantize_z1(&input, 8.0)?;
    let a2 = quantize_a2(&input, 8.0)?;

    let z1_profile = estimate_best_rice_profile(&z1.codes, 0..=8)?;
    let a2_profile = estimate_best_rice_profile(&a2.codes, 0..=8)?;

    let exports = [
        ExportedReceipt {
            profile: "z1",
            receipt: z1.receipt(),
            rice_k: z1_profile.k,
            rice_bits_per_scalar: z1_profile.bits_per_scalar,
            rice_encoded_bytes: z1_profile.encoded_bytes,
        },
        ExportedReceipt {
            profile: "a2",
            receipt: a2.receipt(),
            rice_k: a2_profile.k,
            rice_bits_per_scalar: a2_profile.bits_per_scalar,
            rice_encoded_bytes: a2_profile.encoded_bytes,
        },
    ];

    println!("{}", serde_json::to_string_pretty(&exports)?);
    Ok(())
}
