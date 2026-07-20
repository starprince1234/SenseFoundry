import torch
from fastapi import APIRouter, HTTPException, Request, status
from pydantic import BaseModel

router = APIRouter(tags=["mlm"])


class SpanInfo(BaseModel):
    start_char: int
    end_char: int
    surface: str


class MlmSubstituteRequest(BaseModel):
    text: str
    span: SpanInfo
    top_k: int = 20


class SubstituteToken(BaseModel):
    token: str
    probability: float


class MlmSubstituteResponse(BaseModel):
    substitutes: list[SubstituteToken]
    model_version: str
    feature_version: str
    device: str


@router.post("/mlm-substitute", response_model=MlmSubstituteResponse)
async def mlm_substitute(
    request: Request,
    body: MlmSubstituteRequest,
) -> MlmSubstituteResponse:
    models = request.app.state.models
    settings = request.app.state.settings

    if not models.loaded:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Models not loaded",
        )

    masked_text = (
        body.text[: body.span.start_char]
        + "[MASK]"
        + body.text[body.span.end_char :]
    )
    encoding = models.mlm_tokenizer(
        masked_text,
        return_tensors="pt",
        truncation=True,
        max_length=512,
    )

    mask_token_id = models.mlm_tokenizer.mask_token_id
    mask_positions = (encoding["input_ids"][0] == mask_token_id).nonzero(
        as_tuple=True
    )[0]
    if len(mask_positions) == 0:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="No MASK token found after encoding",
        )

    model_inputs = {
        key: value.to(settings.infer_device)
        for key, value in encoding.items()
    }
    with torch.inference_mode():
        outputs = models.mlm_model(**model_inputs)

    mask_position = mask_positions[0].item()
    probabilities = torch.softmax(outputs.logits[0, mask_position, :], dim=-1)
    top_k = min(max(body.top_k, 0), probabilities.shape[0])
    top_probabilities, top_ids = torch.topk(probabilities, top_k)

    substitutes = []
    for probability, token_id in zip(
        top_probabilities.tolist(),
        top_ids.tolist(),
    ):
        token = models.mlm_tokenizer.decode([token_id]).strip()
        if token and not token.startswith("##"):
            substitutes.append(
                SubstituteToken(token=token, probability=probability)
            )

    return MlmSubstituteResponse(
        substitutes=substitutes[: body.top_k],
        model_version=settings.mlm_model_name,
        feature_version="0.1.0",
        device=settings.infer_device,
    )
