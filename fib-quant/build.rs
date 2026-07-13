fn main() {
    let mut build = cc::Build::new();
    build
        .file("c-kernels/codec.c")
        .file("c-kernels/attention.c")
        .include("c-kernels")
        .flag_if_supported("-O3");
    enable_target_simd(&mut build);
    build.compile("fib_quant_kernels");

    // Re-run if C source changes.
    println!("cargo:rerun-if-changed=c-kernels/fib_quant.h");
    println!("cargo:rerun-if-changed=c-kernels/codec.c");
    println!("cargo:rerun-if-changed=c-kernels/attention.c");
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
