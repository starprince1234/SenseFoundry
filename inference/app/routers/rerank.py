import torch
from fastapi import APIRouter, HTTPException, Request, status
from pydantic import BaseModel

router = APIRouter(tags=["reranking"])


class RerankItem(BaseModel):
    instance_text: str
    sense_gloss: str
    reference_sense_id: str | None = None


class RerankRequest(BaseModel):
    items: list[RerankItem]


class RerankScore(BaseModel):
    reference_sense_id: str | None
    score: float


class RerankResponse(BaseModel):
    scores: list[RerankScore]
    model_version: str
    feature_version: str
    device: str


@router.post("/rerank", response_model=RerankResponse)
async def rerank(request: Request, body: RerankRequest) -> RerankResponse:
    models = request.app.state.models
    settings = request.app.state.settings

    if not models.loaded:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Models not loaded",
        )

    if not body.items:
        return RerankResponse(
            scores=[],
            model_version=settings.reranker_model_name,
            feature_version="0.1.0",
            device=settings.infer_device,
        )

    encoding = models.reranker_tokenizer(
        [item.instance_text for item in body.items],
        [item.sense_gloss for item in body.items],
        return_tensors="pt",
        padding=True,
        truncation=True,
        max_length=512,
    )
    model_inputs = {
        key: value.to(settings.infer_device)
        for key, value in encoding.items()
    }
    with torch.inference_mode():
        outputs = models.reranker_model(**model_inputs)

    if outputs.logits.shape[-1] == 1:
        scores = torch.sigmoid(outputs.logits.squeeze(-1)).tolist()
    else:
        scores = torch.softmax(outputs.logits, dim=-1)[:, 1].tolist()

    return RerankResponse(
        scores=[
            RerankScore(
                reference_sense_id=item.reference_sense_id,
                score=float(score),
            )
            for item, score in zip(body.items, scores)
        ],
        model_version=settings.reranker_model_name,
        feature_version="0.1.0",
        device=settings.infer_device,
    )
