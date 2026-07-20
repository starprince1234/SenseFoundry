from collections.abc import Iterator
from unittest.mock import patch

import numpy as np
import pytest
from fastapi.testclient import TestClient

from inference.app.clustering import run_hdbscan, run_hierarchical
from inference.app.main import app, model_registry


@pytest.fixture
def client() -> Iterator[TestClient]:
    with patch.object(model_registry, "load_all"):
        with TestClient(app) as test_client:
            yield test_client


def test_hdbscan_reproducible_with_same_seed() -> None:
    generator = np.random.default_rng(0)
    embeddings = generator.standard_normal((30, 8))

    first = run_hdbscan(
        embeddings,
        min_cluster_size=3,
        min_samples=2,
        random_seed=42,
    )
    second = run_hdbscan(
        embeddings,
        min_cluster_size=3,
        min_samples=2,
        random_seed=42,
    )

    assert first["labels"] == second["labels"]
    assert first["random_seed"] == second["random_seed"] == 42


def test_hdbscan_accepts_different_seed() -> None:
    generator = np.random.default_rng(1)
    embeddings = generator.standard_normal((20, 4))

    result = run_hdbscan(
        embeddings,
        min_cluster_size=3,
        min_samples=2,
        random_seed=99,
    )

    assert len(result["labels"]) == len(embeddings)
    assert result["random_seed"] == 99


def test_hierarchical_returns_requested_clusters() -> None:
    embeddings = np.array(
        [[0.0, 0.0], [0.1, 0.1], [5.0, 5.0], [5.1, 5.1]]
    )

    result = run_hierarchical(embeddings, n_clusters=2, random_seed=7)

    assert result["n_clusters"] == 2
    assert result["noise_count"] == 0
    assert sorted(cluster["size"] for cluster in result["clusters"]) == [2, 2]


def test_cluster_endpoint_returns_memberships(client: TestClient) -> None:
    response = client.post(
        "/cluster",
        json={
            "embeddings": [
                {"id": "a", "vector": [0.0, 0.0]},
                {"id": "b", "vector": [0.1, 0.1]},
                {"id": "c", "vector": [5.0, 5.0]},
                {"id": "d", "vector": [5.1, 5.1]},
            ],
            "method": "hierarchical",
            "n_clusters": 2,
            "random_seed": 11,
        },
    )

    assert response.status_code == 200
    body = response.json()
    assert body["random_seed"] == 11
    assert body["noise_count"] == 0
    assert {member["id"] for group in body["clusters"] for member in group["members"]} == {
        "a",
        "b",
        "c",
        "d",
    }


def test_cluster_endpoint_rejects_other_algorithms(client: TestClient) -> None:
    response = client.post(
        "/cluster",
        json={"embeddings": [], "method": "kmeans"},
    )

    assert response.status_code == 400
    assert "Only 'hdbscan' and 'hierarchical'" in response.json()["detail"]


def test_empty_embeddings_returns_empty(client: TestClient) -> None:
    response = client.post(
        "/cluster",
        json={"embeddings": [], "random_seed": 42},
    )

    assert response.status_code == 200
    assert response.json()["clusters"] == []
    assert response.json()["noise_count"] == 0
