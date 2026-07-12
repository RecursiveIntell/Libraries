fn main() {
    // Compile C kernels with -O3 -mavx2 -mfma.
    cc::Build::new()
        .file("c-kernels/codec.c")
        .file("c-kernels/attention.c")
        .include("c-kernels")
        .flag("-O3")
        .flag("-mavx2")
        .flag("-mfma")
        .compile("fib_quant_kernels");

    // Re-run if C source changes.
    println!("cargo:rerun-if-changed=c-kernels/fib_quant.h");
    println!("cargo:rerun-if-changed=c-kernels/codec.c");
    println!("cargo:rerun-if-changed=c-kernels/attention.c");
}
