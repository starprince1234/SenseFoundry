import os
from threading import Lock
from typing import Any

import torch
from transformers import (
    AutoModelForMaskedLM,
    AutoModelForSequenceClassification,
    AutoTokenizer,
)

from .config import Settings


class ModelRegistry:
    def __init__(self, settings: Settings) -> None:
        self.settings = settings
        self.mlm_tokenizer: Any | None = None
        self.mlm_model: Any | None = None
        self.reranker_tokenizer: Any | None = None
        self.reranker_model: Any | None = None
        self.loaded = False
        self._load_lock = Lock()

    def load_all(self) -> None:
        """Load all models, raising on any failure without silent degradation."""
        with self._load_lock:
            if self.loaded:
                return

            os.environ["HF_ENDPOINT"] = self.settings.hf_endpoint
            device = self.settings.infer_device

            self.mlm_tokenizer = AutoTokenizer.from_pretrained(
                self.settings.mlm_model_name,
                cache_dir=self.settings.model_cache_dir,
                revision=self.settings.mlm_model_revision,
            )
            mlm_model = AutoModelForMaskedLM.from_pretrained(
                self.settings.mlm_model_name,
                cache_dir=self.settings.model_cache_dir,
                revision=self.settings.mlm_model_revision,
            ).to(device)
            mlm_model.eval()
            self.mlm_model = mlm_model

            self.reranker_tokenizer = AutoTokenizer.from_pretrained(
                self.settings.reranker_model_name,
                cache_dir=self.settings.model_cache_dir,
                revision=self.settings.reranker_model_revision,
            )
            reranker_model = AutoModelForSequenceClassification.from_pretrained(
                self.settings.reranker_model_name,
                cache_dir=self.settings.model_cache_dir,
                revision=self.settings.reranker_model_revision,
            ).to(device)
            reranker_model.eval()
            self.reranker_model = reranker_model
            self.loaded = True

    def warmup(self) -> list[str]:
        """Run one forward pass through each loaded model."""
        self.load_all()
        assert self.mlm_tokenizer is not None
        assert self.mlm_model is not None
        assert self.reranker_tokenizer is not None
        assert self.reranker_model is not None

        mlm_inputs = self.mlm_tokenizer(
            f"Language is a {self.mlm_tokenizer.mask_token} system.",
            return_tensors="pt",
        ).to(self.settings.infer_device)
        reranker_inputs = self.reranker_tokenizer(
            "This is a usage example.",
            "This is a sense definition.",
            return_tensors="pt",
        ).to(self.settings.infer_device)

        with torch.inference_mode():
            self.mlm_model(**mlm_inputs)
            self.reranker_model(**reranker_inputs)

        return ["mlm", "reranker"]

    def get_model_info(self) -> dict[str, Any]:
        return {
            "mlm": {
                "name": self.settings.mlm_model_name,
                "revision": self.settings.mlm_model_revision,
                "loaded": self.mlm_model is not None,
            },
            "reranker": {
                "name": self.settings.reranker_model_name,
                "revision": self.settings.reranker_model_revision,
                "loaded": self.reranker_model is not None,
            },
            "device": self.settings.infer_device,
        }
