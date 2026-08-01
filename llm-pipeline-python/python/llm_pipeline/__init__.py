"""llm-pipeline — Rust LLM pipeline for Hermes.

Provides ``LlmConfig`` and ``Pipeline`` backed by the
RecursiveIntell ``llm-pipeline`` crate via PyO3.
"""

from llm_pipeline._native import LlmConfig, Pipeline

__all__ = ["LlmConfig", "Pipeline"]
