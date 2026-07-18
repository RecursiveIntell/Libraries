#!/usr/bin/env python3
"""Generate the immutable executable learning-coding-agent v2 fixture corpus."""

from __future__ import annotations

import hashlib
import json
import shutil
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "fixtures" / "learning-coding-agent" / "v2"

FAMILIES = [
    ("borrow-check", "development"),
    ("error-propagation", "development"),
    ("iterator-safety", "development"),
    ("module-hygiene", "calibration"),
    ("wire-schema", "holdout"),
]


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def tree_digest(root: Path) -> str:
    material: list[dict[str, str]] = []
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        material.append(
            {
                "path": path.relative_to(root).as_posix(),
                "sha256": sha256_bytes(path.read_bytes()),
            }
        )
    return sha256_bytes(canonical_bytes(material))


def source_pair(family: str, index: int) -> tuple[str, str, str]:
    suffix = f"_{index}"
    if family == "borrow-check":
        baseline = f'''pub fn duplicate{suffix}(input: &str) -> (String, String) {{
    let owned = input.to_string();
    let moved = owned;
    (owned, moved)
}}

#[cfg(test)]
mod tests {{
    use super::*;
    #[test]
    fn duplicates_owned_value() {{
        assert_eq!(duplicate{suffix}("v"), ("v".into(), "v".into()));
    }}
}}
'''
        candidate = baseline.replace("let moved = owned;", "let moved = owned.clone();")
        prompt = "Repair the ownership move so the function returns two independently owned values."
    elif family == "error-propagation":
        baseline = f'''pub fn parse_positive{suffix}(input: &str) -> Result<u32, String> {{
    let value = input.parse::<u32>().unwrap();
    if value > 0 {{ Ok(value) }} else {{ Err("not-positive".into()) }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    #[test]
    fn invalid_input_is_an_error() {{
        assert!(parse_positive{suffix}("invalid").is_err());
    }}
}}
'''
        candidate = baseline.replace(
            'let value = input.parse::<u32>().unwrap();',
            'let value = input.parse::<u32>().map_err(|_| "invalid-integer".to_string())?;',
        )
        prompt = "Replace panic-based parsing with typed error propagation."
    elif family == "iterator-safety":
        baseline = f'''pub fn even_sum{suffix}(values: &[u32]) -> u32 {{
    values.iter().copied().filter(|value| value % 2 == 1).sum()
}}

#[cfg(test)]
mod tests {{
    use super::*;
    #[test]
    fn sums_only_even_values() {{
        assert_eq!(even_sum{suffix}(&[1, 2, 3, 4]), 6);
    }}
}}
'''
        candidate = baseline.replace("value % 2 == 1", "value % 2 == 0")
        prompt = "Correct the iterator predicate without changing the public API."
    elif family == "module-hygiene":
        baseline = f'''mod internal {{
    fn token{suffix}() -> &'static str {{ "ok" }}
}}

pub fn public_token{suffix}() -> &'static str {{
    internal::token{suffix}()
}}

#[cfg(test)]
mod tests {{
    use super::*;
    #[test]
    fn public_wrapper_works() {{ assert_eq!(public_token{suffix}(), "ok"); }}
}}
'''
        candidate = baseline.replace(f"    fn token{suffix}", f"    pub(super) fn token{suffix}")
        prompt = "Repair module visibility using the narrowest sufficient scope."
    elif family == "wire-schema":
        baseline = f'''pub fn encode_user{suffix}(name: &str) -> String {{
    format!(r#"{{{{"userName":"{{}}"}}}}"#, name)
}}

#[cfg(test)]
mod tests {{
    use super::*;
    #[test]
    fn preserves_snake_case_wire_name() {{
        assert_eq!(encode_user{suffix}("Ada"), r#"{{"user_name":"Ada"}}"#);
    }}
}}
'''
        candidate = baseline.replace("userName", "user_name")
        prompt = "Restore the declared snake_case wire field without changing values."
    else:
        raise ValueError(family)
    return baseline, candidate, prompt


def patch_document(task_id: str, baseline: str, candidate: str) -> dict[str, object]:
    return {
        "patch_id": str(uuid.uuid5(uuid.NAMESPACE_URL, f"recursiveintell:{task_id}:v2")),
        "summary": f"canonical evaluator patch for {task_id}",
        "edits": [
            {
                "path": "src/lib.rs",
                "ops": [
                    {
                        "Replace": {
                            "range": {
                                "start": 1,
                                "end_exclusive": len(baseline.splitlines()) + 1,
                            },
                            "lines": candidate.splitlines(),
                        }
                    }
                ],
                "mode": "Modify",
            }
        ],
        "notes": ["evaluator-only oracle; excluded from learner public projection"],
    }


def main() -> None:
    if CORPUS.exists():
        shutil.rmtree(CORPUS)
    (CORPUS / "tasks").mkdir(parents=True)
    (CORPUS / "oracles").mkdir(parents=True)

    tasks: list[dict[str, object]] = []
    for family, split in FAMILIES:
        for index in range(1, 4):
            task_id = f"{family}-{index:02d}"
            fixture_dir = CORPUS / "tasks" / family / task_id / "fixture"
            (fixture_dir / "src").mkdir(parents=True)
            baseline, candidate, prompt = source_pair(family, index)
            cargo = f'''[package]
name = "learning_{family.replace('-', '_')}_{index:02d}"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]
'''
            (fixture_dir / "Cargo.toml").write_text(cargo)
            (fixture_dir / "src" / "lib.rs").write_text(baseline)

            patch = patch_document(task_id, baseline, candidate)
            oracle_path = CORPUS / "oracles" / f"{task_id}.patch.json"
            oracle_path.write_text(json.dumps(patch, indent=2, sort_keys=True) + "\n")
            oracle_digest = sha256_bytes(canonical_bytes(patch))

            public_task = {
                "schema": "AiDENsExecutableLearningTaskV2",
                "task_id": task_id,
                "family": family,
                "split": split,
                "prompt": prompt,
                "fixture_tree_digest": tree_digest(fixture_dir),
                "oracle_digest": oracle_digest,
                "verifier": ["cargo", "test", "--offline", "--quiet"],
                "baseline_expected": "fail",
                "treatment_expected": "pass",
            }
            task_path = CORPUS / "tasks" / family / task_id / "task.json"
            task_path.write_text(json.dumps(public_task, indent=2, sort_keys=True) + "\n")
            tasks.append(
                {
                    "task_id": task_id,
                    "family": family,
                    "split": split,
                    "task_path": task_path.relative_to(CORPUS).as_posix(),
                    "fixture_path": fixture_dir.relative_to(CORPUS).as_posix(),
                    "task_digest": sha256_bytes(canonical_bytes(public_task)),
                    "fixture_tree_digest": public_task["fixture_tree_digest"],
                    "oracle_digest": oracle_digest,
                }
            )

    payload = {
        "schema": "AiDENsExecutableLearningCorpusV2",
        "version": 2,
        "claim_scope": "local-executable-corpus-only",
        "paired_evaluation_status": "not-run-owner-evidence-required",
        "partition_policy": {
            "family_clustered": True,
            "counts": {"development": 9, "calibration": 3, "holdout": 3},
            "oracle_excluded_from_public_task": True,
        },
        "tasks": tasks,
    }
    payload["corpus_digest"] = sha256_bytes(canonical_bytes(payload))
    (CORPUS / "manifest.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    print(f"generated {len(tasks)} executable tasks: {payload['corpus_digest']}")


if __name__ == "__main__":
    main()
