//! Unsafe syscall wrappers for check-runner process management.
//! Each function includes a // SAFETY: justification block.
#![allow(unsafe_code)]

/// Send SIGKILL to an entire process group.
/// SAFETY: pgid must be a valid process group ID owned by this session.
pub fn kill_process_group(pgid: libc::pid_t) {
    // SAFETY: caller guarantees pgid is a valid process group.
    // This is the only Unix mechanism to terminate an entire process group.
    let _ = unsafe { libc::killpg(pgid, libc::SIGKILL) };
}

/// Check if a process exists (kill with signal 0).
/// Returns true if the process exists, false otherwise.
/// SAFETY: pid must be a valid process ID.
pub fn process_exists(pid: libc::pid_t) -> bool {
    // SAFETY: signal 0 checks process existence without sending a signal.
    unsafe { libc::kill(pid, 0) == 0 }
}

/// Set an environment variable in the current process.
/// Only used in test context.
/// SAFETY: var and value must be valid C strings.
pub fn set_env(var: &str, value: &str) {
    // SAFETY: pointer lifetime is guaranteed by the caller's borrow.
    let _ = unsafe {
        libc::setenv(
            var.as_ptr() as *const libc::c_char,
            value.as_ptr() as *const libc::c_char,
            1,
        )
    };
}

/// Remove an environment variable from the current process.
/// Only used in test context.
/// SAFETY: var must be a valid C string.
pub fn remove_env(var: &str) {
    // SAFETY: pointer lifetime is guaranteed by the caller's borrow.
    let _ = unsafe { libc::unsetenv(var.as_ptr() as *const libc::c_char) };
}
