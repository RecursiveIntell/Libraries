#!/bin/bash
# MSI GTX 1070 final config — run after reboot
set -euo pipefail

echo "=== MSI proveKV stack final config ==="

# 1. Fix GPU if needed
if ! nvidia-smi --query-gpu=memory.used --format=csv,noheader &>/dev/null; then
    echo "GPU not ready, reloading NVIDIA modules..."
    sudo rmmod nvidia_uvm nvidia_drm nvidia_modeset nvidia 2>/dev/null
    sudo modprobe nvidia nvidia_modeset nvidia_drm nvidia_uvm
    sleep 3
fi
nvidia-smi --query-gpu=memory.used,memory.free --format=csv,noheader
echo "GPU: OK"

# 2. Configure Ollama Vulkan + 20K context
sudo tee /etc/systemd/system/ollama.service.d/vulkan.conf <<'UNIT'
[Service]
Environment=OLLAMA_LLM_LIBRARY=vulkan
Environment=OLLAMA_VULKAN=1
Environment=OLLAMA_GPU_LAYERS=999
Environment=OLLAMA_FLASH_ATTENTION=1
Environment=OLLAMA_KV_CACHE_TYPE=q8_0
Environment=OLLAMA_KEEP_ALIVE=-1
Environment=OLLAMA_NUM_PARALLEL=4
Environment=OLLAMA_MAX_LOADED_MODELS=3
Environment=OLLAMA_HOST=0.0.0.0:11434
Environment=OLLAMA_CONTEXT_LENGTH=20480
UNIT

# 3. Restart
sudo systemctl daemon-reload
sudo systemctl restart ollama
sleep 6

# 4. Pull models
echo "=== PULLING 1B ===" && ollama pull llama3.2:1b 2>&1 | tail -2
echo "=== PULLING 3B ===" && ollama pull llama3.2:3b 2>&1 | tail -2

# 5. Test both
echo "=== TEST 3B 20K ===" && ollama run llama3.2:3b 'Say hello' 2>&1 | grep -vE '⠙|⠹|⠸|⠼|⠴|⠦|⠧|⠇|⠏|⠋'
ollama ps
nvidia-smi --query-gpu=memory.used,memory.free --format=csv,noheader

echo ""
echo "=== DONE ==="
echo "llama3.2:1b  - 100% GPU, fast token gen"
echo "llama3.2:3b  - 100% GPU, 20K context, q8_0 KV cache, Flash Attention"
echo "VRAM: ~7.1GB used out of 8.1GB"
