# PATCH_FORMAT.md
# StructuredPatch v1

## Why StructuredPatch instead of raw diffs
- Enforces policy caps BEFORE diff creation (fail early, no wasted apply work)
- Enables strategy-topology novelty scoring (op type distribution, anchor styles)
- Enables CEA: anchor metadata maps directly to `EditOpSignature` construction
- Enables robust apply via semantic anchors instead of fragile line numbers

---

## Schema

```rust
pub struct StructuredPatch {
    pub patch_id: Uuid,
    pub summary:  String,           // short human-readable description
    pub edits:    Vec<FileEdit>,
    pub notes:    Vec<String>,      // optional; not used in validation
}

pub struct FileEdit {
    pub path: PathBuf,              // repo-relative; validated against forbidden globs
    pub ops:  Vec<EditOp>,
    pub mode: Option<FileMode>,     // None = no mode change; Some(mode) = chmod
}

pub enum FileMode {
    Create,     // file does not exist yet; ops are Insert-only
    Delete,     // file will be deleted; ops must be empty
    Modify,     // default; file exists and will be modified
}

pub enum EditOp {
    Insert  { anchor: Anchor, lines: Vec<String> },
    Delete  { range: LineRange },
    Replace { range: LineRange, lines: Vec<String> },
}

pub enum Anchor {
    AfterLine  { line: u32, context_before: Vec<String>, context_after: Vec<String> },
    BeforeLine { line: u32, context_before: Vec<String>, context_after: Vec<String> },
    AfterMatch  { needle: String, occurrence: u32 },
    BeforeMatch { needle: String, occurrence: u32 },
}

pub struct LineRange {
    pub start:        u32,    // 1-indexed, inclusive
    pub end_exclusive: u32,   // exclusive
}
```

---

## Validation rules (pre-apply, fail returns all violations)

Reject patch if:
1. Any `FileEdit.path` matches a forbidden glob (per config; see INVARIANTS.md §I3)
2. Total files changed > `caps.max_files_changed`
3. Total lines changed (added + removed across all ops) > `caps.max_total_lines_changed`
4. Any single file's lines changed > `caps.max_lines_changed_per_file`
5. Patch has no `FileEdit` entries (empty patch) — reject unless `plan_only_mode = true`
6. A `FileEdit` has no ops and `mode != Some(Delete)` — reject (useless edit)
7. `LineRange.end_exclusive <= LineRange.start` — reject (degenerate range)
8. `Anchor::AfterMatch` or `BeforeMatch` with `occurrence == 0` — reject (1-indexed; 0 is invalid)

Validation returns `ValidationResult { ok: bool, violations: Vec<Violation> }` — NOT fail-fast.
All violations collected and returned together.

---

## Apply algorithm (deterministic)

```
apply(patch, workspace_dir):
  for each FileEdit in patch.edits:
    path = workspace_dir / edit.path
    
    if edit.mode == Create:
      ensure path does not exist (error if it does)
      content = []
    elif edit.mode == Delete:
      remove path; continue to next edit
    else:
      content = read_lines(path)
    
    line_mapping = identity mapping (original_line → current_line)
    
    for each op in edit.ops (in order):
      match op:
        Insert { anchor, lines }:
          insert_index = resolve_anchor(anchor, content)
          if insert_index is Err → fail entire patch
          content.splice(insert_index, 0, lines)
          update line_mapping for all lines >= insert_index
        
        Delete { range }:
          mapped_range = apply_line_mapping(line_mapping, range)
          if mapped_range out of bounds → fail
          content.drain(mapped_range)
          update line_mapping
        
        Replace { range, lines }:
          mapped_range = apply_line_mapping(line_mapping, range)
          if mapped_range out of bounds → fail
          content.splice(mapped_range, lines)
          update line_mapping
    
    write_lines(path, content)
    record (edit.path, line_mapping) for CEA attribution
  
  if any op fails → restore workspace from pre-apply snapshot → return Err
  return Ok(LineAttributionMap)
```

