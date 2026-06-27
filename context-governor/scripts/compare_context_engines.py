#!/usr/bin/env python3
"""Compare context engine benchmark reports."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def _pct(value: float) -> str:
    return f"{value * 100:.1f}%"


def _load_report(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    data["_path"] = str(path)
    return data


def _row(report: dict[str, Any]) -> dict[str, Any]:
    aggregate = report.get("aggregate") or {}
    return {
        "engine": report.get("engine") or report.get("name") or "unknown",
        "mode": report.get("mode") or report.get("budget_mode") or "unknown",
        "path": report.get("_path", ""),
        "runs": int(report.get("runs") or report.get("sessions") or 0),
        "failures": int(report.get("failures") or 0),
        "avg_full_tokens": float(aggregate.get("avg_full_tokens") or 0),
        "avg_compacted_tokens": float(aggregate.get("avg_compacted_tokens") or aggregate.get("avg_governed_tokens") or 0),
        "avg_token_reduction": float(aggregate.get("avg_token_reduction") or 0),
        "active_task_visible_rate": float(aggregate.get("active_task_visible_rate") or 0),
        "visible_probe_rate": float(aggregate.get("visible_probe_rate") or 0),
        "recoverable_probe_rate": float(aggregate.get("recoverable_probe_rate") or aggregate.get("required_recoverable_rate") or 0),
        "warnings": int(aggregate.get("warnings") or 0),
    }


def compare_reports(report_paths: list[Path], out_base: Path) -> dict[str, Any]:
    rows = [_row(_load_report(path)) for path in report_paths]
    best_recoverable = max(rows, key=lambda row: row["recoverable_probe_rate"], default=None)
    best_reduction = max(rows, key=lambda row: row["avg_token_reduction"], default=None)
    summary = {
        "schema": "ContextEngineComparisonReportV1",
        "reports": rows,
        "best_by_recoverable_rate": best_recoverable,
        "best_by_token_reduction": best_reduction,
        "claim_boundary": "Offline benchmark comparison. Recoverability/anchor survival is not proof of downstream LLM answer quality.",
    }
    out_base.parent.mkdir(parents=True, exist_ok=True)
    out_base.with_suffix(".json").write_text(json.dumps(summary, indent=2, ensure_ascii=False))
    out_base.with_suffix(".md").write_text(_render_markdown(summary))
    return summary


def _render_markdown(summary: dict[str, Any]) -> str:
    lines = [
        "# Context engine comparison",
        "",
        "Claim boundary: offline benchmark comparison. Recoverability/anchor survival is not proof of downstream LLM answer quality.",
        "",
        "| engine | mode | runs | failures | avg full tokens | avg compacted tokens | token reduction | active task visible | visible probes | recoverable probes | warnings |",
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for row in summary["reports"]:
        lines.append(
            "| {engine} | {mode} | {runs} | {failures} | {full:.1f} | {compacted:.1f} | {reduction} | {active} | {visible} | {recoverable} | {warnings} |".format(
                engine=row["engine"],
                mode=row["mode"],
                runs=row["runs"],
                failures=row["failures"],
                full=row["avg_full_tokens"],
                compacted=row["avg_compacted_tokens"],
                reduction=_pct(row["avg_token_reduction"]),
                active=_pct(row["active_task_visible_rate"]),
                visible=_pct(row["visible_probe_rate"]),
                recoverable=_pct(row["recoverable_probe_rate"]),
                warnings=row["warnings"],
            )
        )
    if summary.get("best_by_recoverable_rate"):
        best = summary["best_by_recoverable_rate"]
        lines.extend([
            "",
            f"Best recoverable-probe rate: `{best['engine']}` / `{best['mode']}` at {_pct(best['recoverable_probe_rate'])}.",
        ])
    if summary.get("best_by_token_reduction"):
        best = summary["best_by_token_reduction"]
        lines.append(f"Best token reduction: `{best['engine']}` / `{best['mode']}` at {_pct(best['avg_token_reduction'])}.")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", action="append", required=True, help="Report JSON path. Repeat for multiple reports.")
    parser.add_argument("--out", required=True, help="Output base path without extension")
    args = parser.parse_args()
    summary = compare_reports([Path(p) for p in args.report], Path(args.out))
    print(f"wrote {Path(args.out).with_suffix('.json')}")
    print(f"wrote {Path(args.out).with_suffix('.md')}")
    if summary.get("best_by_recoverable_rate"):
        best = summary["best_by_recoverable_rate"]
        print(f"best_recoverable={best['engine']}:{best['mode']}:{_pct(best['recoverable_probe_rate'])}")


if __name__ == "__main__":
    main()
