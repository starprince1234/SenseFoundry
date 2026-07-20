from collections.abc import Iterator
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

import pytest
import torch
from fastapi.testclient import TestClient

from ..app.main import app, model_registry


@pytest.fixture
def client() -> Iterator[TestClient]:
    tokenizer = MagicMock()
    tokenizer.mask_token_id = 103
    tokenizer.return_value = {
        "input_ids": torch.tensor([[101, 103, 102]]),
        "attention_mask": torch.ones((1, 3), dtype=torch.long),
    }
    tokenizer.decode.side_effect = lambda token_ids: {
        7: "replacement",
        8: "alternative",
    }[token_ids[0]]

    logits = torch.full((1, 3, 10), -10.0)
    logits[0, 1, 7] = 4.0
    logits[0, 1, 8] = 3.0
    model = MagicMock(return_value=SimpleNamespace(logits=logits))

    with patch.object(model_registry, "load_all"):
        model_registry.loaded = True
        model_registry.mlm_tokenizer = tokenizer
        model_registry.mlm_model = model
        with TestClient(app) as test_client:
            yield test_client
        model_registry.loaded = False
        model_registry.mlm_tokenizer = None
        model_registry.mlm_model = None


def test_mlm_empty_text_is_handled(client: TestClient) -> None:
    response = client.post(
        "/mlm-substitute",
        json={
            "text": "",
            "span": {"start_char": 0, "end_char": 0, "surface": ""},
            "top_k": 2,
        },
    )

    assert response.status_code == 200
    assert len(response.json()["substitutes"]) == 2


def test_mlm_response_structure(client: TestClient) -> None:
    response = client.post(
        "/mlm-substitute",
        json={
            "text": "target word",
            "span": {"start_char": 0, "end_char": 6, "surface": "target"},
            "top_k": 2,
        },
    )

    assert response.status_code == 200
    body = response.json()
    assert body["substitutes"][0]["token"] == "replacement"
    assert isinstance(body["substitutes"][0]["probability"], float)
    assert body["model_version"] == "bert-base-chinese"
    assert body["feature_version"] == "0.1.0"
    assert body["device"] == "cpu"
