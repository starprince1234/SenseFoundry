import os
import subprocess
import sys

import pytest

from inference.app.config import Settings, validate_device


def test_settings_read_environment(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("MODEL_CACHE_DIR", "/tmp/model-cache")
    monkeypatch.setenv("HF_ENDPOINT", "https://hf.example.test")

    settings = Settings()

    assert settings.model_cache_dir == "/tmp/model-cache"
    assert settings.hf_endpoint == "https://hf.example.test"


def test_cpu_device_is_accepted() -> None:
    validate_device("cpu")


def test_cuda_fail_fast_when_not_available() -> None:
    """INFER_DEVICE=cuda exits when CUDA is unavailable, never falling back."""
    environment = os.environ.copy()
    environment["INFER_DEVICE"] = "cuda"
    result = subprocess.run(
        [
            sys.executable,
            "-c",
            "import torch; torch.cuda.is_available = lambda: False; "
            "import inference.app.config",
        ],
        capture_output=True,
        check=False,
        env=environment,
    )

    assert result.returncode == 1
    assert (
        "INFER_DEVICE=cuda but CUDA is not available. Refusing to start."
        in result.stderr.decode()
    )


def test_invalid_device_fails_fast() -> None:
    environment = os.environ.copy()
    environment["INFER_DEVICE"] = "mps"
    result = subprocess.run(
        [sys.executable, "-c", "import inference.app.config"],
        capture_output=True,
        check=False,
        env=environment,
    )

    assert result.returncode == 1
    assert "must be 'cpu' or 'cuda'" in result.stderr.decode()
