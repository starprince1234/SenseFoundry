import numpy as np
from fastapi import APIRouter, HTTPException, Request, status
from pydantic import BaseModel

from ..clustering import compute_stability
from .cluster import EmbeddingItem

router = APIRouter(tags=["clustering"])


class StabilityRequest(BaseModel):
    embeddings: list[EmbeddingItem]
    method: str = "hdbscan"
    min_cluster_size: int = 5
    min_samples: int = 3
    random_seed: int = 42
    n_resample: int = 10
    subsample_ratio: float = 0.8


class ClusterStability(BaseModel):
    run: int
    n_clusters: int


class StabilityResponse(BaseModel):
    stability_score: float
    cluster_stabilities: list[ClusterStability]
    random_seed: int
    n_resample: int
    model_version: str
    feature_version: str
    device: str


@router.post("/stability", response_model=StabilityResponse)
async def stability(
    request: Request,
    body: StabilityRequest,
) -> StabilityResponse:
    settings = request.app.state.settings

    if body.method != "hdbscan":
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Stability currently supports only 'hdbscan'.",
        )

    if not body.embeddings:
        stability_score = 0.0
        cluster_stabilities: list[ClusterStability] = []
        completed_resamples = 0
    else:
        embeddings = np.asarray(
            [item.vector for item in body.embeddings],
            dtype=float,
        )
        result = compute_stability(
            embeddings,
            n_resample=body.n_resample,
            subsample_ratio=body.subsample_ratio,
            min_cluster_size=body.min_cluster_size,
            min_samples=body.min_samples,
            random_seed=body.random_seed,
        )
        stability_score = result["stability_score"]
        cluster_stabilities = [
            ClusterStability.model_validate(item)
            for item in result["cluster_stabilities"]
        ]
        completed_resamples = result["n_resample"]

    return StabilityResponse(
        stability_score=stability_score,
        cluster_stabilities=cluster_stabilities,
        random_seed=body.random_seed,
        n_resample=completed_resamples,
        model_version="hdbscan-resampling",
        feature_version="0.1.0",
        device=settings.infer_device,
    )
