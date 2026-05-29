use crate::pool::{decode_layer_from_inner, decode_slice_from_inner, DecodedKvSlice, DecodedLayer};
use crate::{PolyKvError, ReaderInjectionReceiptV1};
use quant_codec_core::{KvSliceRequest, LayerId};
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReaderConfig {
    pub reader_label: Option<String>,
    pub scratch_budget_bytes: u64,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            reader_label: None,
            scratch_budget_bytes: 64 * 1024,
        }
    }
}

impl ReaderConfig {
    pub fn scratch_bytes(&self) -> u64 {
        self.scratch_budget_bytes
    }
}

#[derive(Debug)]
pub struct PoolReader {
    pub(crate) inner: Arc<crate::pool::SharedKvPoolInner>,
    config: ReaderConfig,
    receipt: ReaderInjectionReceiptV1,
}

impl PoolReader {
    pub(crate) fn attach(
        inner: Arc<crate::pool::SharedKvPoolInner>,
        config: ReaderConfig,
    ) -> Result<Self, PolyKvError> {
        let reader_id = inner.next_reader_id.fetch_add(1, Ordering::SeqCst);
        let count = inner.active_readers.fetch_add(1, Ordering::SeqCst) + 1;
        inner
            .active_reader_scratch_bytes
            .fetch_add(config.scratch_bytes(), Ordering::SeqCst);
        let receipt = ReaderInjectionReceiptV1 {
            schema_version: 1,
            reader_id,
            manifest_digest: inner.manifest.manifest_digest,
            encoded_shared_bytes: inner.manifest.encoded_bytes,
            per_reader_scratch_bytes: config.scratch_bytes(),
            reader_count_after_attach: count as u64,
        };
        Ok(Self {
            inner,
            config,
            receipt,
        })
    }

    pub fn decode_layer(&self, layer: LayerId) -> Result<DecodedLayer, PolyKvError> {
        decode_layer_from_inner(&self.inner, layer, self.config.scratch_bytes())
    }

    pub fn decode_slice(&self, req: KvSliceRequest) -> Result<DecodedKvSlice, PolyKvError> {
        decode_slice_from_inner(&self.inner, req, self.config.scratch_bytes(), false)
    }

    pub fn decode_slice_exact_fallback(
        &self,
        req: KvSliceRequest,
    ) -> Result<DecodedKvSlice, PolyKvError> {
        decode_slice_from_inner(&self.inner, req, self.config.scratch_bytes(), true)
    }

    pub fn injection_receipt(&self) -> &ReaderInjectionReceiptV1 {
        &self.receipt
    }
}

impl Drop for PoolReader {
    fn drop(&mut self) {
        self.inner.active_readers.fetch_sub(1, Ordering::SeqCst);
        self.inner
            .active_reader_scratch_bytes
            .fetch_sub(self.config.scratch_bytes(), Ordering::SeqCst);
    }
}
