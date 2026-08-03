fn main() {
    let mut build = cc::Build::new();
    build
        .file("c-kernels/fwht.c")
        .file("c-kernels/qjl.c")
        .file("c-kernels/polar.c")
        .flag_if_supported("-O3");
    enable_target_simd(&mut build);
    build.compile("turbo_quant_kernels");

    println!("cargo:rerun-if-changed=c-kernels/turbo_quant.h");
    println!("cargo:rerun-if-changed=c-kernels/fwht.c");
    println!("cargo:rerun-if-changed=c-kernels/bitpack.c");
    println!("cargo:rerun-if-changed=c-kernels/qjl.c");
    println!("cargo:rerun-if-changed=c-kernels/polar.c");
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
