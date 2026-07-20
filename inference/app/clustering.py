from typing import Any, TypedDict

import hdbscan  # pyright: ignore[reportMissingImports]
import numpy as np
from numpy.typing import NDArray
from sklearn.cluster import AgglomerativeClustering


class StabilityRun(TypedDict):
    run: int
    n_clusters: int


class StabilityResult(TypedDict):
    stability_score: float
    cluster_stabilities: list[StabilityRun]
    random_seed: int
    n_resample: int


def run_hdbscan(
    embeddings: NDArray[np.floating[Any]],
    min_cluster_size: int = 5,
    min_samples: int = 3,
    random_seed: int = 42,
) -> dict[str, Any]:
    """Run reproducible HDBSCAN clustering with soft memberships."""
    np.random.seed(random_seed)

    clusterer = hdbscan.HDBSCAN(
        min_cluster_size=min_cluster_size,
        min_samples=min_samples,
        metric="euclidean",
        cluster_selection_method="eom",
        prediction_data=True,
    )
    labels = clusterer.fit_predict(embeddings)

    cluster_ids = sorted(set(labels) - {-1})
    clusters = []
    for cluster_id in cluster_ids:
        members = np.where(labels == cluster_id)[0].tolist()
        clusters.append(
            {
                "cluster_id": int(cluster_id),
                "member_indices": members,
                "probabilities": clusterer.probabilities_[members].tolist(),
                "size": len(members),
            }
        )

    return {
        "labels": labels.tolist(),
        "clusters": clusters,
        "noise_count": int((labels == -1).sum()),
        "n_clusters": len(cluster_ids),
        "random_seed": random_seed,
    }


def run_hierarchical(
    embeddings: NDArray[np.floating[Any]],
    n_clusters: int = 5,
    linkage: str = "ward",
    random_seed: int = 42,
) -> dict[str, Any]:
    """Run hierarchical agglomerative clustering."""
    np.random.seed(random_seed)

    clustering = AgglomerativeClustering(
        n_clusters=n_clusters,
        linkage=linkage,
    )
    labels = clustering.fit_predict(embeddings)

    clusters = []
    for cluster_id in range(n_clusters):
        members = np.where(labels == cluster_id)[0].tolist()
        clusters.append(
            {
                "cluster_id": cluster_id,
                "member_indices": members,
                "probabilities": [1.0] * len(members),
                "size": len(members),
            }
        )

    return {
        "labels": labels.tolist(),
        "clusters": clusters,
        "noise_count": 0,
        "n_clusters": n_clusters,
        "random_seed": random_seed,
    }


def compute_stability(
    embeddings: NDArray[np.floating[Any]],
    n_resample: int = 10,
    subsample_ratio: float = 0.8,
    min_cluster_size: int = 5,
    min_samples: int = 3,
    random_seed: int = 42,
) -> StabilityResult:
    """Estimate HDBSCAN stability from variation across random subsets."""
    np.random.seed(random_seed)
    sample_count = len(embeddings)
    subsample_size = int(sample_count * subsample_ratio)

    label_sets: list[tuple[NDArray[np.integer[Any]], list[int]]] = []
    for run_index in range(n_resample):
        indices = np.random.choice(
            sample_count,
            subsample_size,
            replace=False,
        )
        subsample = embeddings[indices]
        if len(subsample) < min_cluster_size * 2:
            continue

        try:
            result = run_hdbscan(
                subsample,
                min_cluster_size=min_cluster_size,
                min_samples=min_samples,
                random_seed=random_seed + run_index,
            )
        except Exception:
            continue
        label_sets.append((indices, result["labels"]))

    if not label_sets:
        return {
            "stability_score": 0.0,
            "cluster_stabilities": [],
            "random_seed": random_seed,
            "n_resample": 0,
        }

    cluster_counts = [
        len(set(labels) - {-1}) for _, labels in label_sets
    ]
    average_count = np.mean(cluster_counts)
    count_deviation = np.std(cluster_counts)
    stability_score = float(
        max(0.0, 1.0 - count_deviation / max(average_count, 1))
    )

    return {
        "stability_score": stability_score,
        "cluster_stabilities": [
            {"run": run_index, "n_clusters": cluster_count}
            for run_index, cluster_count in enumerate(cluster_counts)
        ],
        "random_seed": random_seed,
        "n_resample": len(label_sets),
    }
