from typing import Any

import torch
from fastapi import APIRouter, HTTPException, Request, status
from pydantic import BaseModel

from ..span_pooling import get_context_window_embedding, pool_target_span

router = APIRouter(tags=["embeddings"])


class SpanInfo(BaseModel):
    start_char: int
    end_char: int
    surface: str
    target_headword: str | None = None


class EmbedRequest(BaseModel):
    text: str
    span: SpanInfo
    include_gloss: str | None = None


class EmbedResponse(BaseModel):
    h_target: list[float]
    h_sentence: list[float]
    h_window: list[float]
    h_gloss: list[float] | None = None
    model_version: str
    feature_version: str
    device: str
    fallback_used: bool = False


@router.post("/embed", response_model=EmbedResponse)
async def embed(request: Request, body: EmbedRequest) -> EmbedResponse:
    models = request.app.state.models
    settings = request.app.state.settings

    if not models.loaded:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Models not loaded",
        )

    try:
        encoding: dict[str, Any] = models.mlm_tokenizer(
            body.text,
            return_tensors="pt",
            return_offsets_mapping=True,
            truncation=True,
            max_length=512,
        )
        input_ids = encoding["input_ids"][0]
        offset_mapping = encoding["offset_mapping"][0].tolist()

        char_to_token: dict[int, int] = {}
        for token_index, (char_start, char_end) in enumerate(offset_mapping):
            for char_index in range(char_start, char_end):
                char_to_token[char_index] = token_index

        model_inputs = {
            key: value.to(settings.infer_device)
            for key, value in encoding.items()
            if key != "offset_mapping"
        }
        with torch.inference_mode():
            outputs = models.mlm_model(
                **model_inputs,
                output_hidden_states=True,
            )
        hidden = outputs.hidden_states[-1][0].cpu()
        h_sentence = hidden[0].numpy().tolist()

        span_start = body.span.start_char
        span_end = body.span.end_char
        fallback_used = not (
            span_start in char_to_token and span_end - 1 in char_to_token
        )

        if fallback_used:
            h_target = h_sentence
            h_window = h_sentence
        else:
            method = "mean" if span_end - span_start == 1 else "attention_weighted"
            character_map = [
                char_to_token.get(char_index, 0)
                for char_index in range(len(body.text))
            ]
            h_target = pool_target_span(
                hidden,
                input_ids,
                span_start,
                span_end,
                character_map,
                method=method,
            ).tolist()
            token_start = char_to_token[span_start]
            token_end = char_to_token[span_end - 1] + 1
            h_window = get_context_window_embedding(
                hidden,
                token_start,
                token_end,
            ).tolist()

        h_gloss = None
        if body.include_gloss:
            gloss_encoding = models.mlm_tokenizer(
                body.include_gloss,
                return_tensors="pt",
                truncation=True,
                max_length=128,
            )
            gloss_inputs = {
                key: value.to(settings.infer_device)
                for key, value in gloss_encoding.items()
            }
            with torch.inference_mode():
                gloss_outputs = models.mlm_model(
                    **gloss_inputs,
                    output_hidden_states=True,
                )
            h_gloss = gloss_outputs.hidden_states[-1][0][0].cpu().numpy().tolist()

        return EmbedResponse(
            h_target=h_target,
            h_sentence=h_sentence,
            h_window=h_window,
            h_gloss=h_gloss,
            model_version=(
                f"{settings.mlm_model_name}@{settings.mlm_model_revision}"
            ),
            feature_version="0.1.0",
            device=settings.infer_device,
            fallback_used=fallback_used,
        )
    except Exception as error:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Embedding failed: {error}",
        ) from error
