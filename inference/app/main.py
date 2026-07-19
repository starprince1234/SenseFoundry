from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager

from fastapi import FastAPI

from .config import Settings, validate_device
from .models import ModelRegistry
from .routers import embed, health, models as models_router

settings = Settings()
validate_device(settings.infer_device)
model_registry = ModelRegistry(settings)


@asynccontextmanager
async def lifespan(_app: FastAPI) -> AsyncGenerator[None, None]:
    model_registry.load_all()
    yield


app = FastAPI(
    title="SenseFoundry Inference",
    version="0.1.0",
    lifespan=lifespan,
)
app.state.models = model_registry
app.state.settings = settings

app.include_router(health.router)
app.include_router(embed.router)
app.include_router(models_router.router, prefix="/models")
