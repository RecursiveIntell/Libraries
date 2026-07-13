fn main() {
    if std::env::var_os("CARGO_FEATURE_C_KERNELS").is_none() {
        return;
    }

    let mut build = cc::Build::new();
    build.file("c-kernels/scoring.c").flag_if_supported("-O3");
    enable_target_simd(&mut build);
    build.compile("compressed_scorer_kernels");

    println!("cargo:rerun-if-changed=c-kernels/scoring.h");
    println!("cargo:rerun-if-changed=c-kernels/scoring.c");
}

fn enable_target_simd(build: &mut cc::Build) {
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let has = |feature: &str| features.split(',').any(|item| item == feature);
    if has("avx2") {
        build.flag_if_supported("-mavx2");
    }
    if has("fma") {
        build.flag_if_supported("-mfma");
    }
}
