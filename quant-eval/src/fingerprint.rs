//! Machine fingerprint for benchmark receipt provenance.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

/// Machine fingerprint combines machine identity for benchmark receipts.
///
/// This enables reproducible comparisons across different machines
/// by providing a stable identifier for the run environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineFingerprint {
    /// Full fingerprint string (hex encoded)
    fingerprint: String,
}

impl MachineFingerprint {
    /// Create a new fingerprint from the current machine environment.
    pub fn new() -> Self {
        let mut hasher = Hasher::new();

        // hostname
        if let Ok(hostname) = env::var("HOSTNAME").or_else(|_| env::var("HOST")) {
            hasher.update(b"hostname:");
            hasher.update(hostname.as_bytes());
        }

        // username
        if let Ok(user) = env::var("USER").or_else(|_| env::var("USERNAME")) {
            hasher.update(b"user:");
            hasher.update(user.as_bytes());
        }

        // architecture
        hasher.update(b"arch:");
        hasher.update(std::env::consts::ARCH.as_bytes());

        // OS
        hasher.update(b"os:");
        hasher.update(std::env::consts::OS.as_bytes());

        // cpu count
        hasher.update(b"cpus:");
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        hasher.update(&cpus.to_le_bytes());

        // machine id (if available)
        if let Some(machine_id) = Self::machine_id() {
            hasher.update(b"machine_id:");
            hasher.update(machine_id.as_bytes());
        }

        let hash = hasher.finalize();
        let fingerprint = hex::encode(hash.as_bytes());

        Self { fingerprint }
    }

    /// Create a fingerprint from a hex string (for testing).
    pub fn from_hex(hex: &str) -> Self {
        Self {
            fingerprint: hex.to_string(),
        }
    }

    /// Return the fingerprint as a string slice.
    pub fn as_str(&self) -> &str {
        &self.fingerprint
    }

    /// Read machine ID from /etc/machine-id or similar.
    fn machine_id() -> Option<String> {
        let candidates = [
            "/etc/machine-id",
            "/var/lib/dbus/machine-id",
            "/etc/arch-release",
        ];

        for path in candidates {
            if let Ok(contents) = fs::read_to_string(path) {
                let id = contents.trim().to_string();
                if !id.is_empty() {
                    return Some(id);
                }
            }
        }
        None
    }
}

// Simple hex encoder
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(data: impl AsRef<[u8]>) -> String {
        let bytes = data.as_ref();
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0xf) as usize] as char);
        }
        s
    }
}

impl Default for MachineFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MachineFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_creation() {
        let fp = MachineFingerprint::new();
        assert!(!fp.fingerprint.is_empty());
        assert_eq!(fp.fingerprint.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_fingerprint_from_hex() {
        let fp = MachineFingerprint::from_hex("00".repeat(64).as_str());
        assert_eq!(fp.as_str(), "00".repeat(64).as_str());
    }

    #[test]
    fn test_fingerprint_display() {
        let fp = MachineFingerprint::from_hex("ab".repeat(32).as_str());
        let display = format!("{}", fp);
        assert_eq!(display, "ab".repeat(32).as_str());
    }
}