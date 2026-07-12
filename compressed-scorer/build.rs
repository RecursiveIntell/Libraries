fn main() {
    cc::Build::new()
        .file("c-kernels/scoring.c")
        .flag_if_supported("-O3")
        .flag_if_supported("-mavx2")
        .flag_if_supported("-mfma")
        .compile("compressed_scorer_kernels");

    println!("cargo:rerun-if-changed=c-kernels/scoring.h");
    println!("cargo:rerun-if-changed=c-kernels/scoring.c");
}