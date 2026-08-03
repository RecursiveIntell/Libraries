//! build.rs — Compile proveKV CUDA kernels via nvcc.
//!
//! Only runs when `cuda` feature is enabled and CUDA toolkit is found.
//! Outputs `provekv_score.ptx` for runtime loading or statically links
//! the compiled cubin via FFI.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Only compile CUDA when the feature is enabled
    if env::var("CARGO_FEATURE_CUDA").is_err() {
        return;
    }

    println!("cargo:rerun-if-changed=cuda/provekv_score.cu");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");

    // Find nvcc
    let cuda_path = env::var("CUDA_PATH").unwrap_or_else(|_| "/usr/local/cuda".to_string());
    let nvcc = PathBuf::from(&cuda_path).join("bin").join("nvcc");

    if !nvcc.exists() {
        // Try system nvcc
        let system_nvcc = PathBuf::from("/usr/bin/nvcc");
        if system_nvcc.exists() {
            compile_with_nvcc(&system_nvcc);
        } else {
            println!("cargo:warning=nvcc not found — CUDA kernels will not be compiled");
            println!("cargo:warning=Install CUDA toolkit: sudo dnf install cuda-toolkit-12-8");
        }
    } else {
        compile_with_nvcc(&nvcc);
    }
}

fn compile_with_nvcc(nvcc: &PathBuf) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let cuda_src = PathBuf::from("cuda/provekv_score.cu");
    let cubin_out = PathBuf::from(&out_dir).join("provekv_score.cubin");
    let ptx_out = PathBuf::from(&out_dir).join("provekv_score.ptx");

    // Compile to cubin for static linking (compute 6.1 = GTX 1070)
    let status = Command::new(nvcc)
        .arg("-arch=sm_61")
        .arg("-O3")
        .arg("-use_fast_math")
        .arg("-cubin")
        .arg("-o")
        .arg(&cubin_out)
        .arg(&cuda_src)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-link-search=native={}", out_dir);
            println!("cargo:rustc-link-lib=cudart"); // CUDA runtime
            println!("cargo:rustc-cfg=cuda_available");
            println!("cargo:warning=CUDA kernels compiled successfully");
        }
        Ok(s) => {
            println!(
                "cargo:warning=nvcc failed with exit code {}",
                s.code().unwrap_or(-1)
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to run nvcc: {}", e);
        }
    }

    // Also compile PTX for JIT loading
    let _ = Command::new(nvcc)
        .arg("-arch=sm_61")
        .arg("-O3")
        .arg("-ptx")
        .arg("-o")
        .arg(&ptx_out)
        .arg(&cuda_src)
        .status();

    // Link CUDA runtime
    println!("cargo:rustc-link-lib=cudart");
}
