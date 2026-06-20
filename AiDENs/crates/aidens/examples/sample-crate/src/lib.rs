// Sample crate with intentional unsafe patterns for The Auditor demo.
// These are deliberately bad — DO NOT use as reference.

use std::os::raw::c_int;

/// BUG: unsafe pointer dereference without safety comment or null check
pub fn read_via_ptr(ptr: *const u8) -> u8 {
    unsafe { *ptr }
}

/// BUG: raw pointer arithmetic without bounds checking
pub fn slice_via_raw(ptr: *const u8, offset: usize) -> u8 {
    unsafe { *ptr.add(offset) }
}

/// BUG: transmute between incompatible types
pub fn cast_int(x: c_int) -> u32 {
    unsafe { std::mem::transmute(x) }
}

/// BUG: mutable static without synchronization
static mut COUNTER: u64 = 0;

pub fn increment_counter() {
    unsafe { COUNTER += 1; }
}

pub fn get_counter() -> u64 {
    unsafe { COUNTER }
}

/// SAFE: this function is fine, no issues
pub fn add_numbers(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

/// SAFE: uses safe Rust idioms
pub fn read_file_safe(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}