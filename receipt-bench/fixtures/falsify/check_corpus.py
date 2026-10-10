#!/usr/bin/env python3
"""Independent Fraction-based CLI regression harness, not a continuum verifier."""
from __future__ import annotations

import argparse
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def q(value):
    result = Fraction(value)
    require(value == f"{result.numerator}/{result.denominator}", "noncanonical rational")
    return result


def check(problem, result):
    require(set(result) == {"schema", "scope", "kind", "values", "attempts", "reason"}, "result fields")
    require(result["schema"] == "cw.candidate.v1" and result["scope"] == "finite_dimensional", "result scope")
    require(type(result["attempts"]) is int and 0 < result["attempts"] <= 10000, "attempts")
    a = [[q(x) for x in row] for row in problem["a"] + problem["transport"]]
    rhs = list(map(q, problem["rhs"])) + [Fraction(0)] * len(problem["transport"])
    v = list(map(q, result["values"]))
    if result["kind"] == "primal":
        require(result["reason"] == "checked_exactly" and len(v) == len(a[0]), "primal shape")
        require(all(x >= 0 for x in v), "negative coefficient")
        require(all(sum(x*y for x,y in zip(row,v)) == target for row,target in zip(a,rhs)), "primal equalities")
    elif result["kind"] == "dual":
        require(result["reason"] == "checked_exactly" and len(v) == len(a), "dual shape")
        require(sum(x*y for x,y in zip(rhs,v)) > 0, "separating margin")
        require(all(sum(x*y for x,y in zip(col,v)) <= 0 for col in zip(*a)), "dual sign")
    else:
        require(result["kind"] == "unresolved" and not v and result["reason"] in
                ("search_budget_exhausted", "no_certificate_in_bounded_search"), "unresolved semantics")


def wire(problem, budget):
    a, t, rhs = problem["a"], problem["transport"], problem["rhs"]
    lines = [f"CW1 {len(a)} {len(a[0])} {len(t)} {budget}"]
    lines.extend(" ".join(row) for row in a)
    lines.append(" ".join(rhs))
    lines.extend(" ".join(row) for row in t)
    return ("\n".join(lines) + "\n").encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--export-case")
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    corpus_path = Path(__file__).with_name("cases.json")
    raw = corpus_path.read_bytes()
    corpus = json.loads(raw)
    if args.export_case:
        require(args.out is not None, "--out required")
        case = next(c for c in corpus["cases"] if c["id"] == args.export_case)
        args.out.mkdir(mode=0o700, parents=False, exist_ok=False)
        for name in ("statement", "problem", "candidate"):
            (args.out/f"{name}.json").write_text(json.dumps(case[name],indent=2)+"\n")
        print(json.dumps({"exported": case["id"], "scope": corpus["scope"]}))
        return
    results = []
    for case in corpus["cases"]:
        check(case["problem"], case["candidate"])
        if args.binary:
            completed = subprocess.run([str(args.binary.resolve(strict=True))], input=wire(case["problem"],case["budget"]),
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10, check=True)
            require(len(completed.stdout) <= 65536, "excessive output")
            candidate = json.loads(completed.stdout)
            check(case["problem"],candidate)
            require(candidate["kind"] == case["expected_solver_kind"], f"unexpected result for {case['id']}")
            results.append({"id":case["id"],"kind":candidate["kind"],"attempts":candidate["attempts"]})
        else:
            results.append({"id":case["id"],"static_witness_checked":True})
    print(json.dumps({"corpus_sha256":hashlib.sha256(raw).hexdigest(),"scope":corpus["scope"],
                      "native_binary_exercised": bool(args.binary),"results":results},indent=2))


if __name__ == "__main__":
    main()
