from typing import Any

from fastapi import APIRouter, Request, status
from fastapi.responses import JSONResponse

router = APIRouter(tags=["health"])


@router.get("/health")
async def get_health(request: Request) -> JSONResponse:
    registry = request.app.state.models
    model_info: dict[str, Any] = registry.get_model_info()
    models_loaded = [
        name
        for name in ("mlm", "reranker")
        if model_info[name]["loaded"]
    ]
    ready = registry.loaded
    return JSONResponse(
        status_code=status.HTTP_200_OK if ready else status.HTTP_503_SERVICE_UNAVAILABLE,
        content={
            "status": "ok" if ready else "degraded",
            "device": model_info["device"],
            "models_loaded": models_loaded,
        },
    )
