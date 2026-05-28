//! Basic policy evaluation example.

use quant_governor::{
    evaluate, AdmissibilityClass, CodecProfile, ContentType, GovernancePolicy, GovernanceRequest,
};

fn main() {
    println!("=== Quant Governor Basic Policy Example ===\n");

    // Create default policy
    let policy = GovernancePolicy::default();
    println!("Policy: {}", policy.name());

    // Example 1: Critical model content
    let request1 = GovernanceRequest {
        content_type: ContentType::Model,
        size_bytes: 50_000_000,
        accuracy_requirement: 0.99,
        latency_tolerance_ms: 5000,
        admissibility: AdmissibilityClass::Critical,
    };

    let decision1 = evaluate(request1.clone(), &policy).expect("evaluation failed");
    println!("\n1. Critical Model Content:");
    println!("   Content type: {:?}", request1.content_type);
    println!("   Size: {} bytes", request1.size_bytes);
    println!("   Accuracy required: {:.2}", request1.accuracy_requirement);
    println!("   Selected codec: {}", decision1.codec);
    println!("   Degradation budget: {:.4}", decision1.degradation_budget);

    // Example 2: Large image with moderate accuracy
    let request2 = GovernanceRequest {
        content_type: ContentType::Image,
        size_bytes: 10_000_000,
        accuracy_requirement: 0.85,
        latency_tolerance_ms: 2000,
        admissibility: AdmissibilityClass::Standard,
    };

    let decision2 = evaluate(request2.clone(), &policy).expect("evaluation failed");
    println!("\n2. Large Image (Moderate Accuracy):");
    println!("   Content type: {:?}", request2.content_type);
    println!("   Size: {} bytes", request2.size_bytes);
    println!("   Accuracy required: {:.2}", request2.accuracy_requirement);
    println!("   Selected codec: {}", decision2.codec);
    println!("   Degradation budget: {:.4}", decision2.degradation_budget);

    // Example 3: Low latency audio
    let request3 = GovernanceRequest {
        content_type: ContentType::Audio,
        size_bytes: 2_000_000,
        accuracy_requirement: 0.90,
        latency_tolerance_ms: 50, // Very low latency
        admissibility: AdmissibilityClass::HighPriority,
    };

    let decision3 = evaluate(request3.clone(), &policy).expect("evaluation failed");
    println!("\n3. Low Latency Audio:");
    println!("   Content type: {:?}", request3.content_type);
    println!("   Latency tolerance: {}ms", request3.latency_tolerance_ms);
    println!("   Selected codec: {}", decision3.codec);
    println!("   Degradation budget: {:.4}", decision3.degradation_budget);

    // Example 4: Small text bypass
    let request4 = GovernanceRequest {
        content_type: ContentType::Text,
        size_bytes: 100, // Small content
        accuracy_requirement: 0.80,
        latency_tolerance_ms: 1000,
        admissibility: AdmissibilityClass::Standard,
    };

    let decision4 = evaluate(request4.clone(), &policy).expect("evaluation failed");
    println!("\n4. Small Text Content:");
    println!("   Content type: {:?}", request4.content_type);
    println!("   Size: {} bytes (below threshold)", request4.size_bytes);
    println!("   Selected codec: {}", decision4.codec);

    // Show policy presets
    println!("\n=== Policy Presets ===");
    let presets = [
        GovernancePolicy::storage_efficient(),
        GovernancePolicy::low_latency(),
        GovernancePolicy::accuracy_oriented(),
    ];

    for preset in presets {
        println!(
            "  - {}: max_deg={:.2}",
            preset.name(),
            preset.max_degradation()
        );
    }

    println!("\nDone!");
}
