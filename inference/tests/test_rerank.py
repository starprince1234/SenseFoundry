from collections.abc import Iterator
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

import pytest
import torch
from fastapi.testclient import TestClient

from ..app.main import app, model_registry


@pytest.fixture
def client() -> Iterator[TestClient]:
    tokenizer = MagicMock(
        return_value={
            "input_ids": torch.tensor([[101, 102], [101, 102]]),
            "attention_mask": torch.ones((2, 2), dtype=torch.long),
        }
    )
    model = MagicMock(
        return_value=SimpleNamespace(logits=torch.tensor([[0.0], [2.0]]))
    )

    with patch.object(model_registry, "load_all"):
        model_registry.loaded = True
        model_registry.reranker_tokenizer = tokenizer
        model_registry.reranker_model = model
        with TestClient(app) as test_client:
            yield test_client
        model_registry.loaded = False
        model_registry.reranker_tokenizer = None
        model_registry.reranker_model = None


def test_empty_rerank_returns_empty_scores(client: TestClient) -> None:
    """AC-014: An empty rerank returns an empty list without model work."""
    response = client.post("/rerank", json={"items": []})

    assert response.status_code == 200
    assert response.json() == {
        "scores": [],
        "model_version": "BAAI/bge-reranker-base",
        "feature_version": "0.1.0",
        "device": "cpu",
    }


def test_rerank_returns_score_for_each_pair(client: TestClient) -> None:
    response = client.post(
        "/rerank",
        json={
            "items": [
                {
                    "instance_text": "first instance",
                    "sense_gloss": "first gloss",
                    "reference_sense_id": "sense-1",
                },
                {
                    "instance_text": "second instance",
                    "sense_gloss": "second gloss",
                },
            ]
        },
    )

    assert response.status_code == 200
    scores = response.json()["scores"]
    assert scores[0] == {"reference_sense_id": "sense-1", "score": 0.5}
    assert scores[1]["reference_sense_id"] is None
    assert scores[1]["score"] == pytest.approx(0.880797)