### Anchor resolution

```
resolve_anchor(anchor, content) → Result<usize, AnchorError>:
  AfterLine { line, context_before, context_after }:
    idx = line as usize (1-indexed → 0-indexed = line - 1)
    verify: content[idx - context_before.len() .. idx] == context_before (trimmed)
    verify: content[idx .. idx + context_after.len()] == context_after (trimmed)
    if verify fails and AfterMatch is also provided → fall through to match
    return idx  (insert AFTER line → splice at idx)
  
  BeforeLine { line, context_before, context_after }:
    similar; return idx - 1
  
  AfterMatch { needle, occurrence }:
    positions = find_all(content, needle)
    if positions.len() < occurrence → Err(AnchorNotFound)
    if multiple matches exist for occurrence N → Ok (occurrence is 1-indexed, unambiguous)
    return positions[occurrence - 1] + 1
  
  BeforeMatch { needle, occurrence }:
    similar; return positions[occurrence - 1]
```

Ambiguity rule: `occurrence` makes match anchors unambiguous by design.
If `occurrence` exceeds the number of matches → `AnchorError::OccurrenceOutOfRange`.

---

## Diff rendering

```
render_diff(original_dir, patched_dir) → String:
  if `git` on PATH:
    run: git diff --no-index -- original_dir/ patched_dir/
    capture stdout → unified diff string
  else:
    use internal line-diff (Myers algorithm or similar)
    format as standard unified diff (--- a/... +++ b/... @@ ... @@)
  
  return diff string (empty string if no changes)
```

The diff must be reproducible: given the same original and patched content, output is stable.

---

## Strategy-tag extraction (for novelty scoring)

Extract tags from patch topology. Rules are heuristic in v1; can be ML-refined later.

```
extract_strategy_tags(patch) → Vec<StrategyTag>:
  tags = []
  
  n_files = patch.edits.len()
  if n_files == 1 → tags.push("single_file")
  if n_files > 3  → tags.push("multi_file")
  
  total_inserts = count Insert ops
  total_replaces = count Replace ops
  total_deletes = count Delete ops
  
  if total_replaces > total_inserts + total_deletes → tags.push("replace_heavy")
  if total_inserts > total_replaces * 2             → tags.push("insert_heavy")
  if total_deletes > 0 and total_inserts == 0       → tags.push("deletion_only")
  
  for each FileEdit:
    if mode == Create → tags.push("new_file")
    if mode == Delete → tags.push("file_deletion")
    
    for each op:
      context = op's anchor context lines joined
      if context contains "fn "        → tags.push("fn_level_edit")
      if context contains "impl "      → tags.push("impl_level_edit")
      if context contains "trait "     → tags.push("trait_level_edit")
      if context contains "mod "       → tags.push("mod_level_edit")
      if context contains "macro_rules!" → tags.push("macro_level_edit")
      if context contains "async "     → tags.push("async_boundary")
      if context contains "-> Result"  → tags.push("error_type_edit")
  
  if "new_file" in tags and ("fn_level_edit" or "impl_level_edit") in tags:
    tags.push("module_split")
  
  deduplicate tags; return sorted
```

---

## EditOpSignature construction (for CEA)

Constructed during instrumentation (not stored in StructuredPatch itself):

```rust
EditOpSignature {
  op_kind:        op.kind(),
  anchor_kind:    op.anchor_kind(),
  lines_added:    op.lines_added(),
  lines_removed:  op.lines_removed(),
  context_hash:   blake3(context_before.join("\n").trim() + "\n" + context_after.join("\n").trim()),
  file_extension: "rs",
  scope_tag:      infer_scope(context_before + context_after),
  op_index:       position of this op in FileEdit.ops (first/middle/last),
  file_index:     position of this FileEdit in patch.edits (first/middle/last),
}
```
