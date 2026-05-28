#!/usr/bin/env python3
"""P20 executable guardrail scanner for AiDENs.

The scanner is intentionally conservative about claims, not about honest
limitations. It blocks local shadow-truth ownership, fake capability promotion,
and missing required evidence while keeping deferred/scaffold labels visible as
reportable evidence.
"""
from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path

PUBLIC_TYPE_RE = re.compile(r"^\s*pub\s+(struct|enum|type)\s+([A-Za-z0-9_]+)", re.M)

SCAFFOLD_ONLY_CRATES = {
    "aidens-profile-daemon",
    "aidens-profile-desktop",
    "aidens-profile-memory",
    "aidens-profile-research",
}

CANONICAL_SIBLING_CRATES = {
    "assurance-runtime",
    "attestation-exchange",
    "authority-delegation",
    "constraint-compiler",
    "contract-schema-gen",
    "federated-settlement",
    "forge-memory-bridge",
    "forge-pilot",
    "kernel-conformance",
    "kernel-execution",
    "kernel-oracles",
    "knowledge-runtime",
    "llm-tool-runtime",
    "mechanism-runtime",
    "recursive-kernel-core",
    "remote-oracle-admission",
    "semantic-memory",
    "semantic-memory-forge",
    "stack-ids",
    "verification-adjudication",
    "verification-calibration",
    "verification-control",
    "verification-policy",
}

CANONICAL_WORDS = {
    "Adjudication",
    "Bitemporal",
    "Claim",
    "Contradiction",
    "Evidence",
    "Episode",
    "Kernel",
    "Memory",
    "Repair",
    "Residual",
    "Syndrome",
    "Temporal",
    "Truth",
    "Verification",
    "Witness",
}

FORBIDDEN_TYPE_FRAGMENTS = {
    "BitemporalTruth",
    "CanonicalClaim",
    "CanonicalEpisode",
    "CanonicalEvidence",
    "ClaimTruth",
    "ContradictionLaw",
    "EvidenceTruth",
    "KernelWitness",
    "MemoryTruth",
    "RepairLaw",
    "ShadowMemory",
    "TemporalTruth",
    "TruthStore",
    "VerificationTruth",
}

QUARANTINED_PUBLIC_TYPES = {
    "AdmissionDecisionV1",
    "AdmissionDispositionV1",
    "AttestationVerificationStatusV1",
    "JsonRepairReportV2",
    "RemoteOracleReportV1",
    "ResidualV1",
    "SettlementStateV1",
    "SharedDispositionOutcomeV1",
    "StopRuleEvidenceV1",
    "SyndromeKindV1",
    "SyndromeV1",
    "TreatyV1",
    "TrustRootStatusV1",
    "TrustRootV1",
}

ALLOWED_CANONICALISH_PUBLIC_TYPES = {
    "CanonicalBackpointerV1",
    "CompletionAuditStateV1",
    "KernelResidualReportV1",
    "KernelRunDisplayReportV1",
    "KernelStopRuleReportV1",
    "KernelSyndromeKindDisplayV1",
    "KernelSyndromeReportV1",
    "RuntimeCapabilityTruthV1",
    "SandboxCapabilityTruthV1",
}

SAFE_AIDENS_SUFFIXES = (
    "AdapterReceiptV1",
    "CliOutputV1",
    "ConfigV1",
    "DecisionDraftV1",
    "DisplayReportV1",
    "DisplayV1",
    "DraftV1",
    "EntryV1",
    "FindingV1",
    "MatrixV1",
    "ModeV1",
    "OutcomeV1",
    "PlanV1",
    "PolicyV1",
    "ReportDraftV1",
    "ReportV1",
    "ReportV2",
    "RequestV1",
    "ScopeV1",
    "StateV1",
    "StatusV1",
    "VerdictV1",
)

