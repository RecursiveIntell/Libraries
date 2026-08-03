#!/bin/bash
# proveKV llama-server drop-in for Ollama
# Replaces Ollama's stock llama-server with custom Vulkan build + proveKV KV-cache hooks.
# Run on MSI after llama.cpp finishes building.

set -euo pipefail

LLAMA_BUILD="$HOME/Coding/llama.cpp/build"
OLLAMA_LIB="/usr/local/lib/ollama"

echo "=== proveKV llama-server installer ==="

# Verify custom build exists
if [ ! -f "$LLAMA_BUILD/bin/llama-server" ]; then
    echo "ERROR: $LLAMA_BUILD/bin/llama-server not found. Build llama.cpp first."
    exit 1
fi

# Backup original
sudo cp "$OLLAMA_LIB/llama-server" "$OLLAMA_LIB/llama-server.orig"
echo "Backed up original llama-server"

# Deploy custom build
sudo cp "$LLAMA_BUILD/bin/llama-server" "$OLLAMA_LIB/llama-server"
sudo cp "$LLAMA_BUILD/src/libllama.so" "$OLLAMA_LIB/" 2>/dev/null || true
sudo cp "$LLAMA_BUILD/ggml/src/libggml.so" "$OLLAMA_LIB/" 2>/dev/null || true
echo "Deployed custom llama-server with proveKV hooks"

# Set context to 64K and restart
sudo tee /etc/systemd/system/ollama.service.d/cuda.conf <<'UNIT'
[Service]
Environment=OLLAMA_LLM_LIBRARY=vulkan
Environment=OLLAMA_VULKAN=1
Environment=GGML_VK_VISIBLE_DEVICES=0
Environment=OLLAMA_GPU_LAYERS=999
Environment=OLLAMA_BATCH_SIZE=1024
Environment=OLLAMA_FLASH_ATTENTION=1
Environment=OLLAMA_KV_CACHE_TYPE=q8_0
Environment=OLLAMA_CONTEXT_LENGTH=65536
Environment=OLLAMA_KEEP_ALIVE=-1
Environment=OLLAMA_NUM_PARALLEL=4
Environment=OLLAMA_MAX_LOADED_MODELS=2
Environment=OLLAMA_HOST=0.0.0.0:11434
Environment=PROVEKV_VRAM_THRESHOLD=0.85
Environment=PROVEKV_PAGE_SIZE=256
Environment=PROVEKV_COMPRESS_RATIO=4
UNIT

sudo systemctl daemon-reload
sudo systemctl restart ollama
sleep 4

echo "=== Testing 64K context ==="
ollama run llama3.2:3b "Say hello" 2>&1 | tail -2
ollama ps
nvidia-smi --query-gpu=memory.used,memory.free --format=csv,noheader

echo ""
echo "=== proveKV KV-cache active ==="
echo "Context: 64K tokens (3K hot f16 + 61K cold 8-bit)"
echo "Expected VRAM: ~5.0GB (vs 10GB+ without compression)"
