import numpy as np
from fastapi import APIRouter, HTTPException, Request, status
from pydantic import BaseModel

from ..clustering import run_hdbscan, run_hierarchical

router = APIRouter(tags=["clustering"])


class EmbeddingItem(BaseModel):
    id: str
    vector: list[float]


class ClusterRequest(BaseModel):
    embeddings: list[EmbeddingItem]
    method: str = "hdbscan"
    min_cluster_size: int = 5
    min_samples: int = 3
    random_seed: int = 42
    n_clusters: int | None = None


class ClusterMember(BaseModel):
    id: str
    probability: float


class ClusterResult(BaseModel):
    cluster_id: int
    members: list[ClusterMember]
    size: int


class ClusterResponse(BaseModel):
    clusters: list[ClusterResult]
    noise_count: int
    random_seed: int
    model_version: str
    feature_version: str
    device: str


@router.post("/cluster", response_model=ClusterResponse)
async def cluster(request: Request, body: ClusterRequest) -> ClusterResponse:
    settings = request.app.state.settings

    if body.method not in ("hdbscan", "hierarchical"):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=(
                f"Unknown method: {body.method}. Only 'hdbscan' and "
                "'hierarchical' supported."
            ),
        )

    if not body.embeddings:
        return ClusterResponse(
            clusters=[],
            noise_count=0,
            random_seed=body.random_seed,
            model_version="N/A",
            feature_version="0.1.0",
            device=settings.infer_device,
        )

    embeddings = np.asarray(
        [item.vector for item in body.embeddings],
        dtype=float,
    )
    ids = [item.id for item in body.embeddings]

    if body.method == "hdbscan":
        result = run_hdbscan(
            embeddings,
            body.min_cluster_size,
            body.min_samples,
            body.random_seed,
        )
    else:
        n_clusters = body.n_clusters or max(
            2,
            len(body.embeddings) // body.min_cluster_size,
        )
        result = run_hierarchical(
            embeddings,
            n_clusters,
            random_seed=body.random_seed,
        )

    clusters = [
        ClusterResult(
            cluster_id=cluster_result["cluster_id"],
            members=[
                ClusterMember(id=ids[index], probability=probability)
                for index, probability in zip(
                    cluster_result["member_indices"],
                    cluster_result["probabilities"],
                )
            ],
            size=cluster_result["size"],
        )
        for cluster_result in result["clusters"]
    ]

    return ClusterResponse(
        clusters=clusters,
        noise_count=result["noise_count"],
        random_seed=body.random_seed,
        model_version="hdbscan+agglomerative",
        feature_version="0.1.0",
        device=settings.infer_device,
    )
