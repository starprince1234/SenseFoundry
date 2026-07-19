from collections.abc import Iterator
from unittest.mock import patch

import pytest
from fastapi.testclient import TestClient

from inference.app.main import app, model_registry


@pytest.fixture
def client() -> Iterator[TestClient]:
    with patch.object(model_registry, "load_all"):
        model_registry.loaded = True
        model_registry.mlm_model = object()
        model_registry.reranker_model = object()
        with TestClient(app) as test_client:
            yield test_client
        model_registry.loaded = False
        model_registry.mlm_model = None
        model_registry.reranker_model = None


def test_health_endpoint_structure(client: TestClient) -> None:
    response = client.get("/health")

    assert response.status_code == 200
    assert response.json() == {
        "status": "ok",
        "device": "cpu",
        "models_loaded": ["mlm", "reranker"],
    }


def test_models_endpoint_reports_configured_models(client: TestClient) -> None:
    response = client.get("/models")

    assert response.status_code == 200
    body = response.json()
    assert body["device"] == "cpu"
    assert body["mlm"] == {
        "name": "bert-base-chinese",
        "revision": "main",
        "loaded": True,
    }
    assert body["reranker"] == {
        "name": "BAAI/bge-reranker-base",
        "revision": "main",
        "loaded": True,
    }


def test_warmup_endpoint_invokes_registry(client: TestClient) -> None:
    with patch.object(
        model_registry,
        "warmup",
        return_value=["mlm", "reranker"],
    ) as warmup:
        response = client.post("/models/warmup")

    assert response.status_code == 200
    assert response.json()["status"] == "ok"
    assert response.json()["models_warmed"] == ["mlm", "reranker"]
    assert isinstance(response.json()["elapsed_ms"], int)
    warmup.assert_called_once_with()
