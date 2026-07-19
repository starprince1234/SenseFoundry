from collections.abc import Sequence
from typing import cast

import numpy as np
from numpy.typing import NDArray
import torch

FloatArray = NDArray[np.float32]


def pool_target_span(
    token_embeddings: torch.Tensor,
    input_ids: torch.Tensor,
    span_start: int,
    span_end: int,
    char_to_token_map: Sequence[int],
    method: str = "mean",
) -> FloatArray:
    """Pool embeddings for the tokens corresponding to a character span."""
    del input_ids  # Kept in the interface for callers that also track token IDs.

    if method not in {"mean", "attention_weighted"}:
        raise ValueError(f"Unsupported pooling method: {method}")

    if (
        span_start < 0
        or span_end <= span_start
        or span_start >= len(char_to_token_map)
        or token_embeddings.shape[0] == 0
    ):
        return cast(FloatArray, token_embeddings[0].detach().cpu().numpy())

    final_char = min(span_end - 1, len(char_to_token_map) - 1)
    token_start = char_to_token_map[span_start]
    token_end = char_to_token_map[final_char] + 1
    sequence_length = token_embeddings.shape[0]

    if token_start < 0 or token_start >= sequence_length:
        return cast(FloatArray, token_embeddings[0].detach().cpu().numpy())

    token_end = min(max(token_end, token_start), sequence_length)
    span_tokens = token_embeddings[token_start:token_end]

    if span_tokens.shape[0] == 0:
        return cast(FloatArray, token_embeddings[0].detach().cpu().numpy())

    if method == "mean" or span_tokens.shape[0] == 1:
        pooled = span_tokens.mean(dim=0)
    else:
        cls_embedding = token_embeddings[0]
        scale = np.sqrt(span_tokens.shape[-1])
        scores = torch.nn.functional.softmax(
            torch.matmul(span_tokens, cls_embedding.unsqueeze(-1)).squeeze(-1)
            / scale,
            dim=0,
        )
        pooled = (span_tokens * scores.unsqueeze(-1)).sum(dim=0)

    return cast(FloatArray, pooled.detach().cpu().numpy())


def get_context_window_embedding(
    token_embeddings: torch.Tensor,
    span_start_token: int,
    span_end_token: int,
    window_size: int = 5,
) -> FloatArray:
    """Mean-pool a bounded token window around the target span."""
    sequence_length = token_embeddings.shape[0]
    window_start = max(0, span_start_token - window_size)
    window_end = min(sequence_length, span_end_token + window_size)
    window = token_embeddings[window_start:window_end]

    if window.shape[0] == 0:
        return cast(FloatArray, token_embeddings[0].detach().cpu().numpy())

    return cast(FloatArray, window.mean(dim=0).detach().cpu().numpy())
