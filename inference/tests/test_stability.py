from collections.abc import Iterator
from unittest.mock import patch

import numpy as np
import pytest
from fastapi.testclient import TestClient

from inference.app.clustering import compute_stability
from inference.app.main import app, model_registry


@pytest.fixture
def client() -> Iterator[TestClient]:
    with patch.object(model_registry, "load_all"):
        with TestClient(app) as test_client:
            yield test_client


def test_stability_reproducible_with_same_seed() -> None:
    generator = np.random.default_rng(0)
    embeddings = np.vstack(
        (
            generator.normal(0, 0.1, (20, 4)),
            generator.normal(5, 0.1, (20, 4)),
        )
    )

    first = compute_stability(
        embeddings,
        n_resample=5,
        min_cluster_size=3,
        min_samples=2,
        random_seed=42,
    )
    second = compute_stability(
        embeddings,
        n_resample=5,
        min_cluster_size=3,
        min_samples=2,
        random_seed=42,
    )

    assert first == second
    assert 0.0 <= first["stability_score"] <= 1.0
    assert first["n_resample"] == 5


def test_stability_returns_zero_when_subsamples_are_too_small() -> None:
    embeddings = np.ones((4, 2))

    result = compute_stability(
        embeddings,
        n_resample=3,
        subsample_ratio=0.5,
        min_cluster_size=3,
        random_seed=8,
    )

    assert result == {
        "stability_score": 0.0,
        "cluster_stabilities": [],
        "random_seed": 8,
        "n_resample": 0,
    }


def test_stability_endpoint_returns_metadata(client: TestClient) -> None:
    generator = np.random.default_rng(4)
    vectors = np.vstack(
        (
            generator.normal(0, 0.1, (10, 2)),
            generator.normal(4, 0.1, (10, 2)),
        )
    )
    response = client.post(
        "/stability",
        json={
            "embeddings": [
                {"id": str(index), "vector": vector.tolist()}
                for index, vector in enumerate(vectors)
            ],
            "method": "hdbscan",
            "min_cluster_size": 3,
            "min_samples": 2,
            "n_resample": 4,
            "random_seed": 23,
        },
    )

    assert response.status_code == 200
    body = response.json()
    assert body["random_seed"] == 23
    assert body["n_resample"] == 4
    assert 0.0 <= body["stability_score"] <= 1.0
    assert body["model_version"] == "hdbscan-resampling"
    assert body["device"] == "cpu"


def test_stability_endpoint_rejects_unsupported_method(
    client: TestClient,
) -> None:
    response = client.post(
        "/stability",
        json={"embeddings": [], "method": "kmeans"},
    )

    assert response.status_code == 400
