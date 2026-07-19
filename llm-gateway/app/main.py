from typing import Annotated

from fastapi import Depends, FastAPI, HTTPException
from pydantic import BaseModel, Field, model_validator

from .gateway import LlmGateway

app = FastAPI(title="SenseFoundry LLM Gateway")


class EvidenceItem(BaseModel):
    id: str
    text: str = Field(min_length=1)


class DraftRequest(BaseModel):
    headword: str = Field(min_length=1)
    pos: str = Field(min_length=1)
    evidence_ids: list[str]
    evidence_items: list[EvidenceItem]

    @model_validator(mode="after")
    def validate_evidence(self) -> "DraftRequest":
        if not self.evidence_ids or not self.evidence_items:
            raise ValueError("evidence is required")
        item_ids = {item.id for item in self.evidence_items}
        if any(evidence_id not in item_ids for evidence_id in self.evidence_ids):
            raise ValueError("every evidence_id must have an evidence item")
        return self


class DraftResponse(BaseModel):
    definition: str
    evidence_ids_used: list[str]


def get_gateway() -> LlmGateway:
    return LlmGateway()


@app.post("/draft-definition", response_model=DraftResponse)
async def draft_definition(
    request: DraftRequest,
    gateway: Annotated[LlmGateway, Depends(get_gateway)],
) -> DraftResponse:
    evidence_by_id = {item.id: item.text for item in request.evidence_items}
    definition = await gateway.draft_definition(
        request.headword,
        request.pos,
        [evidence_by_id[evidence_id] for evidence_id in request.evidence_ids],
    )
    return DraftResponse(
        definition=definition,
        evidence_ids_used=request.evidence_ids,
    )


@app.get("/health")
async def health(
    gateway: Annotated[LlmGateway, Depends(get_gateway)],
) -> dict[str, str | bool]:
    configured = bool(gateway.api_key)
    reachable = await gateway.reachable() if configured else False
    if not configured or not reachable:
        raise HTTPException(
            status_code=503,
            detail={"configured": configured, "reachable": reachable},
        )
    return {"status": "ok", "configured": True, "reachable": True}
