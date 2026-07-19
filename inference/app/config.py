import os
import sys
from typing import NoReturn

import torch
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    infer_device: str = "cpu"
    model_cache_dir: str = "/models"
    hf_endpoint: str = "https://huggingface.co"
    mlm_model_name: str = "bert-base-chinese"
    mlm_model_revision: str = "main"
    reranker_model_name: str = "BAAI/bge-reranker-base"
    reranker_model_revision: str = "main"
    app_port: int = 8000

    model_config = SettingsConfigDict(env_file=".env", extra="ignore")


def _fail(message: str) -> NoReturn:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_device(device: str) -> None:
    """Reject unsupported or unavailable devices instead of falling back."""
    if device == "cuda":
        if not torch.cuda.is_available():
            _fail("INFER_DEVICE=cuda but CUDA is not available. Refusing to start.")
    elif device != "cpu":
        _fail(f"INFER_DEVICE must be 'cpu' or 'cuda', got: {device}")


INFER_DEVICE = os.environ.get("INFER_DEVICE", "cpu")
validate_device(INFER_DEVICE)
