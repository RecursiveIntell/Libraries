//! Bridge between the MCP server and semantic-memory MemoryStore.
//!
//! semantic-memory's MemoryStore is opened synchronously and search methods
//! are async (tokio). This bridge wraps the store and uses the current
//! tokio runtime handle for async calls (no separate runtime).

use semantic_memory::{MemoryStore, MemoryConfig, EmbeddingConfig, SearchConfig};
use semantic_memory::embedder::{OllamaEmbedder, MockEmbedder, Embedder};
use std::path::PathBuf;
use tokio::runtime::Handle;

pub struct MemoryBridge {
    pub store: MemoryStore,
}

pub struct BridgeConfig {
    /// MCP-005: renamed from db_path to memory_dir — this is a directory,
    /// not a SQLite file path. semantic-memory creates base_dir/memory.db
    /// inside it.
    pub memory_dir: PathBuf,
    pub embedding_url: String,
    pub embedding_model: String,
    pub embedding_dims: usize,
    pub use_mock_embedder: bool,
}

impl BridgeConfig {
    pub fn from_args(
        memory_dir: &str,
        embedding_url: Option<&str>,
        embedding_model: Option<&str>,
        embedding_dims: Option<usize>,
    ) -> Self {
        Self {
            memory_dir: PathBuf::from(memory_dir),
            embedding_url: embedding_url.unwrap_or("http://localhost:11434").to_string(),
            embedding_model: embedding_model.unwrap_or("nomic-embed-text").to_string(),
            embedding_dims: embedding_dims.unwrap_or(768),
            use_mock_embedder: false,
        }
    }
}

impl MemoryBridge {
    /// Open the memory store with the given config.
    /// Store opening is synchronous — no runtime needed here.
    pub fn open(config: BridgeConfig) -> anyhow::Result<Self> {
        let embedding_config = EmbeddingConfig {
            ollama_url: config.embedding_url,
            model: config.embedding_model,
            dimensions: config.embedding_dims,
            batch_size: 32,
            timeout_secs: 60,
        };

        let embedder: Box<dyn Embedder> = if config.use_mock_embedder {
            Box::new(MockEmbedder::new(config.embedding_dims))
        } else {
            Box::new(OllamaEmbedder::try_new(&embedding_config)?)
        };

        let mem_config = MemoryConfig {
            base_dir: config.memory_dir,
            embedding: embedding_config,
            search: SearchConfig::default(),
            ..Default::default()
        };

        let store = MemoryStore::open_with_embedder(mem_config, embedder)?;

        Ok(Self { store })
    }

    /// Get the current tokio runtime handle.
    /// This must be called from within a tokio runtime context.
    pub fn handle() -> Handle {
        Handle::current()
    }
}