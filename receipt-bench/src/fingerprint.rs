//! Machine fingerprint generation using SHA-256 of local system characteristics.

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::process::Command;

/// A unique fingerprint of the current machine environment.
///
/// Combines hostname, username, architecture, and OS kernel info
/// to produce a reproducible 64-character hex string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MachineFingerprint(String);

impl MachineFingerprint {
    /// Generate a new machine fingerprint by hashing local system characteristics.
    pub fn generate() -> Self {
        let mut hasher = Sha256::new();

        // Hostname
        let hostname = hostname().unwrap_or_else(|| "unknown".to_string());
        hasher.update(hostname.as_bytes());

        // Username
        let username = username().unwrap_or_else(|| "unknown".to_string());
        hasher.update(username.as_bytes());

        // Architecture
        let arch = env::consts::ARCH;
        hasher.update(arch.as_bytes());

        // OS family
        let os = env::consts::OS;
        hasher.update(os.as_bytes());

        // Number of CPUs (cap at reasonable limit to avoid DoS)
        let cpus = num_cpus().min(256);
        hasher.update([cpus as u8]);

        // Page size logarithm (log2, capped)
        let page_size = page_size_log();
        hasher.update([page_size as u8]);

        // Machine ID if available (avoids hostname collisions)
        if let Some(machine_id) = machine_id() {
            hasher.update(machine_id.as_bytes());
        }

        let result = hasher.finalize();
        let hex = encode_hex(&result);

        // Truncate to 64 chars for readability (256-bit → 64 hex chars)
        MachineFingerprint(hex[..64].to_string())
    }

    /// Create a fingerprint from a pre-existing hex string (for testing).
    #[cfg(test)]
    pub fn from_hex(hex: &str) -> Self {
        MachineFingerprint(hex[..64].to_string())
    }

    /// Returns the raw fingerprint string (64 hex characters).
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MachineFingerprint {
    fn default() -> Self {
        Self::generate()
    }
}

impl std::fmt::Display for MachineFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Internal helpers

fn hostname() -> Option<String> {
    #[cfg(unix)]
    {
        Command::new("hostname")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
    #[cfg(not(unix))]
    {
        None
    }
}

fn username() -> Option<String> {
    env::var("USER").or_else(|_| env::var("USERNAME")).ok()
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}

fn page_size_log() -> usize {
    // Standard page size is 4096, log2(4096) = 12
    // If we can't detect, assume 12
    #[cfg(unix)]
    {
        use std::fs;
        if let Ok(content) = fs::read_to_string("/proc/sys/kernel/osureset_page_size") {
            if let Ok(size) = content.trim().parse::<usize>() {
                let mut log = 0usize;
                let mut s = size;
                while s > 1 {
                    s /= 2;
                    log += 1;
                }
                return log.min(16);
            }
        }
        12
    }
    #[cfg(not(unix))]
    {
        12
    }
}

fn machine_id() -> Option<String> {
    #[cfg(unix)]
    {
        let paths = ["/etc/machine-id", "/var/lib/dbus/machine-id"];
        for path in &paths {
            if let Ok(content) = fs::read_to_string(path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }
    #[cfg(not(unix))]
    {
        None
    }
}

// Pure Rust hex encoder (no dependencies)
fn encode_hex(data: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0xf) as usize] as char);
    }
    s
}
