#!/usr/bin/env python3
"""Tests for adaptive_budget.py"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from adaptive_budget import (
    allocate_budgets,
    allocate_head_budgets,
    compute_expected_mean_k,
    validate_budgets,
    fragility_from_receipt,
)


def test_allocate_budgets_stable_layer_gets_reduced():
    fragility = {0: 0.92, 7: 0.999, 23: 0.9999}
    budgets = allocate_budgets(fragility, ref_k=64, target_cosine=0.995)
    assert budgets[0] > budgets[7], f"fragile layer 0 ({budgets[0]}) should get more than layer 7 ({budgets[7]})"
    assert all(32 + 16 <= v <= 256 + 16 for v in budgets.values()), f"budgets out of range: {budgets}"


def test_allocate_budgets_all_above_target():
    fragility = {0: 0.998, 1: 0.999}
    budgets = allocate_budgets(fragility, ref_k=128, target_cosine=0.995)
    assert all(v <= 128 + 16 for v in budgets.values()), f"all should be <= ref_k + guard: {budgets}"


def test_fragile_layer_gets_more_than_stable():
    fragility = {0: 0.92, 10: 0.999, 20: 1.0}
    budgets = allocate_budgets(fragility, ref_k=64, target_cosine=0.995)
    assert budgets[0] > budgets[10], f"layer 0 should get more than layer 10"
    assert budgets[10] >= budgets[20], f"layer 10 should get at least as much as layer 20"


def test_compute_expected_mean_k():
    budgets = {0: 80, 1: 48, 2: 48}
    mean_k = compute_expected_mean_k(budgets, 256)
    assert 55 < mean_k < 60, f"mean_k={mean_k}"


def test_validate_budgets():
    budgets = {0: 48, 1: 128}
    assert validate_budgets(budgets, 16, 256)
    budgets = {0: 500}
    assert not validate_budgets(budgets, 16, 256)


def test_allocate_head_budgets():
    head_fragility = {(0, 0): 0.92, (0, 1): 0.998, (1, 0): 0.999, (1, 1): 0.9999}
    budgets = allocate_head_budgets(head_fragility, ref_k=64, target_cosine=0.995)
    assert budgets[(0, 0)] > budgets[(0, 1)], "fragile head should get more"
    assert budgets[(0, 0)] > budgets[(1, 0)], "layer 0 head 0 should get more than layer 1 head 0"
    assert all(16 + 16 <= v <= 256 + 16 for v in budgets.values())


def test_fragility_from_receipt():
    receipt = {
        "attention": {
            "per_layer": {
                "0": {"cosine_p05": 0.92, "samples": 100},
                "1": {"cosine_p05": 0.999, "samples": 100},
            }
        }
    }
    frag = fragility_from_receipt(receipt)
    assert frag == {0: 0.92, 1: 0.999}


if __name__ == "__main__":
    import pytest
    sys.exit(pytest.main([__file__, "-v"]))