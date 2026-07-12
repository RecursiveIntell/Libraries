use std::path::PathBuf;

fn main() {
    // ── C kernel: cosine similarity (SIMD via GCC auto-vectorization) ───────
    // Compiled unconditionally — it is a tiny self-contained translation unit
    // with no external C++ deps.  The resulting object is linked into the crate
    // and the Rust FFI wrapper in hubness.rs calls `sm_cosine_similarity`.
    //
    // The usearch cxx bridge (when the `usearch-backend` feature is enabled)
    // is handled by the `usearch` crate's own build script — we do not need
    // cxx-build here.  This build.rs only compiles the C SIMD kernel.
    let kernel_dir = PathBuf::from("c-kernels");
    cc::Build::new()
        .file(kernel_dir.join("similarity.c"))
        .include(&kernel_dir)
        .flag_if_supported("-O3")
        .flag_if_supported("-mavx2")
        .flag_if_supported("-mfma")
        .compile("sm_similarity");

    println!("cargo:rerun-if-changed=c-kernels/similarity.c");
    println!("cargo:rerun-if-changed=c-kernels/similarity.h");
}