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
