from time import perf_counter
from typing import Any

from fastapi import APIRouter, Request
from starlette.concurrency import run_in_threadpool

router = APIRouter(tags=["models"])


@router.get("")
async def list_models(request: Request) -> dict[str, Any]:
    return request.app.state.models.get_model_info()


@router.post("/warmup")
async def warmup_models(request: Request) -> dict[str, Any]:
    started_at = perf_counter()
    models_warmed = await run_in_threadpool(request.app.state.models.warmup)
    return {
        "status": "ok",
        "models_warmed": models_warmed,
        "elapsed_ms": round((perf_counter() - started_at) * 1000),
    }
