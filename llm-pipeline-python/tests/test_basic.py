"""Basic import and API smoke tests (run after maturin builds the extension)."""


def test_public_exports():
    from llm_pipeline import Client, CostModel, LlmResult, ReceiptSigner, ReceiptVerifier, verify_receipt

    assert all((Client, CostModel, LlmResult, ReceiptSigner, ReceiptVerifier, verify_receipt))
