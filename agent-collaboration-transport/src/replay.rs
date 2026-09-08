use std::collections::HashSet;
use std::sync::Mutex;

pub const DEFAULT_MAX_NONCES: usize = 4096;

pub struct ReplayCache {
    nonces: Mutex<HashSet<String>>,
    max_entries: usize,
}

impl ReplayCache {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_NONCES)
    }

    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            nonces: Mutex::new(HashSet::new()),
            max_entries,
        }
    }

    /// Atomically admit a nonce once; `false` means it was already observed
    /// or the bounded replay cache is full.
    pub fn admit(&self, nonce: &str) -> bool {
        let Ok(mut nonces) = self.nonces.lock() else {
            return false;
        };
        if nonces.contains(nonce) || nonces.len() >= self.max_entries {
            return false;
        }
        nonces.insert(nonce.to_owned())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.nonces.lock().map(|nonces| nonces.len()).unwrap_or(0)
    }
}

impl Default for ReplayCache {
    fn default() -> Self {
        Self::new()
    }
}
