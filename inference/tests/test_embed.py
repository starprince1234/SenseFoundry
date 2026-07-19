import numpy as np
import torch

from ..app.span_pooling import (
    get_context_window_embedding,
    pool_target_span,
)


def test_span_pooling_single_token() -> None:
    embeddings = torch.randn(5, 32)
    char_to_token = [0, 1, 2, 3, 4]

    result = pool_target_span(
        embeddings,
        torch.zeros(5, dtype=torch.long),
        1,
        2,
        char_to_token,
        "mean",
    )

    np.testing.assert_allclose(result, embeddings[1].numpy(), rtol=1e-5)


def test_span_pooling_multi_token() -> None:
    embeddings = torch.randn(6, 32)
    char_to_token = [0, 1, 2, 3, 4, 5]

    single = pool_target_span(
        embeddings,
        torch.zeros(6, dtype=torch.long),
        1,
        2,
        char_to_token,
        "mean",
    )
    multi = pool_target_span(
        embeddings,
        torch.zeros(6, dtype=torch.long),
        1,
        4,
        char_to_token,
        "mean",
    )

    assert not np.allclose(single, multi)


def test_span_result_not_equal_to_sentence() -> None:
    embeddings = torch.randn(10, 64)

    h_target = pool_target_span(
        embeddings,
        torch.zeros(10, dtype=torch.long),
        2,
        4,
        list(range(10)),
        "mean",
    )

    assert not np.allclose(h_target, embeddings[0].numpy())


def test_attention_weighted_span_has_expected_shape() -> None:
    embeddings = torch.randn(10, 64)

    result = pool_target_span(
        embeddings,
        torch.zeros(10, dtype=torch.long),
        2,
        5,
        list(range(10)),
        "attention_weighted",
    )

    assert result.shape == (64,)
    assert not np.allclose(result, embeddings[0].numpy())


def test_context_window_embedding() -> None:
    embeddings = torch.randn(10, 64)

    window = get_context_window_embedding(embeddings, 3, 5, window_size=2)

    assert window.shape == (64,)


def test_fallback_on_empty_span() -> None:
    embeddings = torch.randn(5, 32)

    result = pool_target_span(
        embeddings,
        torch.zeros(5, dtype=torch.long),
        10,
        12,
        [0, 1, 2, 3, 4],
        "mean",
    )

    np.testing.assert_allclose(result, embeddings[0].numpy(), rtol=1e-5)