DOC_OVERCLAIM_PATTERNS = [
    ("mostly_complete", re.compile(r"\bmostly complete\b", re.I)),
    ("fully_implemented", re.compile(r"\bfully implemented\b", re.I)),
    ("production_ready", re.compile(r"\bproduction[- ]ready\b|\bready for production\b", re.I)),
    (
        "all_providers_supported",
        re.compile(
            r"\b(?:supports all providers|all providers (?:are )?(?:supported|available|ready|implemented))\b",
            re.I,
        ),
    ),
    (
        "native_tool_calling_supported",
        re.compile(
            r"\b(?:supports native tool calling|native tool (?:calling|loop)s? (?:are )?(?:supported|available|ready|implemented|true|enabled))\b",
            re.I,
        ),
    ),
]

PROVIDER_CLAIM_RE = re.compile(
    r"\b(openai-compatible|openai|openrouter|anthropic|cloud provider(?:s)?|native tool (?:calling|loop)s?)\b",
    re.I,
)
POSITIVE_CAPABILITY_RE = re.compile(
    r"\b(supported|available|ready|healthy|implemented|enabled|executable|native_tool_loop\s*=\s*true|true)\b",
    re.I,
)
NEGATIVE_CAPABILITY_RE = re.compile(
    r"\b(deferred|unavailable|disabled|blocked|false|not|no|without|partial|fixture|not claimed|depends on local service)\b",
    re.I,
)

DEFERRED_REFERENCE_RE = re.compile(
    r"\b(reference(?: interpreter| behavior| semantics)?|temporal|bitemporal)\b.*\b(deferred|deferred=true|deferred:\s*true)\b|"
    r"\b(deferred|deferred=true|deferred:\s*true)\b.*\b(reference(?: interpreter| behavior| semantics)?|temporal|bitemporal)\b",
    re.I,
)
COMPLETE_READY_RE = re.compile(
    r"\b(complete|implemented|supported|healthy|ready|production[- ]ready|done)\b",
    re.I,
)

DEFERRED_OR_SCAFFOLD_MARKER_RE = re.compile(
    r"\b(deferred|TODO|stub|placeholder|scaffold|not implemented|unimplemented!|todo!)\b",
    re.I,
)
COMPATIBILITY_RE = re.compile(
    r"\b(compatibility layer|legacy shim|fallback compatibility|best effort parse|lenient)\b",
    re.I,
)
RUNTIME_COMPATIBILITY_RE = re.compile(
    r"\b(compatibility layer|legacy shim|fallback compatibility|best effort parse|lenient(?: parser| parsing)?)\b",
    re.I,
)

