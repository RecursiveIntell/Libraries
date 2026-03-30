#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

errors=0

for path in README.md PACK_README.md CONFORMANCE_GATES.md RELEASE_CHECKLIST.md CLAUDE_AUDIT_RECONCILIATION.md MASTER_ISSUE_MATRIX.md; do
  if [[ ! -f "$path" ]]; then
    echo "missing doc: $path" >&2
    errors=1
  fi
done

if [[ -f README.md ]] && ! grep -q 'make gate' README.md; then
  echo "README.md must point to make gate" >&2
  errors=1
fi

if [[ -f PACK_README.md ]] && ! grep -q 'make gate' PACK_README.md; then
  echo "PACK_README.md must point to make gate" >&2
  errors=1
fi

if [[ -f PACK_README.md ]] && ! grep -q 'They are not a competing release authority' PACK_README.md; then
  echo "PACK_README.md must distinguish the numbered hostile pack from the active release authority" >&2
  errors=1
fi

if [[ -f CONFORMANCE_GATES.md ]] && ! grep -q 'cargo run -p contract-schema-gen -- schemas.generated' CONFORMANCE_GATES.md; then
  echo "CONFORMANCE_GATES.md must mention schema regeneration" >&2
  errors=1
fi

if [[ -f CONFORMANCE_GATES.md ]] && ! grep -q 'cargo clippy --workspace --all-targets --all-features -- -D warnings' CONFORMANCE_GATES.md; then
  echo "CONFORMANCE_GATES.md must describe warnings-fatal clippy" >&2
  errors=1
fi

if [[ -f CLAUDE_AUDIT_RECONCILIATION.md ]] && ! grep -Eq 'stale or wrong|Stale / wrong as written' CLAUDE_AUDIT_RECONCILIATION.md; then
  echo "CLAUDE_AUDIT_RECONCILIATION.md must record stale external-audit claims explicitly" >&2
  errors=1
fi

if [[ -f CLAUDE_AUDIT_RECONCILIATION.md ]] && ! grep -q 'do \*\*not\*\* have zero tests anymore' CLAUDE_AUDIT_RECONCILIATION.md; then
  echo "CLAUDE_AUDIT_RECONCILIATION.md must record the corrected zero-tests claims" >&2
  errors=1
fi

if [[ -f MASTER_ISSUE_MATRIX.md ]] && ! grep -q 'The supplied audit correctly found the top CEA bug, but several “zero tests” claims are now stale or wrong.' MASTER_ISSUE_MATRIX.md; then
  echo "MASTER_ISSUE_MATRIX.md must preserve the audit reconciliation note for DOC-001" >&2
  errors=1
fi

if grep -q 'forge-memory-bridge[[:space:]]*=' semantic-memory/Cargo.toml 2>/dev/null; then
  if [[ ! -f CANONICAL_STACK_SPEC_V6.md ]] || ! grep -q 'repo-local compatibility note for the historical `CANONICAL_STACK_SPEC_V6` label' CANONICAL_STACK_SPEC_V6.md; then
    echo "CANONICAL_STACK_SPEC_V6.md must identify itself as a compatibility note, not a full canonical spec" >&2
    errors=1
  fi
  if [[ ! -f CANONICAL_STACK_SPEC_V6.md ]] || ! grep -q 'currently depends directly on forge-memory-bridge for canonical ProjectionImportBatchV3 importer wire types' CANONICAL_STACK_SPEC_V6.md; then
    echo "CANONICAL_STACK_SPEC_V6.md must preserve the current semantic-memory -> forge-memory-bridge importer dependency" >&2
    errors=1
  fi
  if [[ ! -f CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md ]] || ! grep -q 'repo-local compatibility note for the historical `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL` label' CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md; then
    echo "CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md must identify itself as a compatibility note, not a full canonical spec" >&2
    errors=1
  fi
  if [[ ! -f CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md ]] || ! grep -q 'semantic-memory.*currently depends directly on `forge-memory-bridge` for canonical `ProjectionImportBatchV3` importer wire types' CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md; then
    echo "CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md must preserve the current semantic-memory -> forge-memory-bridge importer dependency" >&2
    errors=1
  fi
fi

if grep -q 'constraint-compiler[[:space:]]*=' knowledge-runtime/Cargo.toml 2>/dev/null; then
  if [[ ! -f CANONICAL_STACK_SPEC_V6.md ]] || ! grep -q 'Do not cite it as the full canonical law for the stack' CANONICAL_STACK_SPEC_V6.md; then
    echo "CANONICAL_STACK_SPEC_V6.md must state its scope limit explicitly" >&2
    errors=1
  fi
  if [[ ! -f CANONICAL_STACK_SPEC_V6.md ]] || ! grep -q 'currently depends directly on constraint-compiler, kernel-execution, kernel-oracles, and recursive-kernel-core for bounded advisory inference' CANONICAL_STACK_SPEC_V6.md; then
    echo "CANONICAL_STACK_SPEC_V6.md must preserve the current knowledge-runtime kernel dependency cone" >&2
    errors=1
  fi
  if [[ ! -f CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md ]] || ! grep -q 'Do not present it as the full canonical publication for the stack' CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md; then
    echo "CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md must state its scope limit explicitly" >&2
    errors=1
  fi
  if [[ ! -f CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md ]] || ! grep -q '`knowledge-runtime` currently depends directly on `constraint-compiler`, `kernel-execution`, `kernel-oracles`, and `recursive-kernel-core` for bounded advisory inference' CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md; then
    echo "CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md must preserve the current knowledge-runtime kernel dependency cone" >&2
    errors=1
  fi
fi

if [[ -f RELEASE_CHECKLIST.md ]] && ! grep -q 'bash scripts/check_doc_truth.sh' RELEASE_CHECKLIST.md; then
  echo "RELEASE_CHECKLIST.md must mention doc-truth gate" >&2
  errors=1
fi

if [[ -f RELEASE_CHECKLIST.md ]] && ! grep -q 'make gate' RELEASE_CHECKLIST.md; then
  echo "RELEASE_CHECKLIST.md must include make gate as the front-door command" >&2
  errors=1
fi

if [[ -f RELEASE_CHECKLIST.md ]] && grep -q '/home/' RELEASE_CHECKLIST.md; then
  echo "RELEASE_CHECKLIST.md must not contain local filesystem links" >&2
  errors=1
fi

if (( errors != 0 )); then
  echo "doc truth check failed" >&2
  exit 1
fi

echo "doc truth check passed"
