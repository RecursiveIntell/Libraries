#!/usr/bin/env bash
set -euo pipefail
FAIL=0

# Generated receipt stores must never contribute source-policy findings.
GREP_SOURCE=(grep -RIn --exclude-dir=target --exclude-dir=.git --exclude-dir=.codex_evidence)

if "${GREP_SOURCE[@]}" 'pub struct ToolDescriptorV1\|pub enum ToolDescriptorV1\|pub type ToolDescriptorV1' crates/aidens-contracts/src 2>/dev/null; then
  if ! "${GREP_SOURCE[@]}" 'llm_tool_runtime::\|llm-tool-runtime' crates/aidens-* Cargo.toml 2>/dev/null; then
    echo "FAIL: local ToolDescriptorV1 exists but no llm-tool-runtime grounding found."
    FAIL=1
  fi
  if ! "${GREP_SOURCE[@]}" 'fn canonical_descriptor_from_aidens\|CanonicalToolDescriptor' crates/aidens-tool-kit/src crates/aidens-contracts/src 2>/dev/null >/dev/null; then
    echo "FAIL: local ToolDescriptorV1 exists but no canonical llm-tool-runtime descriptor bridge was found."
    FAIL=1
  fi
fi

if "${GREP_SOURCE[@]}" 'pub struct ToolCallRequestV1\|pub struct ToolCallResultV1' crates/aidens-contracts/src 2>/dev/null; then
  if ! "${GREP_SOURCE[@]}" 'ToolReceipt\|ToolDescriptor\|ToolCall' crates/aidens-* 2>/dev/null | grep -v aidens-contracts >/dev/null; then
    echo "FAIL: local tool call DTOs exist without visible canonical tool runtime integration."
    FAIL=1
  fi
  if ! "${GREP_SOURCE[@]}" 'validate_canonical_arguments\|validate_tool_input_with_canonical_runtime' crates/aidens-tool-kit/src 2>/dev/null >/dev/null; then
    echo "FAIL: local tool call DTOs exist but tool input validation is not delegated to llm-tool-runtime."
    FAIL=1
  fi
  if ! "${GREP_SOURCE[@]}" 'canonical-tool-call-owner\|canonical-tool-result-owner\|canonical-tool-receipt-owner' crates/aidens-contracts/src 2>/dev/null >/dev/null; then
    echo "FAIL: local tool call/report DTOs lack explicit llm-tool-runtime backpointer markers."
    FAIL=1
  fi
fi

if [[ "$FAIL" -ne 0 ]]; then
  echo "Tool runtime truth must be grounded in llm-tool-runtime. AiDENs may keep display/report wrappers only."
  exit 1
fi

echo "PASS: tool runtime delegation gate did not find blocking local-only tool truth."