SOURCE_SUFFIXES = {".rs", ".toml", ".json", ".yaml", ".yml"}
TEXT_SUFFIXES = SOURCE_SUFFIXES | {".md", ".txt", ".csv"}
SKIP_DIRS = {".git", ".idea", ".vscode", "node_modules", "target"}
POLICY_DOC_BASENAMES = {
    "ACCEPTANCE_GATES.md",
    "FINAL_STATE_SPEC.md",
    "FORBIDDEN_PATTERNS.md",
    "P20_ACCEPTANCE_GATES.md",
    "P20_DELETION_QUARANTINE_RULES.md",
    "P20_DOCS_TRUTH_REWRITE_GUIDE.md",
    "P20_EXPECTED_FINAL_REPOSITORY_STATE.md",
    "P20_FINISHLINE_SCOPE.md",
    "P20_INVARIANT_CHECKLISTS.md",
    "P20_REFERENCE_INTERPRETER_CLOSEOUT.md",
    "P20_REFERENCE_INTERPRETER_CONFORMANCE_PLAN.md",
    "P20_RISK_REGISTER.md",
    "P20_ROLLBACK_REPAIR_QUARANTINE_PLAN.md",
    "RISK_REGISTER.md",
    "ROLLBACK_AND_QUARANTINE_PLAN.md",
}


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def rel(root: Path, path: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def line_no(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def get_line(text: str, number: int) -> str:
    lines = text.splitlines()
    if 1 <= number <= len(lines):
        return lines[number - 1].strip()
    return ""


def add_finding(
    findings: list[dict],
    *,
    kind: str,
    category: str,
    severity: str,
    file: str,
    line: int | None,
    match: str,
    message: str,
    rule: str,
    context: str = "",
) -> None:
    item = {
        "kind": kind,
        "category": category,
        "severity": severity,
        "file": file,
        "match": match,
        "message": message,
        "rule": rule,
    }
    if line is not None:
        item["line"] = line
    if context:
        item["context"] = context[:240]
    findings.append(item)


def iter_text_files(root: Path):
    for path in root.rglob("*"):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        if path.is_file() and path.suffix.lower() in TEXT_SUFFIXES:
            yield path


def is_test_path(path: str) -> bool:
    return "/tests/" in path or path.endswith("_test.rs") or path.endswith("_tests.rs")


def is_historical_or_protocol_doc(path: str) -> bool:
    return (
        path.startswith("docs/p20/prompts/")
        or path.startswith("docs/p20/reports/")
        or path.startswith("docs/p20/templates/")
        or path.startswith("docs/p20/prior-design-packet/")
        or path.startswith("docs/prior-design-packet/")
        or "/quarantine/" in path
    )


def is_active_doc(path: str) -> bool:
    return path.endswith(".md") and not is_historical_or_protocol_doc(path)


def is_policy_doc(path: str) -> bool:
    return Path(path).name in POLICY_DOC_BASENAMES


def is_negated_or_policy_context(line: str) -> bool:
    lowered = line.lower()
    policy_markers = (
        "must not",
        "do not",
        "not ",
        " not",
        "no ",
        "without",
        "unless",
        "forbidden",
        "out of scope",
        "outside",
        "not claimed",
        "not reached",
        "not complete",
        "is not complete",
        "are not complete",
        "only if",
        "deferred",
        "unavailable",
        "historical",
        "superseded",
        "risk",
        "issue",
        "listed as",
        "fails",
        "failed",
        "should not",
        "unsupported",
    )
    return any(marker in lowered for marker in policy_markers)


def classify_public_type(name: str) -> tuple[str, str, str]:
    if name in QUARANTINED_PUBLIC_TYPES:
        return (
            "blocking",
            "quarantined_public_type",
            "type was previously quarantined as duplicate or ambiguous canonical surface",
        )
    if name in ALLOWED_CANONICALISH_PUBLIC_TYPES:
        return (
            "info",
            "allowed_aidens_report_or_control_dto",
            "explicitly allowlisted by Phase 03 ownership inventory",
        )
    fragment = next((hint for hint in FORBIDDEN_TYPE_FRAGMENTS if hint in name), None)
    if fragment:
        return (
            "blocking",
            "shadow_truth_type_name",
            f"name contains forbidden canonical ownership fragment `{fragment}`",
        )
    if any(word in name for word in CANONICAL_WORDS) and not name.startswith("Aidens"):
        if not name.endswith(SAFE_AIDENS_SUFFIXES):
            return (
                "blocking",
                "ambiguous_canonical_surface",
                "canonical-domain term appears without an AiDENs/report/display/config suffix",
            )
        return (
            "info",
            "canonical_domain_report_or_dto",
            "canonical-domain term is constrained to a report/display/config/DTO suffix",
        )
    if name.startswith("Aidens") or name.startswith("AiDENs") or name.endswith(SAFE_AIDENS_SUFFIXES):
        return ("info", "aidens_orchestration_dto", "AiDENs-owned report/display/config DTO naming")
    return ("info", "local_public_type", "no canonical ownership marker detected")


def scan_public_types(root: Path) -> tuple[list[dict], list[dict]]:
    public_types: list[dict] = []
    findings: list[dict] = []
    target = root / "crates" / "aidens-contracts" / "src" / "lib.rs"
    text = read(target)
    file_name = rel(root, target)
    for match in PUBLIC_TYPE_RE.finditer(text):
        kind = match.group(1)
        name = match.group(2)
        line = line_no(text, match.start())
        severity, classification, reason = classify_public_type(name)
        public_types.append(
            {
                "file": file_name,
                "line": line,
                "declaration_kind": kind,
                "type": name,
                "classification_hint": classification,
                "severity": severity,
                "reason": reason,
            }
        )
        if severity == "blocking":
            add_finding(
                findings,
                kind="shadow_truth_type",
                category="shadow_truth_types",
                severity="blocking",
                file=file_name,
                line=line,
                match=name,
                message=reason,
                rule="public_type_shadow_truth",
                context=get_line(text, line),
            )
    return public_types, findings


def scan_canonical_inventory(
    root: Path, findings: list[dict], aidens_overlay_only: bool
) -> dict:
    canonical_root = root.parent
    present_crates = []
    canonical_type_count = 0
    for crate in sorted(CANONICAL_SIBLING_CRATES):
        crate_dir = canonical_root / crate
        if not crate_dir.exists():
            continue
        present_crates.append(crate)
        for path in crate_dir.rglob("*.rs"):
            if any(part in SKIP_DIRS for part in path.parts):
                continue
            canonical_type_count += len(PUBLIC_TYPE_RE.findall(read(path)))

    aidens_contracts_type_count = 0
    contracts = root / "crates" / "aidens-contracts" / "src" / "lib.rs"
    if contracts.exists():
        aidens_contracts_type_count = len(PUBLIC_TYPE_RE.findall(read(contracts)))

    available = canonical_type_count > 0
    if not available and not aidens_overlay_only:
        add_finding(
            findings,
            kind="canonical_inventory_unavailable",
            category="canonical_inventory",
            severity="blocking",
            file="",
            line=None,
            match="canonical_type_count=0",
            message=(
                "canonical sibling type inventory is empty; ownership scanner cannot certify "
                "absence of duplicate canonical types"
            ),
            rule="canonical_baseline_required",
        )

    return {
        "canonical_inventory_available": available,
        "canonical_type_count": canonical_type_count,
        "aidens_contracts_type_count": aidens_contracts_type_count,
        "present_canonical_crates": present_crates,
        "aidens_overlay_only": aidens_overlay_only,
    }


def scan_docs_overclaims(root: Path, findings: list[dict]) -> None:
    for path in iter_text_files(root):
        r = rel(root, path)
        if not is_active_doc(r):
            continue
        text = read(path)
        for rule, pattern in DOC_OVERCLAIM_PATTERNS:
            for match in pattern.finditer(text):
                line = line_no(text, match.start())
                context = get_line(text, line)
                severity = "info" if is_policy_doc(r) or is_negated_or_policy_context(context) else "blocking"
                add_finding(
                    findings,
                    kind="docs_overclaim",
                    category="docs_overclaiming_support",
                    severity=severity,
                    file=r,
                    line=line,
                    match=match.group(0),
                    message="active documentation uses support/completion language without local proof context"
                    if severity == "blocking"
                    else "claim language appears in a negated, policy, historical, or limitation context",
                    rule=rule,
                    context=context,
                )


def scan_provider_capability_claims(root: Path, findings: list[dict]) -> None:
    for path in iter_text_files(root):
        r = rel(root, path)
        if path.suffix.lower() == ".md" and not is_active_doc(r):
            continue
        if path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        text = read(path)
        for number, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            if stripped.startswith("| Provider |") or stripped.startswith("|---"):
                continue
            if not PROVIDER_CLAIM_RE.search(line):
                continue
            if path.suffix.lower() == ".rs":
                source_positive = re.search(
                    r"\b(?:native_tool_loop(?:_executable)?|native_tool_calling|chat_completion_executable|executable)\s*:\s*true\b|"
                    r"\bProviderBackendStatusV1::Executable\b",
                    line,
                    re.I,
                )
                if not source_positive:
                    continue
            if not POSITIVE_CAPABILITY_RE.search(line):
                continue
            negative = bool(NEGATIVE_CAPABILITY_RE.search(line)) or is_negated_or_policy_context(line)
            severity = "info" if negative else "blocking"
            add_finding(
                findings,
                kind="provider_capability_claim",
                category="provider_capability_overclaims",
                severity=severity,
                file=r,
                line=number,
                match=PROVIDER_CLAIM_RE.search(line).group(0),
                message="provider/native-tool capability is positively claimed without an explicit limitation"
                if severity == "blocking"
                else "provider/native-tool capability mention is explicitly limited",
                rule="provider_capability_truth",
                context=line.strip(),
            )


def scan_deferred_reference_semantics(root: Path, findings: list[dict]) -> None:
    for path in iter_text_files(root):
        r = rel(root, path)
        if path.suffix.lower() == ".md" and not is_active_doc(r):
            continue
        text = read(path)
        for number, line in enumerate(text.splitlines(), start=1):
            if not DEFERRED_REFERENCE_RE.search(line):
                continue
            complete = bool(COMPLETE_READY_RE.search(line))
            policy = is_negated_or_policy_context(line)
            severity = "blocking" if complete and not policy else "info"
            add_finding(
                findings,
                kind="deferred_reference_semantics",
                category="deferred_reference_marked_complete",
                severity=severity,
                file=r,
                line=number,
                match=DEFERRED_REFERENCE_RE.search(line).group(0),
                message="reference or temporal semantics are deferred while the same line marks them complete/supported"
                if severity == "blocking"
                else "deferred reference/temporal semantics are disclosed or discussed as policy",
                rule="deferred_reference_truth",
                context=line.strip(),
            )


def scan_compatibility_language(root: Path, findings: list[dict]) -> None:
    for path in iter_text_files(root):
        r = rel(root, path)
        text = read(path)
        for number, line in enumerate(text.splitlines(), start=1):
            match = COMPATIBILITY_RE.search(line)
            if not match:
                continue
            if path.suffix.lower() == ".rs" and not is_test_path(r):
                runtime_match = RUNTIME_COMPATIBILITY_RE.search(line)
                if runtime_match and not is_negated_or_policy_context(line):
                    add_finding(
                        findings,
                        kind="compatibility_shim_language",
                        category="compatibility_shim_language",
                        severity="blocking",
                        file=r,
                        line=number,
                        match=runtime_match.group(0),
                        message="runtime source contains compatibility/leniency language without an explicit non-widening policy context",
                        rule="runtime_compatibility_shim",
                        context=line.strip(),
                    )
                    continue
            if path.suffix.lower() == ".md" and is_active_doc(r) and not is_negated_or_policy_context(line):
                add_finding(
                    findings,
                    kind="compatibility_shim_language",
                    category="compatibility_shim_language",
                    severity="warning",
                    file=r,
                    line=number,
                    match=match.group(0),
                    message="active docs mention compatibility/leniency; verify it is not a semantic widening claim",
                    rule="docs_compatibility_shim",
                    context=line.strip(),
                )
            else:
                add_finding(
                    findings,
                    kind="compatibility_shim_language",
                    category="compatibility_shim_language",
                    severity="info",
                    file=r,
                    line=number,
                    match=match.group(0),
                    message="compatibility/leniency language appears in a negated, test, policy, or historical context",
                    rule="compatibility_context",
                    context=line.strip(),
                )


def scan_scaffold_promotion(root: Path, findings: list[dict]) -> list[dict]:
    crates: list[dict] = []
    status_text = read(root / "STATUS.md")
    crates_dir = root / "crates"
    if crates_dir.exists():
        for cargo in sorted(crates_dir.glob("*/Cargo.toml")):
            crate = cargo.parent.name
            lib = cargo.parent / "src" / "lib.rs"
            loc = len(read(lib).splitlines()) if lib.exists() else 0
            status_row = re.search(
                rf"^\|\s+`{re.escape(crate)}`\s+\|\s+(implemented|partial|scaffold-only)\s+\|",
                status_text,
                re.M,
            )
            crates.append(
                {
                    "crate": crate,
                    "lib_rs_lines": loc,
                    "status": status_row.group(1) if status_row else None,
                    "expected_scaffold_only": crate in SCAFFOLD_ONLY_CRATES,
                    "scaffold_hint": loc < 100,
                }
            )
            if not status_row:
                add_finding(
                    findings,
                    kind="missing_crate_status",
                    category="scaffold_promotion",
                    severity="blocking",
                    file="STATUS.md",
                    line=None,
                    match=crate,
                    message="crate is absent from STATUS.md crate surface status table",
                    rule="status_lists_all_crates",
                )
            elif crate in SCAFFOLD_ONLY_CRATES and status_row.group(1) != "scaffold-only":
                add_finding(
                    findings,
                    kind="scaffold_crate_promoted",
                    category="scaffold_promotion",
                    severity="blocking",
                    file="STATUS.md",
                    line=None,
                    match=crate,
                    message="known scaffold-only crate is not marked scaffold-only",
                    rule="scaffold_crates_stay_deferred",
                )

    marker_re = re.compile(
        r"Scaffolded for future AiDENs implementation|scaffolded; implement according to AiDENs docs",
        re.I,
    )
    if crates_dir.exists():
        for path in crates_dir.rglob("*.rs"):
            r = rel(root, path)
            crate = r.split("/")[1] if r.startswith("crates/") and len(r.split("/")) > 1 else ""
            for match in marker_re.finditer(read(path)):
                severity = "info" if crate in SCAFFOLD_ONLY_CRATES else "blocking"
                add_finding(
                    findings,
                    kind="scaffold_marker",
                    category="scaffold_promotion",
                    severity=severity,
                    file=r,
                    line=line_no(read(path), match.start()),
                    match=match.group(0),
                    message="scaffold marker is limited to a scaffold-only crate"
                    if severity == "info"
                    else "scaffold marker appears outside the scaffold-only crate list",
                    rule="scaffold_marker_location",
                    context=get_line(read(path), line_no(read(path), match.start())),
                )

    promotion_re = re.compile(
        r"\b(?:scaffold|deferred)\b.{0,80}\b(?:healthy|ready|production|supported|implemented|complete)\b",
        re.I,
    )
    for doc in (root / "README.md", root / "STATUS.md"):
        text = read(doc)
        r = rel(root, doc)
        for match in promotion_re.finditer(text):
            line = line_no(text, match.start())
            context = get_line(text, line)
            negative = is_negated_or_policy_context(context) or "scaffold-only" in context.lower()
            add_finding(
                findings,
                kind="scaffold_or_deferred_promotion",
                category="scaffold_promotion",
                severity="info" if negative else "blocking",
                file=r,
                line=line,
                match=match.group(0),
                message="scaffold/deferred surface appears in a limitation context"
                if negative
                else "scaffold/deferred surface is promoted as ready/supported/complete",
                rule="scaffold_promotion_language",
                context=context,
            )

    return crates


def scan_deferred_markers(root: Path, findings: list[dict]) -> None:
    for path in iter_text_files(root):
        r = rel(root, path)
        if path.suffix.lower() == ".md" and not is_active_doc(r):
            continue
        text = read(path)
        for match in DEFERRED_OR_SCAFFOLD_MARKER_RE.finditer(text):
            line = line_no(text, match.start())
            context = get_line(text, line)
            severity = "warning" if not is_negated_or_policy_context(context) and match.group(0).lower() in {"stub", "placeholder", "not implemented", "unimplemented!", "todo!"} else "info"
            add_finding(
                findings,
                kind="deferred_or_scaffold_marker",
                category="deferred_marker_inventory",
                severity=severity,
                file=r,
                line=line,
                match=match.group(0),
                message="deferred/scaffold marker kept as inventory; blocking only when promoted elsewhere",
                rule="deferred_inventory",
                context=context,
            )


def scan_phase_reports(root: Path, through: int, findings: list[dict]) -> list[dict]:
    phase_reports: list[dict] = []
    reports_dir = root / "docs" / "p20" / "reports"
    for index in range(through + 1):
        path = reports_dir / f"PHASE_{index:02d}_REPORT.md"
        exists = path.exists()
        phase_reports.append({"phase": index, "file": rel(root, path), "exists": exists})
        if not exists:
            add_finding(
                findings,
                kind="missing_phase_report",
                category="missing_phase_reports",
                severity="blocking",
                file=rel(root, path),
                line=None,
                match=f"PHASE_{index:02d}_REPORT.md",
                message="required P20 phase report is missing",
                rule="required_phase_reports",
            )
    return phase_reports


def summarize(
    findings: list[dict],
    public_types: list[dict],
    crates: list[dict],
    phase_reports: list[dict],
    canonical_inventory: dict,
) -> dict:
    categories: dict[str, dict[str, int]] = {}
    severities = {"blocking": 0, "warning": 0, "info": 0}
    for finding in findings:
        severity = finding["severity"]
        severities[severity] = severities.get(severity, 0) + 1
        bucket = categories.setdefault(finding["category"], {"blocking": 0, "warning": 0, "info": 0, "total": 0})
        bucket[severity] = bucket.get(severity, 0) + 1
        bucket["total"] += 1
    return {
        "findings_total": len(findings),
        "blocking_findings": severities.get("blocking", 0),
        "warning_findings": severities.get("warning", 0),
        "info_findings": severities.get("info", 0),
        "public_types": len(public_types),
        "canonical_inventory_available": canonical_inventory["canonical_inventory_available"],
        "canonical_type_count": canonical_inventory["canonical_type_count"],
        "aidens_contracts_type_count": canonical_inventory["aidens_contracts_type_count"],
        "crates": len(crates),
        "scaffold_only_crates": sum(1 for crate in crates if crate["expected_scaffold_only"]),
        "missing_phase_reports": sum(1 for report in phase_reports if not report["exists"]),
        "categories": categories,
    }


def write_reports(out: Path, result: dict) -> None:
    out.mkdir(parents=True, exist_ok=True)
    (out / "p20_scan.json").write_text(json.dumps(result, indent=2), encoding="utf-8")

    summary = result["summary"]
    lines = [
        "# P20 Static Scan Report",
        "",
        f"Generated: {result['generated_at']}",
        "",
        "## Summary",
        "",
        f"- Blocking findings: {summary['blocking_findings']}",
        f"- Warning findings: {summary['warning_findings']}",
        f"- Info findings: {summary['info_findings']}",
        f"- Public types inspected: {summary['public_types']}",
        f"- Canonical inventory available: {summary['canonical_inventory_available']}",
        f"- Canonical sibling public types inspected: {summary['canonical_type_count']}",
        f"- AiDENs contracts public types inspected: {summary['aidens_contracts_type_count']}",
        f"- Crates inspected: {summary['crates']}",
        f"- Required phase reports missing: {summary['missing_phase_reports']}",
        "",
        "## Category Counts",
        "",
    ]
    for category, counts in sorted(summary["categories"].items()):
        lines.append(
            f"- `{category}`: blocking={counts.get('blocking', 0)}, "
            f"warning={counts.get('warning', 0)}, info={counts.get('info', 0)}, total={counts.get('total', 0)}"
        )

    lines.extend(["", "## Blocking Findings", ""])
    blocking = result["blocking_findings"]
    if blocking:
        for finding in blocking:
            location = finding["file"]
            if "line" in finding:
                location = f"{location}:{finding['line']}"
            lines.append(f"- `{finding['kind']}` at `{location}`: {finding['message']} (`{finding['match']}`)")
            if finding.get("context"):
                lines.append(f"  - Context: `{finding['context']}`")
    else:
        lines.append("- None.")

    lines.extend(["", "## Warning Findings", ""])
    warnings = [finding for finding in result["guardrail_findings"] if finding["severity"] == "warning"]
    if warnings:
        for finding in warnings[:200]:
            location = finding["file"]
            if "line" in finding:
                location = f"{location}:{finding['line']}"
            lines.append(f"- `{finding['kind']}` at `{location}`: {finding['message']} (`{finding['match']}`)")
    else:
        lines.append("- None.")

    lines.extend(["", "## Public Type Guardrail", ""])
    public_alerts = [item for item in result["public_types"] if item["severity"] != "info"]
    if public_alerts:
        for item in public_alerts:
            lines.append(
                f"- `{item['type']}` at `{item['file']}:{item['line']}`: "
                f"{item['classification_hint']} - {item['reason']}"
            )
    else:
        lines.append("- No blocking public type ownership findings.")

    lines.extend(["", "## Phase Reports", ""])
    for report in result["phase_reports"]:
        state = "present" if report["exists"] else "missing"
        lines.append(f"- Phase {report['phase']:02d}: `{report['file']}` - {state}")

    lines.extend(["", "## Crate Scaffold Status", ""])
    for crate in result["crates"]:
        expected = "scaffold-only" if crate["expected_scaffold_only"] else "active"
        lines.append(
            f"- `{crate['crate']}`: status={crate['status'] or 'missing'}, "
            f"expected={expected}, lib_rs_lines={crate['lib_rs_lines']}"
        )

    lines.extend(["", "## Evidence Inventory", ""])
    inventory = [finding for finding in result["guardrail_findings"] if finding["severity"] == "info"]
    lines.append(f"- Informational findings retained in JSON: {len(inventory)}")
    lines.append("- See `p20_scan.json` for the full machine-readable finding set.")

    (out / "p20_scan.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def run_scan(root: Path, required_phase_reports_through: int, aidens_overlay_only: bool) -> dict:
    public_types, public_type_findings = scan_public_types(root)
    findings: list[dict] = list(public_type_findings)
    canonical_inventory = scan_canonical_inventory(root, findings, aidens_overlay_only)
    scan_docs_overclaims(root, findings)
    scan_provider_capability_claims(root, findings)
    scan_deferred_reference_semantics(root, findings)
    scan_compatibility_language(root, findings)
    crates = scan_scaffold_promotion(root, findings)
    scan_deferred_markers(root, findings)
    phase_reports = scan_phase_reports(root, required_phase_reports_through, findings)
    blocking = [finding for finding in findings if finding["severity"] == "blocking"]
    result = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "root": str(root),
        "required_phase_reports_through": required_phase_reports_through,
        "public_types": public_types,
        "canonical_inventory": canonical_inventory,
        "guardrail_findings": findings,
        "pattern_findings": findings,
        "blocking_findings": blocking,
        "crates": crates,
        "phase_reports": phase_reports,
    }
    result["summary"] = summarize(findings, public_types, crates, phase_reports, canonical_inventory)
    return result


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--out", default="target/p20-scan")
    parser.add_argument("--require-phase-reports-through", type=int, default=10)
    parser.add_argument("--fail-on-blocking", action="store_true")
    parser.add_argument(
        "--aidens-overlay-only",
        action="store_true",
        help="Allow scanner use without canonical sibling crates; duplicate canonical type certification is disabled.",
    )
    parser.add_argument(
        "--fail-on-high",
        action="store_true",
        help="Backward-compatible alias for --fail-on-blocking.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.require_phase_reports_through < 0 or args.require_phase_reports_through > 10:
        raise SystemExit("--require-phase-reports-through must be between 0 and 10")

    root = Path(args.root).resolve()
    out = Path(args.out)
    result = run_scan(root, args.require_phase_reports_through, args.aidens_overlay_only)
    write_reports(out, result)

    blocking = result["summary"]["blocking_findings"]
    warnings = result["summary"]["warning_findings"]
    print(f"P20 scan complete: {out}/p20_scan.md")
    print(f"Blocking findings: {blocking}")
    print(f"Warning findings: {warnings}")
    if (args.fail_on_blocking or args.fail_on_high) and blocking:
        raise SystemExit(2)


if __name__ == "__main__":
    main()
