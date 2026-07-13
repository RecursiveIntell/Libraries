//! Unsafe syscall wrappers for check-runner process management.
//! Each function includes a // SAFETY: justification block.
#![allow(unsafe_code)]

/// Send SIGKILL to an entire process group.
/// SAFETY: pgid must be a valid process group ID owned by this session.
pub fn kill_process_group(pgid: libc::pid_t) -> std::io::Result<()> {
    // SAFETY: caller guarantees pgid is a valid process group.
    // This is the only Unix mechanism to terminate an entire process group.
    let result = unsafe { libc::killpg(pgid, libc::SIGKILL) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
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
pub fn set_env(var: &str, value: &str) -> std::io::Result<()> {
    let var = std::ffi::CString::new(var).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "environment variable name contains NUL",
        )
    })?;
    let value = std::ffi::CString::new(value).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "environment variable value contains NUL",
        )
    })?;
    // SAFETY: CString guarantees both pointers are valid and NUL-terminated for the call.
    let result = unsafe { libc::setenv(var.as_ptr(), value.as_ptr(), 1) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Remove an environment variable from the current process.
/// Only used in test context.
/// SAFETY: var must be a valid C string.
pub fn remove_env(var: &str) -> std::io::Result<()> {
    let var = std::ffi::CString::new(var).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "environment variable name contains NUL",
        )
    })?;
    // SAFETY: CString guarantees the pointer is valid and NUL-terminated for the call.
    let result = unsafe { libc::unsetenv(var.as_ptr()) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn kill_process_group_surfaces_syscall_failure() {
        let error = super::kill_process_group(i32::MAX).unwrap_err();
        assert_ne!(error.raw_os_error(), Some(0));
    }

    #[test]
    fn environment_wrappers_reject_interior_nul() {
        assert!(super::set_env("BAD\0NAME", "value").is_err());
        assert!(super::set_env("NAME", "bad\0value").is_err());
        assert!(super::remove_env("BAD\0NAME").is_err());
    }
}
