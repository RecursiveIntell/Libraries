use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MemoryAccounting {
    pub exact_fallback_bytes: u64,
    pub encoded_shared_bytes: u64,
    pub manifest_bytes: u64,
    pub per_reader_scratch_bytes: u64,
    pub reader_count: u64,
}

impl MemoryAccounting {
    pub fn total_bytes(self) -> u64 {
        self.exact_fallback_bytes
            .saturating_add(self.encoded_shared_bytes)
            .saturating_add(self.manifest_bytes)
            .saturating_add(self.per_reader_scratch_bytes)
    }

    pub fn with_reader_count(mut self, reader_count: u64, scratch_per_reader: u64) -> Self {
        self.reader_count = reader_count;
        self.per_reader_scratch_bytes = reader_count.saturating_mul(scratch_per_reader);
        self
    }

    pub fn with_active_reader_scratch(
        mut self,
        reader_count: u64,
        active_reader_scratch_bytes: u64,
    ) -> Self {
        self.reader_count = reader_count;
        self.per_reader_scratch_bytes = active_reader_scratch_bytes;
        self
    }
}
